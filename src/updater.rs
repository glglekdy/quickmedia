//! GitHub 릴리즈의 설치 파일로 자기 자신을 업데이트한다.
//!
//! 1. `fetch_latest()`  releases/latest 를 읽어 지금보다 새 버전인지 본다.
//! 2. `download()`      QuickMedia-Setup-*.exe 자산을 받아 임시 폴더에 둔다.
//! 3. `launch_apply()`  도우미 스크립트를 띄우고 앱은 종료한다. 스크립트가 앱이
//!    완전히 끝나기를 기다렸다가 설치 파일을 조용히 실행한다.
//!    설치가 끝나면 설치 파일이 앱을 다시 띄운다.
//!
//! 실행 중인 앱 파일은 윈도우가 잠그고 있어 덮어쓸 수 없어서 마지막 단계는
//! 바깥 프로세스가 맡는다.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;

use crate::app_info::{NAME, REPO, VERSION};
use crate::http::{self, HttpError};

pub const LATEST_URL: &str = "https://api.github.com/repos/glglekdy/quickmedia/releases/latest";

/// 받은 설치 파일과 도우미 스크립트를 두는 곳.
pub fn update_dir() -> PathBuf {
    std::env::temp_dir().join(format!("{NAME}-update"))
}

/// `v1.2.3`, `1.2`, `1.2.3-beta` 모두 숫자 부분만 비교한다. 끝의 0 은 뗀다(1.1.0 == 1.1).
pub fn parse_version(text: &str) -> Vec<u32> {
    let text = text.trim_start();
    let text = text.strip_prefix(['v', 'V']).unwrap_or(text);
    let numbers: &str = text.split(|c: char| !(c.is_ascii_digit() || c == '.')).next().unwrap_or("");
    let mut parts: Vec<u32> = Vec::new();
    for piece in numbers.split('.') {
        match piece.parse() {
            Ok(n) => parts.push(n),
            Err(_) => break,
        }
    }
    while parts.len() > 1 && parts.last() == Some(&0) {
        parts.pop();
    }
    parts
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    parse_version(latest) > parse_version(current)
}

/// 소스로 실행 중(개발용 빌드)이면 설치된 앱이 아니라 덮어쓸 것이 없다.
pub fn can_self_update() -> bool {
    !cfg!(debug_assertions)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    /// 릴리즈 설명(마크다운).
    pub notes: String,
    pub asset_url: String,
    /// 0 이면 GitHub 가 알려 주지 않았다.
    pub asset_size: u64,
    /// 소문자 hex. GitHub 가 주지 않았으면 빈 글.
    pub sha256: String,
}

/// 릴리즈 자산 중 설치 파일. 다른 exe 는 보지 않는다.
pub fn is_setup_name(name: &str) -> bool {
    let low = name.to_lowercase();
    let prefix = format!("{}-setup-", NAME.to_lowercase());
    low.strip_prefix(&prefix).and_then(|rest| rest.strip_suffix(".exe")).is_some_and(|version| !version.is_empty())
}

/// releases/latest 응답에서 지금(`current`)보다 새 릴리즈. `is_asset` 이 받을 자산을 이름으로 고른다.
pub fn parse_release(json: &str, current: &str, is_asset: &dyn Fn(&str) -> bool) -> Option<Release> {
    let data: Value = serde_json::from_str(json).ok()?;
    let text = |v: &Value, key: &str| v.get(key).and_then(Value::as_str).unwrap_or("").to_string();
    let version = text(&data, "tag_name").trim_start_matches(['v', 'V']).to_string();
    if !is_newer(&version, current) {
        return None;
    }
    let asset = data.get("assets")?.as_array()?.iter().find(|a| is_asset(&text(a, "name")))?;
    let asset_url = text(asset, "browser_download_url");
    if asset_url.is_empty() {
        return None;
    }
    let digest = text(asset, "digest");
    Some(Release {
        version,
        notes: text(&data, "body"),
        asset_url,
        asset_size: asset.get("size").and_then(Value::as_u64).unwrap_or(0),
        sha256: digest.strip_prefix("sha256:").map(str::to_lowercase).unwrap_or_default(),
    })
}

/// `repo` 의 최신 정식 릴리즈가 `current` 보다 새것이면 돌려준다.
pub fn fetch_release(repo: &str, current: &str, is_asset: &dyn Fn(&str) -> bool) -> Result<Option<Release>, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let agent = http::agent("");
    match http::get_bytes(&agent, &url, "application/vnd.github+json", 4 * 1024 * 1024, Duration::from_secs(20)) {
        Ok(body) => Ok(parse_release(&String::from_utf8_lossy(&body), current, is_asset)),
        Err(HttpError::Status(404)) => Ok(None), // 릴리즈가 하나도 없음
        Err(HttpError::Status(code)) => Err(format!("릴리즈 정보 조회 실패 (HTTP {code})")),
        Err(_) => Err("GitHub 연결 실패".to_string()),
    }
}

