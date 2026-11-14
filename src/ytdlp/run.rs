//! 다운로드 한 건. 재생목록이면 항목만 펼쳐 알리고 끝난다.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use super::parse::{
    archive_ids, final_ext, friendly_error, numbered_template, parse_line, parse_probe, Line, PlaylistEntry, Probe,
    ProgressMeter, PRINT_ARGS, PROBE_ARGS,
};
use super::process::{drain_stderr, run, KillGroup};
use crate::options::{full_args, split_extra, AppSettings, DownloadOptions, PlaylistContext};

/// 실행 파일 위치.
#[derive(Clone, Debug)]
pub struct Tools {
    pub ytdlp: PathBuf,
    /// `ffmpeg.exe` 가 든 폴더. 없으면 yt-dlp 가 PATH 에서 찾는다.
    pub ffmpeg_dir: Option<PathBuf>,
}

/// 다운로드를 누른 순간의 값으로 만든 작업.
#[derive(Clone, Debug)]
pub struct JobSpec {
    pub url: String,
    pub opts: DownloadOptions,
    pub settings: AppSettings,
    pub folder: String,
    /// 받은 기록 파일.
    pub archive: PathBuf,
    /// 재생목록에서 펼친 항목이면 그 자리. 항목은 정보 조회 없이 바로 받는다.
    pub playlist: Option<PlaylistContext>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// 추출기 이름과 영상 주소.
    Site { key: String, url: String },
    Title(String),
    /// 퍼센트, 상태 문구.
    Progress(f32, String),
    /// 저장된 파일 경로.
    Done(String),
    /// 이미 받은 기록이 있어 받지 않음.
    Skipped,
    Failed(String),
    /// 재생목록을 펼친 결과. 받을 항목과 받은 기록 때문에 건너뛴 개수.
    Expanded { title: String, entries: Vec<PlaylistEntry>, skipped: u32 },
}

fn missing() -> &'static str {
    crate::lang::tr("yt-dlp를 찾을 수 없음. 앱을 다시 설치하세요", "yt-dlp not found. Reinstall the app")
}

/// 한 건을 끝까지 돌린다. 마지막 이벤트는 `Done`/`Skipped`/`Failed`/`Expanded` 중 하나다.
pub fn run_job(spec: &JobSpec, tools: &Tools, group: &KillGroup, emit: &dyn Fn(Event)) {
    emit(job(spec, tools, group, emit));
}

fn job(spec: &JobSpec, tools: &Tools, group: &KillGroup, emit: &dyn Fn(Event)) -> Event {
    if !tools.ytdlp.is_file() {
        return Event::Failed(missing().into());
    }
    let archive = spec.archive.to_string_lossy();
    let mut args = full_args(&spec.opts, &spec.settings, &spec.folder, &archive, spec.playlist.as_ref());

    if spec.playlist.is_none() {
        match probe(spec, tools, group, &args) {
            Err(message) => return Event::Failed(message),
            Ok(Probe::Playlist { title, site_key, webpage_url, entries, skipped }) => {
                emit(Event::Site { key: site_key, url: webpage_url });
                return Event::Expanded { title, entries, skipped };
            }
            Ok(Probe::Single { title, site_key, webpage_url, filename, archive_id }) => {
                emit(Event::Site { key: site_key, url: webpage_url });
                emit(Event::Title(title));
                if spec.opts.skip_archived && recorded(&spec.archive).contains(&archive_id) {
                    return Event::Skipped;
                }
                if let (false, Some(filename)) = (spec.settings.overwrite, filename) {
                    // 앱이 넣은 첫 -o. 고급 옵션에 -o 가 또 있으면 yt-dlp 는 그쪽을 쓴다.
                    if let Some(i) = args.iter().position(|a| a == "-o") {
                        args[i + 1] = numbered_template(&args[i + 1], &filename, &final_ext(&spec.opts), &|p| p.exists());
                    }
                }
            }
        }
    }
    download(spec, tools, group, args, emit)
}

fn recorded(archive: &Path) -> std::collections::HashSet<String> {
    archive_ids(&std::fs::read_to_string(archive).unwrap_or_default())
}

