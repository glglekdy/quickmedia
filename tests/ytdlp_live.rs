//! 실제 yt-dlp 와 네트워크로 도는 시험. 평소에는 건너뛴다.
//!
//!     ./scripts/fetch-tools.ps1
//!     cargo test --test ytdlp_live -- --ignored --test-threads 2

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use quickmedia::options::{AppSettings, DownloadOptions};
use quickmedia::ytdlp::{check_extra_args, run_job, ytdlp_version, Event, JobSpec, KillGroup, Tools};

const SHORT: &str = "https://www.youtube.com/watch?v=jNQXAC9IVRw";
const KOREAN: &str = "https://www.youtube.com/watch?v=x8dS6_7Fuxg";
const PLAYLIST: &str = "https://www.youtube.com/playlist?list=PLBCF2DAC6FFB574DE";

fn tools() -> Tools {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("build").join("tools");
    assert!(dir.join("yt-dlp.exe").exists(), "먼저 ./scripts/fetch-tools.ps1 을 실행하세요");
    Tools { ytdlp: dir.join("yt-dlp.exe"), ffmpeg_dir: Some(dir) }
}

/// 시험마다 비어 있는 폴더.
fn folder(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join("live").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn spec(url: &str, dir: &Path, f: impl FnOnce(&mut DownloadOptions)) -> JobSpec {
    let mut opts = DownloadOptions { quality: "360".into(), ..Default::default() };
    f(&mut opts);
    JobSpec {
        url: url.into(),
        opts,
        settings: AppSettings::default(),
        folder: dir.to_string_lossy().into_owned(),
        archive: dir.join("archive.txt"),
        playlist: None,
    }
}

fn run(spec: &JobSpec) -> Vec<Event> {
    let events = Mutex::new(Vec::new());
    let group = KillGroup::new().unwrap();
    run_job(spec, &tools(), &group, &|e| events.lock().unwrap().push(e));
    events.into_inner().unwrap()
}

fn done_path(events: &[Event]) -> &str {
    match events.last() {
        Some(Event::Done(path)) => path,
        other => panic!("완료로 끝나야 한다: {other:?}\n{events:?}"),
    }
}

#[test]
#[ignore]
fn single_video_downloads() {
    let dir = folder("single");
    let events = run(&spec(SHORT, &dir, |_| {}));
    assert!(events.iter().any(|e| matches!(e, Event::Title(t) if t == "Me at the zoo")), "{events:?}");
    assert!(events.iter().any(|e| matches!(e, Event::Site { key, .. } if key == "Youtube")));
    assert!(events.iter().any(|e| matches!(e, Event::Progress(p, _) if *p > 0.0)));
    let path = done_path(&events);
    assert!(Path::new(path).is_file() && path.ends_with("Me at the zoo.mp4"), "{path}");
}

#[test]
#[ignore]
fn korean_folder_and_title() {
    let dir = folder("한글 폴더 🎵");
    let events = run(&spec(KOREAN, &dir, |_| {}));
    let path = done_path(&events);
    assert!(Path::new(path).is_file(), "{path}");
    assert!(path.contains("한글 폴더 🎵") && path.contains("안녕하세요"), "{path}");
}

#[test]
#[ignore]
fn same_name_gets_number() {
    let dir = folder("number");
    let audio = |o: &mut DownloadOptions| o.kind = "audio".into();
    let first = run(&spec(SHORT, &dir, audio));
    assert!(done_path(&first).ends_with("Me at the zoo.mp3"));
    let second = run(&spec(SHORT, &dir, audio));
    let path = done_path(&second);
    assert!(path.ends_with("Me at the zoo (1).mp3"), "{path}");
    assert!(Path::new(path).is_file() && dir.join("Me at the zoo.mp3").is_file());
}

#[test]
#[ignore]
fn playlist_expands() {
    let dir = folder("playlist");
    let events = run(&spec(PLAYLIST, &dir, |o| o.playlist = "all".into()));
    match events.last() {
        Some(Event::Expanded { title, entries, skipped }) => {
            assert_eq!(title, "Google Search Stories");
            assert!(entries.len() > 5 && *skipped == 0);
            assert!(entries[0].url.starts_with("https://www.youtube.com/watch?v="));
        }
        other => panic!("{other:?}"),
    }
    assert!(std::fs::read_dir(&dir).unwrap().next().is_none(), "펼치기만 하고 받지 않는다");
}

#[test]
#[ignore]
fn archived_is_skipped() {
    let dir = folder("archive");
    let audio = |o: &mut DownloadOptions| {
        o.kind = "audio".into();
        o.skip_archived = true;
    };
    done_path(&run(&spec(SHORT, &dir, audio)));
    let again = run(&spec(SHORT, &dir, audio));
    assert_eq!(again.last(), Some(&Event::Skipped), "{again:?}");
}

#[test]
#[ignore]
fn bad_url_fails() {
    let dir = folder("bad");
    let events = run(&spec("https://example.com/", &dir, |_| {}));
    assert_eq!(events.last(), Some(&Event::Failed("지원하지 않는 링크예요".into())), "{events:?}");
    let events = run(&spec("https://www.youtube.com/watch?v=aaaaaaaaaaa", &dir, |_| {}));
    assert_eq!(events.last(), Some(&Event::Failed("삭제되었거나 볼 수 없는 영상이에요".into())));
}

#[test]
#[ignore]
fn missing_ytdlp_is_reported() {
    let dir = folder("missing");
    let events = Mutex::new(Vec::new());
    let group = KillGroup::new().unwrap();
    let tools = Tools { ytdlp: dir.join("nope.exe"), ffmpeg_dir: None };
    run_job(&spec(SHORT, &dir, |_| {}), &tools, &group, &|e| events.lock().unwrap().push(e));
    let events = events.into_inner().unwrap();
    assert_eq!(events, [Event::Failed("yt-dlp 를 찾지 못했어요. 앱을 다시 설치해 주세요".into())]);
}

#[test]
#[ignore]
fn extra_args_checked() {
    let group = KillGroup::new().unwrap();
    let t = tools();
    assert_eq!(check_extra_args(&t, &group, "--no-mtime --limit-rate 1M"), None);
    let msg = check_extra_args(&t, &group, "--nope").unwrap();
    assert_eq!(msg, "고급 옵션을 읽지 못했어요: no such option: --nope");
    assert!(check_extra_args(&t, &group, "--limit-rate abc").unwrap().contains("invalid rate limit"));
    assert!(check_extra_args(&t, &group, "--version").is_some(), "받기를 막는 옵션");
    let version = ytdlp_version(&t.ytdlp, &group).unwrap();
    assert!(version.starts_with("20") && version.matches('.').count() >= 2, "{version}");
}

#[test]
#[ignore]
fn closing_the_group_kills_the_download() {
    let dir = folder("kill");
    let group = KillGroup::new().unwrap();
    let t = tools();
    let mut cmd = std::process::Command::new(&t.ytdlp);
    // 느리게 받게 해서 끝나기 전에 그룹을 닫는다
    cmd.args(["--ignore-config", "-r", "20K", "-P"]).arg(&dir).arg(KOREAN);
    let mut child = group.spawn(&mut cmd).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(4));
    assert!(child.try_wait().unwrap().is_none(), "아직 받는 중이어야 한다");
    let closed = std::time::Instant::now();
    drop(group);
    child.wait().unwrap();
    // 종료 코드는 0 으로 보일 수 있다. 곧바로 끝났고 받다 만 파일만 남았는지로 본다.
    assert!(closed.elapsed() < std::time::Duration::from_secs(3), "{:?}", closed.elapsed());
    let names: Vec<String> =
        std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert!(!names.is_empty() && names.iter().all(|n| n.ends_with(".part") || n.ends_with(".ytdl")), "{names:?}");
}

#[test]
#[ignore]
fn ytdlp_update_installs() {
    use quickmedia::ytdlp_update::{fetch_latest, install};
    let bin = folder("bin");
    let group = KillGroup::new().unwrap();
    let release = fetch_latest("0").unwrap().expect("0 보다 새 릴리즈가 있어야 한다");
    assert!(release.asset_url.ends_with("/yt-dlp.exe") && release.asset_size > 1_000_000, "{release:?}");
    let seen = Mutex::new((0u64, 0u64));
    let found = install(&release, &bin, &group, &|done, total| *seen.lock().unwrap() = (done, total)).unwrap();
    assert_eq!(found.version, release.version);
    assert_eq!(found.path, bin.join(format!("yt-dlp-{}.exe", release.version)));
    assert_eq!(ytdlp_version(&found.path, &group).as_deref(), Some(release.version.as_str()));
    let (done, total) = *seen.lock().unwrap();
    assert!(done == total && done == release.asset_size, "{done}/{total}");
    // 지금 버전과 같으면 받을 게 없다
    assert_eq!(fetch_latest(&release.version).unwrap(), None);
}