/// 지금보다 새 정식 릴리즈. 없으면 `None`.
pub fn fetch_latest() -> Result<Option<Release>, String> {
    fetch_release(REPO, VERSION, &is_setup_name)
}

/// 받은 파일이 릴리즈와 맞는지.
pub fn verify(size: u64, sha256_hex: &str, r: &Release) -> Result<(), String> {
    if r.asset_size > 0 && size != r.asset_size {
        return Err("파일 크기 불일치. 다시 시도하세요.".to_string());
    }
    if !r.sha256.is_empty() && !sha256_hex.eq_ignore_ascii_case(&r.sha256) {
        return Err("파일 손상 (SHA-256 불일치)".to_string());
    }
    Ok(())
}

/// 릴리즈 자산을 `target` 으로 받는다. 받는 동안은 `<target>.part` 에 쓰고, 확인을 마친 뒤 이름을 바꾼다.
pub fn download_to(r: &Release, target: &Path, progress: &dyn Fn(u64, u64)) -> Result<(), String> {
    let part = PathBuf::from(format!("{}.part", target.display()));
    let known = r.asset_size;
    let report = |done: u64, total: u64| progress(done, if total > 0 { total } else { known });
    let (size, sha) = http::download(&http::agent(""), &r.asset_url, &part, &report)
        .map_err(|_| "업데이트 파일 다운로드 중단. 다시 시도하세요.".to_string())?;
    if let Err(message) = verify(size, &sha, r) {
        let _ = std::fs::remove_file(&part);
        return Err(message);
    }
    let _ = std::fs::remove_file(target);
    std::fs::rename(&part, target).map_err(|_| {
        let _ = std::fs::remove_file(&part);
        "업데이트 파일 저장 실패. 다시 시도하세요.".to_string()
    })
}

/// 설치 파일을 임시 폴더에 받는다.
pub fn download(r: &Release, progress: &dyn Fn(u64, u64)) -> Result<PathBuf, String> {
    let target = update_dir().join(format!("{NAME}-Setup-{}.exe", r.version));
    download_to(r, &target, progress)?;
    Ok(target)
}

/// 앱이 끝나기를 기다렸다가 설치 파일을 조용히 실행한다.
pub const APPLY_SCRIPT: &str = r#"
param([int]$WaitPid, [string]$Setup, [string]$Exe)
Wait-Process -Id $WaitPid -Timeout 120 -ErrorAction SilentlyContinue
$p = Start-Process -FilePath $Setup -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -PassThru
# -Wait 는 설치 파일이 다시 띄운 앱까지 기다린다. 설치 파일 프로세스만 기다린다.
$p.WaitForExit()
# 성공하면 설치 파일의 [Run] 이 앱을 다시 띄운다. 실패하면 Inno 가 되돌려 둔 기존 앱을 띄운다.
if ($p.ExitCode -ne 0 -and (Test-Path $Exe)) { Start-Process -FilePath $Exe -WorkingDirectory (Split-Path $Exe) }
Remove-Item $Setup -Force -ErrorAction SilentlyContinue
"#;

