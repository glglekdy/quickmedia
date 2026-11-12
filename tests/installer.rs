//! 설치 스크립트와 빌드 스크립트가 지켜야 하는 것. 설치된 앱의 업데이트가 여기에 기댄다.

use std::path::Path;

fn read(path: &str) -> String {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    text.trim_start_matches('\u{feff}').replace("\r\n", "\n")
}

/// `[이름]` 섹션의 줄들.
fn section<'a>(iss: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    iss.lines()
        .skip_while(|l| l.trim() != header)
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with(';'))
        .collect()
}

#[test]
fn installer_is_the_same_app_as_1x() {
    // AppId 가 바뀌면 설치된 1.x 가 2.0 을 다른 앱으로 보고 나란히 설치한다
    let iss = read("installer/QuickMedia.iss");
    let setup = section(&iss, "Setup");
    assert!(setup.contains(&"AppId={{8F3C2A71-5D4E-4B9A-9C61-2E7D0F4A3B85}"), "{setup:?}");
    assert!(setup.contains(&"OutputBaseFilename=QuickMedia-Setup-{#AppVersion}"));
    assert!(setup.contains(&"PrivilegesRequired=lowest"));
    assert!(setup.contains(&"DefaultDirName={autopf}\\QuickMedia"));
}

#[test]
fn update_clears_old_python_files() {
    // 1.x(PyInstaller)의 _internal 이 남으면 100MB 쯤이 쓰이지 않은 채 자리만 차지한다
    let iss = read("installer/QuickMedia.iss");
    let delete = section(&iss, "InstallDelete");
    assert_eq!(delete, ["Type: filesandordirs; Name: \"{app}\\_internal\""]);
}

#[test]
fn installer_relaunches_after_silent_update() {
    let iss = read("installer/QuickMedia.iss");
    let run = section(&iss, "Run");
    assert!(run.iter().any(|l| l.contains("{app}\\QuickMedia.exe") && l.contains("skipifnotsilent")), "{run:?}");
    let files = section(&iss, "Files");
    assert!(files.iter().any(|l| l.contains("..\\dist\\QuickMedia\\*") && l.contains("recursesubdirs")), "{files:?}");
}

#[test]
fn build_script_bundles_the_three_executables() {
    let script = read("build.ps1");
    for name in ["QuickMedia.exe", "yt-dlp.exe", "ffmpeg.exe"] {
        assert!(script.contains(name), "{name}");
    }
    assert!(script.contains("cargo build --release") && !script.to_lowercase().contains("pyinstaller"));
}

#[test]
fn release_workflow_builds_only_the_windows_installer() {
    let workflow = read(".github/workflows/release.yml");
    assert!(workflow.contains("./build.ps1 -FFmpeg $env:FFMPEG -YtDlp $env:YTDLP"));
    assert!(workflow.contains("cargo test"));
    assert!(workflow.contains("files: dist/QuickMedia-Setup-*.exe"));
    let low = workflow.to_lowercase();
    assert!(!low.contains("apk") && !low.contains("android") && !low.contains("python"), "옛 작업이 남아 있다");
}

#[test]
fn version_line_is_where_the_workflow_expects_it() {
    // 워크플로는 Cargo.toml 에서 줄 처음의 `version = "…"` 하나만 바꾼다
    let toml = read("Cargo.toml");
    let lines: Vec<&str> = toml.lines().filter(|l| l.starts_with("version = \"")).collect();
    assert_eq!(lines, [format!("version = \"{}\"", env!("CARGO_PKG_VERSION")).as_str()]);
}