/// 제목·사이트·재생목록 여부를 알아본다. 받은 기록은 넘기지 않고 직접 대조해 건너뛴 개수를 센다.
fn probe(spec: &JobSpec, tools: &Tools, group: &KillGroup, args: &[String]) -> Result<Probe, String> {
    let mut probe_args: Vec<&str> = Vec::new();
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
        } else if arg == "--download-archive" {
            skip = true;
        } else {
            probe_args.push(arg);
        }
    }
    let mut cmd = Command::new(&tools.ytdlp);
    cmd.args(&probe_args).args(PROBE_ARGS).arg("--").arg(&spec.url);
    let out = match run(group, &mut cmd, None) {
        Ok(Some(out)) => out,
        _ => return Err(missing().into()),
    };
    let ids = if spec.opts.skip_archived { recorded(&spec.archive) } else { Default::default() };
    match parse_probe(&out.stdout, &spec.url, &|id| ids.contains(id)) {
        Ok(found) if out.success => Ok(found),
        _ => Err(friendly_error(&out.stderr)),
    }
}

fn download(spec: &JobSpec, tools: &Tools, group: &KillGroup, args: Vec<String>, emit: &dyn Fn(Event)) -> Event {
    let mut cmd = Command::new(&tools.ytdlp);
    cmd.args(&args).args(PRINT_ARGS);
    if let Some(dir) = &tools.ffmpeg_dir {
        cmd.arg("--ffmpeg-location").arg(dir);
    }
    cmd.arg("--").arg(&spec.url);
    let Ok(mut child) = group.spawn(&mut cmd) else {
        return Event::Failed(missing().into());
    };
    let stderr = drain_stderr(&mut child);
    let mut meter = ProgressMeter::new();
    let mut file = None;
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut bytes = Vec::new();
        while matches!(reader.read_until(b'\n', &mut bytes), Ok(n) if n > 0) {
            let line = parse_line(&String::from_utf8_lossy(&bytes));
            bytes.clear();
            if let Line::File(path) = &line {
                file = Some(path.clone());
            }
            if let Some((pct, status)) = meter.on_line(&line, Instant::now()) {
                emit(Event::Progress(pct, status));
            }
        }
    }
    let success = child.wait().map(|s| s.success()).unwrap_or(false);
    let stderr = stderr.join().unwrap_or_default();
    match (success, file) {
        (true, Some(path)) => Event::Done(path),
        // 받은 기록에 있거나 고급 옵션 때문에 받지 않았다
        (true, None) => Event::Skipped,
        (false, _) => Event::Failed(friendly_error(&stderr)),
    }
}

/// 고급 옵션을 yt-dlp 로 확인한다. 링크 없이 돌리면 옵션이 맞을 때 "링크가 없다" 는 오류만 난다.
/// 틀리면 화면 문구, 맞거나 확인하지 못했으면 `None`.
pub fn check_extra_args(tools: &Tools, group: &KillGroup, text: &str) -> Option<String> {
    let mut cmd = Command::new(&tools.ytdlp);
    cmd.arg("--ignore-config").args(split_extra(text));
    let out = run(group, &mut cmd, Some(Duration::from_secs(10))).ok()??;
    let error = out.stderr.lines().rev().find_map(|l| l.split_once("error: ").map(|(_, msg)| msg.trim().to_string()));
    match error {
        Some(msg) if msg.starts_with("You must provide at least one URL") => None,
        Some(msg) => Some(crate::trf!("고급 옵션 오류: {msg}", "Advanced options error: {msg}")),
        // 오류 없이 끝났다: --version, --help 처럼 받지 않고 끝내는 옵션
        None => Some(crate::lang::tr("고급 옵션 오류: 다운로드하지 않는 옵션 포함", "Advanced options error: includes an option that doesn't download").into()),
    }
}

/// `yt-dlp --version`.
pub fn ytdlp_version(path: &Path, group: &KillGroup) -> Option<String> {
    let mut cmd = Command::new(path);
    cmd.arg("--version");
    let out = run(group, &mut cmd, Some(Duration::from_secs(30))).ok()??;
    let version = out.stdout.trim().to_string();
    (out.success && !version.is_empty()).then_some(version)
}