/// 도우미 PowerShell 을 띄울 인자. 하나씩 넘기므로 공백·한글 경로도 그대로 간다.
pub fn apply_command(script: &Path, pid: u32, setup: &Path, exe: &Path) -> Vec<String> {
    let path = |p: &Path| p.to_string_lossy().into_owned();
    [
        "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden",
        "-File", &path(script), "-WaitPid", &pid.to_string(), "-Setup", &path(setup), "-Exe", &path(exe),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

/// 도우미를 띄운다. 호출한 쪽은 곧바로 앱을 종료해야 한다.
pub fn launch_apply(setup: &Path) -> Result<(), String> {
    if !can_self_update() {
        return Err("개발용 빌드에서는 자동 업데이트 불가".to_string());
    }
    let failed = |_| "업데이트 시작 실패. 다시 시도하세요.".to_string();
    let exe = std::env::current_exe().map_err(failed)?;
    let script = update_dir().join("apply_update.ps1");
    std::fs::create_dir_all(update_dir()).map_err(failed)?;
    // PowerShell 5.1 은 BOM 이 없으면 ANSI 로 읽어 한글 경로가 깨진다.
    std::fs::write(&script, format!("\u{feff}{APPLY_SCRIPT}")).map_err(failed)?;
    let command = apply_command(&script, std::process::id(), setup, &exe);
    // 앱의 작업 개체에 넣지 않는다. 앱이 끝난 뒤에도 살아 있어야 한다.
    Command::new(&command[0])
        .args(&command[1..])
        .creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(failed)
}

/// 릴리스 설명의 한 줄. 제목 줄은 굵게 보인다.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteLine {
    pub text: String,
    pub heading: bool,
}

/// 릴리스 설명(마크다운)을 줄로. 제목(`#`)과 목록(`-`, `*`)만 알아보고 나머지는 글자 그대로 둔다.
/// 빈 줄은 한 줄로 줄인다.
pub fn note_lines(markdown: &str) -> Vec<NoteLine> {
    let mut lines: Vec<NoteLine> = Vec::new();
    for raw in markdown.lines() {
        let text = raw.trim_end();
        let hashes = text.chars().take_while(|c| *c == '#').count();
        let after_hashes = &text[hashes..];
        let bullet = text.trim_start().strip_prefix(['-', '*']).filter(|rest| rest.starts_with(char::is_whitespace));
        let line = if (1..=6).contains(&hashes) && after_hashes.starts_with(char::is_whitespace) {
            NoteLine { text: after_hashes.trim_start().to_string(), heading: true }
        } else if let Some(rest) = bullet {
            NoteLine { text: format!("• {}", rest.trim_start()), heading: false }
        } else {
            NoteLine { text: text.to_string(), heading: false }
        };
        if !line.text.is_empty() || lines.last().is_some_and(|l| !l.text.is_empty()) {
            lines.push(line);
        }
    }
    while lines.last().is_some_and(|l| l.text.is_empty()) {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert_eq!(parse_version("v1.2.0"), [1, 2]);
        assert_eq!(parse_version("1.2.3-beta"), [1, 2, 3]);
        assert_eq!(parse_version("V2.0"), [2]);
        assert_eq!(parse_version(" 2026.11.05"), [2026, 11, 5]);
        assert!(parse_version("").is_empty() && parse_version("beta").is_empty());
        assert_eq!(parse_version("0.0.0"), [0]);
        assert!(is_newer("1.1.0", "1.0.1"));
        assert!(!is_newer("v1.1", "1.1.0"));
        assert!(!is_newer("1.0.9", "1.1.0"));
        assert!(is_newer("1.1.0", "0.0.0-dev"));
        assert!(is_newer("2.0.0", "1.2.0") && is_newer("1.2.1", "1.2"));
        assert!(!is_newer("", "1.0") && !is_newer("garbage", "1.0"));
    }

    #[test]
    fn setup_asset_name() {
        assert!(is_setup_name("QuickMedia-Setup-1.2.0.exe"));
        assert!(is_setup_name("quickmedia-setup-1.2.0.EXE"));
        for other in ["YoutubeDownloader.exe", "QuickMedia.exe", "QuickMedia-Setup-.exe", "QuickMedia-1.1.0.apk", "notes.txt", ""] {
            assert!(!is_setup_name(other), "{other}");
        }
    }

    const JSON: &str = r#"{"tag_name":"v1.1.0","body":"notes","assets":[
      {"name":"notes.txt","browser_download_url":"https://x/txt","size":9,"digest":null},
      {"name":"QuickMedia-Setup-1.1.0.exe","browser_download_url":"https://x/exe","size":42,"digest":"sha256:ABCD"},
      {"name":"QuickMedia-1.1.0.apk","browser_download_url":"https://x/apk","size":1,"digest":"sha256:aa"}]}"#;

    #[test]
    fn release_picks_setup_asset() {
        let expected = Release {
            version: "1.1.0".into(), notes: "notes".into(), asset_url: "https://x/exe".into(), asset_size: 42, sha256: "abcd".into(),
        };
        assert_eq!(parse_release(JSON, "1.0.1", &is_setup_name), Some(expected));
    }

    #[test]
    fn release_not_newer_or_without_asset_is_none() {
        assert_eq!(parse_release(JSON, "1.1.0", &is_setup_name), None);
        assert_eq!(parse_release(JSON, "2.0.0", &is_setup_name), None);
        assert_eq!(parse_release(&JSON.replace("QuickMedia-Setup-1.1.0.exe", "other.zip"), "1.0.0", &is_setup_name), None);
        assert_eq!(parse_release(r#"{"tag_name":"v9.0.0","body":"n"}"#, "1.0.0", &is_setup_name), None);
        assert_eq!(parse_release(r#"{"tag_name":"v9.0.0","assets":[]}"#, "1.0.0", &is_setup_name), None);
        assert_eq!(parse_release("not json", "1.0.0", &is_setup_name), None);
        assert_eq!(parse_release("{}", "1.0.0", &is_setup_name), None);
    }

    #[test]
    fn release_tolerates_missing_fields() {
        let json = r#"{"tag_name":"V2.0","body":null,"assets":[{"name":"QuickMedia-Setup-2.0.exe","browser_download_url":"u","digest":null}]}"#;
        let r = parse_release(json, "1.2.0", &is_setup_name).unwrap();
        assert_eq!((r.version.as_str(), r.notes.as_str(), r.asset_size, r.sha256.as_str()), ("2.0", "", 0, ""));
        // 받을 주소가 없는 자산은 쓸 수 없다
        let json = r#"{"tag_name":"v2.0","assets":[{"name":"QuickMedia-Setup-2.0.exe"}]}"#;
        assert_eq!(parse_release(json, "1.2.0", &is_setup_name), None);
    }

    #[test]
    fn verify_checks_size_and_hash() {
        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let r = Release { version: "1.1.0".into(), notes: String::new(), asset_url: "u".into(), asset_size: 3, sha256: abc.into() };
        assert_eq!(verify(3, abc, &r), Ok(()));
        assert_eq!(verify(3, &abc.to_uppercase(), &r), Ok(()));
        assert_eq!(verify(4, "x", &r), Err("파일 크기 불일치. 다시 시도하세요.".to_string()));
        assert_eq!(verify(3, "00", &r), Err("파일 손상 (SHA-256 불일치)".to_string()));
        // GitHub 가 크기·digest 를 주지 않은 자산이면 그 검사는 건너뛴다
        let bare = Release { asset_size: 0, sha256: String::new(), ..r };
        assert_eq!(verify(5, "ff", &bare), Ok(()));
    }

    #[test]
    fn apply_command_passes_paths_verbatim() {
        let setup = Path::new("C:/임시 폴더/QuickMedia-Setup-2.0.0.exe");
        let exe = Path::new("D:/내 프로그램/QuickMedia/QuickMedia.exe");
        let cmd = apply_command(Path::new("C:/t/a.ps1"), 42, setup, exe);
        assert_eq!(cmd[..2], ["powershell.exe", "-NoProfile"]);
        let after = |flag: &str| cmd[cmd.iter().position(|a| a == flag).unwrap() + 1].clone();
        assert_eq!(after("-File"), "C:/t/a.ps1");
        assert_eq!(after("-WaitPid"), "42");
        assert_eq!(after("-Setup"), "C:/임시 폴더/QuickMedia-Setup-2.0.0.exe");
        assert_eq!(after("-Exe"), "D:/내 프로그램/QuickMedia/QuickMedia.exe");
    }

    #[test]
    fn apply_script_waits_only_for_setup_process() {
        // Start-Process -Wait 는 설치 파일이 다시 띄운 앱까지 기다려서, 도우미가 앱이 꺼질 때까지 남는다.
        let setup_line = APPLY_SCRIPT.lines().find(|l| l.starts_with("$p = Start-Process")).unwrap();
        assert!(!setup_line.contains("-Wait"));
        assert!(setup_line.contains("'/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART'"));
        assert!(APPLY_SCRIPT.contains("$p.WaitForExit()"));
        assert!(APPLY_SCRIPT.contains("param([int]$WaitPid, [string]$Setup, [string]$Exe)"));
    }

    #[test]
    fn debug_builds_do_not_self_update() {
        assert_eq!(can_self_update(), !cfg!(debug_assertions));
        assert!(update_dir().ends_with("QuickMedia-update"));
        assert!(LATEST_URL.ends_with("/repos/glglekdy/quickmedia/releases/latest"));
    }

    fn line(text: &str) -> NoteLine {
        NoteLine { text: text.into(), heading: false }
    }

    #[test]
    fn note_lines_read_headings_and_bullets() {
        // 릴리스 설명은 마크다운이다. 제목과 목록만 알아보고, 빈 줄은 한 줄로 줄인다.
        assert_eq!(
            note_lines("\r\n## 고친 점\r\n\r\n- 하나\r\n* 둘\r\n\r\n\r\n끝\r\n"),
            [NoteLine { text: "고친 점".into(), heading: true }, line(""), line("• 하나"), line("• 둘"), line(""), line("끝")]
        );
        assert!(note_lines("").is_empty() && note_lines("\n\n").is_empty());
        assert_eq!(note_lines("#해시태그\n  - 들여쓴 목록\n-붙은 글"), [line("#해시태그"), line("• 들여쓴 목록"), line("-붙은 글")]);
    }
}
