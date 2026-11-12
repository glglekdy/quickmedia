# QuickMedia Rust 재작성 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windows 앱 QuickMedia 를 Python + PySide6 에서 Rust + Slint 로 다시 만들어 2.0.0 으로 내고, 안드로이드 앱을 지운다.

**Architecture:** 화면과 무관한 로직(옵션·인자·검사·대기열·yt-dlp 출력 파싱·업데이트 판단)은 라이브러리 크레이트의 순수 함수로 두고 `cargo test` 로 시험한다. 다운로드는 `yt-dlp.exe` 를 자식 프로세스로 띄워 정해진 형식의 줄을 읽는다. Slint 는 Rust 가 정한 상태를 그리기만 하고, 작업 스레드는 `slint::invoke_from_event_loop` 로 결과를 넘긴다.

**Tech Stack:** Rust stable (x86_64-pc-windows-msvc), Slint, serde/serde_json, winreg, ureq, sha2, image(ico·png), rfd, windows, Inno Setup 6, yt-dlp.exe, ffmpeg.exe

**Spec:** `docs/superpowers/specs/2026-11-12-quickmedia-rust-rewrite-design.md`

## Global Constraints

- Windows 전용. 릴리즈 빌드는 콘솔 창이 없다(`#![windows_subsystem = "windows"]`), 자식 프로세스는 `CREATE_NO_WINDOW`.
- 비동기 런타임을 쓰지 않는다. 작업마다 `std::thread`.
- 화면 문구는 Python 소스의 한글 문구를 글자 그대로 옮긴다. Python 소스(`ytdl/`, `tests/`)는 Task 16 까지 남아 있으니 문구·규칙·시험 경우의 출처로 읽는다.
- 설정은 `HKCU\Software\QuickMedia\QuickMedia`, 데이터 폴더는 `%APPDATA%\QuickMedia`. 키 이름과 필드 이름을 바꾸지 않는다.
- 설치 파일 이름 `QuickMedia-Setup-<버전>.exe`, exe 이름 `QuickMedia.exe`, Inno AppId 는 그대로.
- 버전은 `Cargo.toml` 한 곳(`2.0.0`). 코드에서는 `env!("CARGO_PKG_VERSION")`.
- 크레이트 구성: `src/lib.rs`(모듈 공개) + `src/main.rs`(얇은 진입점). 패키지 이름 `quickmedia`, 바이너리 이름 `QuickMedia`.
- 커밋 메시지는 이 저장소처럼 짧은 한글 한 줄, 끝에 `Co-Authored-By: Claude Fable 5.2 <noreply@anthropic.com>`.
- 각 Task 의 마지막 단계는 `cargo test` 와 `cargo clippy -- -D warnings` 가 통과한 뒤 커밋.

## Review Focus

1. **한글·특수문자 경로**: 저장 폴더나 제목에 한글·이모지·공백이 있어도 완료 경로가 깨지지 않고 탐색기에서 열린다. → Task 5(줄 파싱), Task 6(실제 실행).
2. **`%` 가 든 이름**: 재생목록 제목이나 이미 있는 파일 이름에 `%` 가 있어도 출력 템플릿이 깨지지 않는다(`%%`). → Task 3, Task 5.
3. **형식에 안 맞는 yt-dlp 출력**: 경고 줄, 빈 줄, CRLF, 잘린 줄이 섞여도 패닉 없이 무시한다. → Task 5.
4. **전체 크기를 모르는 다운로드**(라이브·조각 스트림의 `NA`): 0 으로 나누지 않고 진행률을 건너뛴다. → Task 5.
5. **이상한 레지스트리 값**: 형이 다르거나 목록 밖·범위 밖 값이면 기본값으로 켜진다. → Task 2, Task 7.

---

### Task 1: 프로젝트 뼈대와 yt-dlp CLI 검증

스펙의 "먼저 검증할 것". 뒤 Task 들이 기대는 사실을 실제 yt-dlp 로 확인하고 출력 샘플을 남긴다.

**Files:**
- Create: `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `scripts/fetch-tools.ps1`
- Create: `tests/fixtures/` (샘플), `docs/superpowers/plans/2026-11-12-ytdlp-cli-findings.md`
- Modify: `.gitignore` (`target/` 추가)

**Interfaces:**
- Produces: `build/tools/yt-dlp.exe`, `build/tools/ffmpeg.exe`(git 에 안 올라감). findings 문서의 결론. `tests/fixtures/probe_single.txt`, `probe_playlist.txt`, `download_two_streams.txt`, `download_unknown_size.txt`, `error_unavailable.txt`.

- [ ] **Step 1:** `Cargo.toml`(패키지 `quickmedia` 2.0.0, edition 2021, `[[bin]] name = "QuickMedia"`), 빈 `src/lib.rs`, `fn main() {}` 인 `src/main.rs` 를 만들고 `cargo build` 가 되는지 본다.
- [ ] **Step 2:** `scripts/fetch-tools.ps1 [-Dest build/tools]`: yt-dlp 최신 릴리즈의 `yt-dlp.exe` 와 BtbN `ffmpeg-master-latest-win64-gpl.zip` 의 `ffmpeg.exe` 를 받는다. 이미 있으면 건너뛴다. 실행해서 두 파일을 받는다.
- [ ] **Step 3:** 아래를 실제로 돌려 보고 결과를 findings 문서에 "질문 / 돌린 명령 / 결과 / 결론" 으로 적는다. 시험 링크는 짧은 공개 영상과 작은 재생목록, 한글 제목 영상 하나씩.
  1. `-J --flat-playlist` 단일 영상 결과에 파일 이름 필드(`filename` 또는 `_filename`)가 있는가. 없으면 `--print filename` 을 쓰는 조회 명령을 정한다.
  2. `--progress-template "download:QMP|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress._speed_str)s"`, `--print "before_dl:QMN|%(requested_formats.:.format_id)j"`, `--print "after_move:QMF|%(filepath)s"`, `--no-simulate`, `--newline`, `--no-warnings`, `--encoding utf-8` 를 함께 준 한 번의 실행에서 세 종류의 줄이 모두 stdout 에 나오는가. 스트림이 하나일 때 `QMN` 줄이 어떻게 나오는가(`NA`/`null`).
  3. 한글 제목·한글 저장 폴더에서 `QMF` 경로가 UTF-8 로 온전한가(`PYTHONUTF8=1`, `PYTHONIOENCODING=utf-8`).
  4. `--embed-subs --write-subs` 로 받은 뒤 자막 파일이 폴더에 남는가.
  5. `--download-archive` 에 이미 있는 영상을 받을 때 종료 코드와 출력.
  6. 없는 영상·지원 안 하는 링크의 stderr 형식과 종료 코드.
  7. 고급 인자 검사: 링크 없이 `yt-dlp --ignore-config <인자>` 를 실행했을 때, 인자가 맞으면 stderr 마지막 줄이 `You must provide at least one URL` 이고 틀리면 다른 오류인가. 걸리는 시간.
  8. `yt-dlp.exe` 한 번 실행의 기본 시간(`--version`).
- [ ] **Step 4:** 2·6 의 실제 stdout/stderr 와 1 의 JSON 을 `tests/fixtures/` 에 저장한다. JSON 은 필요한 필드만 남기고 줄여도 된다.
- [ ] **Step 5:** 결론이 이 계획이나 스펙과 다르면(예: `QMN` 템플릿 문법, 파일 이름 필드) findings 문서 맨 위 "계획과 다른 점" 에 적고, 해당 Task 의 값을 고친 뒤 커밋한다.

### Task 2: 옵션 모델

**Files:**
- Create: `src/options/mod.rs`, `src/options/model.rs`
- Reference: `ytdl/options.py:17-200`, `tests/test_options_model.py`

**Interfaces:**
- Produces:
  - `pub struct DownloadOptions`, `pub struct AppSettings`: 필드 이름·기본값이 Python 과 같다. 고르는 값은 `String`, 켜고 끄는 값은 `bool`, `retries`/`concurrency` 는 `u32`. `Clone + PartialEq + Debug + Default`.
  - `pub const QUALITIES, CONTAINERS, CODECS, AUDIO_FORMATS, AUDIO_QUALITIES, SUB_MODES, SUB_FORMATS, PLAYLIST_MODES, COOKIE_MODES, BROWSERS: &[&str]`, `pub const NAME_TEMPLATES: &[(&str, &str)]`, `pub const PRESET_KEYS: [&str; 3]`(`best`, `720p`, `mp3`)
  - `pub fn preset_label(key: &str) -> &'static str`
  - `pub fn apply_preset(opts: &DownloadOptions, key: &str) -> DownloadOptions`
  - `pub fn matching_preset(opts: &DownloadOptions) -> Option<&'static str>`
  - `pub fn options_to_map(o: &DownloadOptions) -> BTreeMap<String, String>`, `pub fn settings_to_map(s: &AppSettings) -> BTreeMap<String, String>` (bool 은 `"true"`/`"false"`)
  - `pub fn options_from_map(d: &HashMap<String, String>, legacy_format: Option<&str>) -> DownloadOptions`, `pub fn settings_from_map(d: &HashMap<String, String>) -> AppSettings`
  - `pub fn describe(opts: &DownloadOptions) -> String`, `pub fn changed_summary(opts: &DownloadOptions) -> Vec<&'static str>`

- [ ] **Step 1:** `tests/test_options_model.py` 의 8개 경우를 같은 기대값으로 `model.rs` 의 `#[cfg(test)]` 에 옮긴다. 더한다(Review Focus 5):

```rust
#[test]
fn garbage_values_fall_back_to_defaults() {
    let d = HashMap::from([
        ("quality".into(), "9999".into()), ("subs".into(), "maybe".into()),
        ("kind".into(), "".into()),
    ]);
    assert_eq!(options_from_map(&d, None), DownloadOptions::default());
    let s = HashMap::from([("retries".into(), "abc".into()), ("concurrency".into(), "99".into())]);
    assert_eq!(settings_from_map(&s), AppSettings::default());
}
```

- [ ] **Step 2:** `cargo test options::model` → 컴파일 실패 확인.
- [ ] **Step 3:** 구현. bool 은 `true/1/false/0`(대소문자·공백 무시)만 받고, `retries` 0~50, `concurrency` 1~5 밖은 기본값.
- [ ] **Step 4:** `cargo test options::model` → 통과. 커밋.

### Task 3: yt-dlp 인자 만들기

**Files:**
- Create: `src/options/args.rs`
- Reference: `ytdl/options.py:196-356`, `tests/test_options_args.py`

**Interfaces:**
- Consumes: Task 2 의 `DownloadOptions`, `AppSettings`, `NAME_TEMPLATES`
- Produces:
  - `pub struct PlaylistContext { pub title: String, pub index: u32, pub count: u32 }`
  - `pub fn name_template(s: &AppSettings) -> String`
  - `pub fn safe_folder(title: &str) -> String`
  - `pub fn build_args(opts: &DownloadOptions, s: &AppSettings, folder: &str, archive: &str, playlist: Option<&PlaylistContext>) -> Vec<String>`
  - `pub fn split_extra(text: &str) -> Vec<String>`
  - `pub fn full_args(opts, s, folder, archive, playlist) -> Vec<String>`: `["--ignore-config", build_args…, split_extra(s.extra_args)…]`

- [ ] **Step 1:** `tests/test_options_args.py` 의 20개 경우를 옮긴다. Python 이 `to_ydl_opts` 의 dict 를 보던 경우는 인자 목록으로 바꿔 본다. 바뀌는 기대값 하나: 자막을 영상에 넣을 때 인자에 `--embed-subs` 와 `--write-subs` 가 둘 다 있다(자동 자막을 켜도). 더한다(Review Focus 2):

```rust
#[test]
fn percent_in_playlist_title_is_escaped() {
    let o = DownloadOptions { playlist_folder: true, ..Default::default() };
    let ctx = PlaylistContext { title: "100% 모음".into(), index: 1, count: 3 };
    let args = build_args(&o, &AppSettings::default(), "D:\\받기", "a.txt", Some(&ctx));
    assert_eq!(args[3], "100%% 모음/%(title)s.%(ext)s");
}
```

- [ ] **Step 2:** `cargo test options::args` → 실패 확인.
- [ ] **Step 3:** 구현. `safe_folder` 는 yt-dlp `sanitize_filename` 기본 동작 중 Windows 에 필요한 것만: `\ / : * ? " < > |` 와 제어 문자를 `_` 계열로 바꾸는 규칙은 `tests/test_options_args.py` 의 기대값을 따르고, 끝의 점·공백 제거, 빈 이름은 `playlist`, 예약 이름(`CON`, `NUL`, `COM1`…)에는 `_` 를 앞에 붙인다.
- [ ] **Step 4:** 통과 확인, 커밋.

### Task 4: 입력 검사

**Files:**
- Create: `src/options/validate.rs`
- Reference: `ytdl/options.py:358-419`, `tests/test_options_validate.py`

**Interfaces:**
- Consumes: Task 2, Task 3 의 `split_extra`
- Produces:
  - `pub fn parse_duration(text: &str) -> Option<f64>` (`90`, `1:30`, `1:02:03`, 소수 초)
  - `pub fn parse_bytes(text: &str) -> Option<u64>` (`500K`, `5M`, `1.5M`, `2G`, 단위 없으면 바이트, 대소문자 무시)
  - `pub fn playlist_items_ok(text: &str) -> bool` (`1-5,8,10-`, `-3`, `1:10:2` 같은 yt-dlp `-I` 문법)
  - `pub fn validate_options(o: &DownloadOptions) -> BTreeMap<&'static str, String>`
  - `pub type ExtraCheck<'a> = &'a dyn Fn(&str) -> Option<String>` (고급 인자 문자열 → 오류 문구)
  - `pub fn validate_settings(s: &AppSettings, extra: ExtraCheck) -> BTreeMap<&'static str, String>`
  - `pub fn update_settings(s: &AppSettings, cand: AppSettings, changed: &[&str], extra: ExtraCheck) -> (AppSettings, BTreeMap<&'static str, String>)`: 오류 중 `changed` 칸이 원인인 것만 남기고, 있으면 `s` 를 그대로 돌려준다(`ERROR_CAUSES`).
  - `pub fn extra_has_url(text: &str) -> bool`: `split_extra` 조각 중 `://` 가 든 것이 있으면 true.

- [ ] **Step 1:** `tests/test_options_validate.py` 의 14개 경우를 옮긴다. 고급 인자 경우는 `ExtraCheck` 에 가짜 클로저를 넣어 본다. `parse_duration`/`parse_bytes`/`playlist_items_ok` 에 맞는 값·틀린 값 표 시험을 각각 더한다(틀린 값: `"abc"`, `"1:"`, `"5X"`, `"1-,"`, `""`).
- [ ] **Step 2:** 실패 확인 → 구현 → 통과 확인. 오류 문구는 Python 과 글자 그대로. `validate_settings` 는 `extra_has_url` 이면 yt-dlp 를 부르지 않고 `"고급 옵션에는 링크를 넣을 수 없어요"`.
- [ ] **Step 3:** 커밋.

### Task 5: 대기열과 yt-dlp 출력 파싱

**Files:**
- Create: `src/queue.rs`, `src/ytdlp/mod.rs`, `src/ytdlp/parse.rs`
- Reference: `ytdl/job_queue.py`, `ytdl/downloader.py`, `tests/test_job_queue.py`, `tests/test_downloader.py`, Task 1 의 fixtures·findings

**Interfaces:**
- Produces:
  - `pub struct JobQueue<T: Clone + PartialEq>`: `new(limit: usize, start: Box<dyn FnMut(T)>)`, `add(&mut self, job: T)`, `done(&mut self, job: &T)`, `set_limit(&mut self, n: usize)`, `running(&self) -> Vec<T>`, `pending(&self) -> Vec<T>`, `clear_pending(&mut self)`
  - `pub struct PlaylistEntry { pub url: String, pub title: String, pub index: u32 }`
  - `pub enum Probe { Playlist { title: String, site_key: String, webpage_url: String, entries: Vec<PlaylistEntry>, skipped: u32 }, Single { title: String, site_key: String, webpage_url: String, filename: Option<String>, archive_id: String } }`
  - `pub fn parse_probe(json: &str, url: &str, archived: &dyn Fn(&str) -> bool) -> Result<Probe, String>`: `archived` 는 `"<ie_key 소문자> <id>"` 를 받는다.
  - `pub fn archive_ids(text: &str) -> HashSet<String>`
  - `pub fn final_ext(o: &DownloadOptions) -> String` (음원이면 `audio_format`, 영상이면 `container`)
  - `pub fn numbered_template(template: &str, filename: &str, final_ext: &str, exists: &dyn Fn(&Path) -> bool) -> String`: `template` 이 `.%(ext)s` 로 끝나지 않으면 그대로.
  - `pub const PRINT_ARGS: &[&str]`: Task 1 에서 확인한 `--newline`, `--no-warnings`, `--encoding`, `--progress-template`, `--print before_dl`, `--print after_move`, `--no-simulate` 인자.
  - `pub enum Line { Parts(u32), Progress { done: u64, total: Option<u64>, speed: String }, File(String), Other }`, `pub fn parse_line(line: &str) -> Line`
  - `pub struct ProgressMeter`: `new() -> Self`, `on_line(&mut self, line: &Line, now: Instant) -> Option<(f32, String)>`
  - `pub fn friendly_error(stderr: &str) -> String`

- [ ] **Step 1:** `tests/test_job_queue.py`(4), `tests/test_downloader.py`(9)의 경우를 옮긴다. `NumberIfExistsPP` 경우는 `numbered_template` 으로 본다.
- [ ] **Step 2:** fixtures 로 시험을 더한다:
  - `probe_single.txt` → `Probe::Single`, `probe_playlist.txt` → 항목 수·순번, `archived` 가 일부를 true 로 주면 `skipped` 가 그 수.
  - `download_two_streams.txt` 를 줄마다 `parse_line` → `ProgressMeter` 에 넣으면 퍼센트가 줄어드는 일이 없고 마지막이 `(100.0, "파일 정리 중…")`, `Line::File` 경로가 한글 그대로다(Review Focus 1).
  - `download_unknown_size.txt`: `total` 이 `None` 인 줄에서 `on_line` 이 `None` 이고 패닉이 없다(Review Focus 4).
  - `parse_line` 에 `""`, `"WARNING: x"`, `"QMP|1|2"`(칸 부족), `"QMP|a|b|c|d"`, `"QMF|C:\\받기\\제목.mp4\r"` → 앞 넷은 `Line::Other`, 마지막은 `\r` 없는 경로(Review Focus 3).
  - `numbered_template("%(title)s.%(ext)s", "D:\\a\\100%.webm", "mp4", exists)` 에서 `100%.mp4` 가 있으면 결과가 `%(title)s (1).%(ext)s` (Review Focus 2: 줄기는 템플릿이지 파일 이름이 아니다).
  - `friendly_error(error_unavailable.txt 내용)` == `"삭제되었거나 볼 수 없는 영상이에요"`, 표에 없는 오류는 `ERROR: ` 를 뗀 앞 160자 + `" · 업데이트 화면에서 yt-dlp 갱신을 눌러 보세요"`.
- [ ] **Step 3:** 실패 확인 → 구현.
  - `ProgressMeter`: `Line::Parts(n)` 으로 스트림 수를 정한다(없으면 1). 받은 바이트가 앞 줄보다 줄면 다음 스트림. 퍼센트 = `(끝난 스트림 + done/total) / 스트림 수 × 100`, 뒤로 가지 않게. 앞 알림 뒤 0.12초 안이고 5%p 미만이면 `None`. 문구 `받는 중 · <속도>`. 마지막 스트림이 `done >= total` 이면 `(100.0, "파일 정리 중…")` 한 번.
  - `total` 은 `total_bytes`, 없으면 `total_bytes_estimate`. 둘 다 숫자가 아니면 `None`.
- [ ] **Step 4:** 통과 확인, 커밋.

### Task 6: yt-dlp 실행

**Files:**
- Create: `src/ytdlp/run.rs`, `src/ytdlp/process.rs`, `tests/ytdlp_live.rs`
- Reference: `ytdl/downloader.py:118-181`, findings 문서

**Interfaces:**
- Consumes: Task 3 `full_args`, `PlaylistContext`; Task 5 전부
- Produces:
  - `pub struct KillGroup` (Job Object, `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`): `new() -> io::Result<Self>`, `spawn(&self, cmd: &mut Command) -> io::Result<Child>`(`CREATE_NO_WINDOW`, 환경 변수 `PYTHONUTF8=1`·`PYTHONIOENCODING=utf-8`, stdin null, stdout·stderr 파이프). `Send + Sync`.
  - `pub struct Tools { pub ytdlp: PathBuf, pub ffmpeg_dir: Option<PathBuf> }`
  - `pub struct JobSpec { pub url: String, pub opts: DownloadOptions, pub settings: AppSettings, pub folder: String, pub archive: PathBuf, pub playlist: Option<PlaylistContext> }`
  - `pub enum Event { Site { key: String, url: String }, Title(String), Progress(f32, String), Done(String), Skipped, Failed(String), Expanded { title: String, entries: Vec<PlaylistEntry>, skipped: u32 } }`
  - `pub fn run_job(spec: &JobSpec, tools: &Tools, group: &KillGroup, emit: &dyn Fn(Event))`: 끝날 때까지 막는다. 마지막 이벤트는 `Done`/`Skipped`/`Failed`/`Expanded` 중 하나.
  - `pub fn check_extra_args(tools: &Tools, group: &KillGroup, text: &str) -> Option<String>`: findings 7 의 방법. 오류면 `"고급 옵션을 읽지 못했어요: <yt-dlp 오류의 error: 뒤>"`. 10초 넘으면 `None`.
  - `pub fn ytdlp_version(path: &Path, group: &KillGroup) -> Option<String>`

- [ ] **Step 1:** `tests/ytdlp_live.rs` 에 `#[ignore]` 시험(실제 네트워크, `build/tools` 사용, 임시 폴더에 받기):
  - `single_video_downloads`: 이벤트에 `Title`, `Progress` 가 있고 마지막이 `Done(path)`, `path` 가 실제로 있다.
  - `korean_folder_and_title`: 저장 폴더 `…\한글 폴더` 에 한글 제목 영상 → `Done` 경로가 있다(Review Focus 1).
  - `same_name_gets_number`: 같은 링크를 두 번 → 두 번째 `Done` 경로가 ` (1)` 로 끝나는 이름.
  - `playlist_expands`: 마지막이 `Expanded`, 항목 수 > 0.
  - `archived_is_skipped`: `skip_archived` 로 두 번 → 두 번째가 `Skipped`.
  - `bad_url_fails`: `Failed("지원하지 않는 링크예요")`.
  - `extra_args_checked`: `check_extra_args("--no-mtime")` 은 `None`, `"--nope"` 는 `Some`.
- [ ] **Step 2:** `cargo test --test ytdlp_live -- --ignored` → 실패 확인.
- [ ] **Step 3:** 구현. 흐름은 스펙 "다운로드 흐름" 2~6.
  - 조회(`spec.playlist` 가 `None` 일 때만): `full_args` 에서 `--download-archive <경로>` 쌍을 빼고 `-J --flat-playlist` 를 더해 실행 → `parse_probe`(`archived` 는 `skip_archived` 일 때 `archive_ids(archive.txt)` 로). `Site` 를 알리고, 재생목록이면 `Expanded` 로 끝. 단일이면 `Title`; `archive_id` 가 기록에 있으면 `Skipped`.
  - 번호 붙이기: 덮어쓰기가 꺼져 있고 `filename` 이 있으면 `-o` 값을 `numbered_template` 결과로 바꾼다.
  - 받기: `full_args` + `PRINT_ARGS` + `--ffmpeg-location` + 링크. stdout 을 줄마다 `parse_line` → `ProgressMeter`. stderr 는 다른 스레드에서 모은다(파이프가 차서 멈추지 않게).
  - 끝: 종료 코드 0 이고 `Line::File` 이 있었으면 `Done`, 0 인데 없으면 `Skipped`, 아니면 `Failed(friendly_error(stderr))`. findings 4 에서 자막 파일이 남는다고 나왔으면, 자막을 영상에 넣는 작업은 끝난 뒤 `Done` 경로와 줄기가 같은 자막 파일을 지운다.
  - `tools.ytdlp` 가 없으면 `Failed("yt-dlp 를 찾지 못했어요. 앱을 다시 설치해 주세요")`.
- [ ] **Step 4:** `cargo test --test ytdlp_live -- --ignored` → 통과. 커밋.

### Task 7: 설정 저장과 이전 처리

**Files:**
- Create: `src/app_info.rs`, `src/store.rs`, `src/migrate.rs`
- Reference: `ytdl/app_info.py`, `ytdl/migrate.py`, `ytdl/main_window.py:182-213`, `tests/test_migrate.py`, `tests/test_app_info.py`

**Interfaces:**
- Produces:
  - `app_info`: `NAME = "QuickMedia"`, `EXE`, `REPO = "glglekdy/quickmedia"`, `SETTINGS_KEY = r"Software\QuickMedia\QuickMedia"`, `LEGACY_SETTINGS_KEY = r"Software\YoutubeDownloader\YoutubeDownloader"`, `LEGACY_EXE`, `VERSION = env!("CARGO_PKG_VERSION")`, `pub fn data_dir() -> PathBuf`(`%APPDATA%\QuickMedia`), `pub fn user_agent() -> String`(`QuickMedia/<버전>`)
  - `pub trait KeyValue { fn get(&self, key: &str) -> Option<String>; fn set(&mut self, key: &str, value: &str); fn remove(&mut self, key: &str); fn keys(&self) -> Vec<String>; }`: 키는 `folder`, `options/kind` 처럼 `/` 로 하위 키를 나눈다.
  - `pub struct Registry`: `open(path: &str) -> Registry`(HKCU), `impl KeyValue`. `REG_SZ` 는 그대로, `REG_DWORD`/`REG_QWORD` 는 십진 문자열로 읽는다. 다른 형은 없는 것으로. 쓸 때는 `REG_SZ`.
  - `pub struct MemoryStore`(시험용, `impl KeyValue + Default`)
  - `pub fn read_group(kv: &dyn KeyValue, group: &str) -> HashMap<String, String>`, `pub fn write_group(kv: &mut dyn KeyValue, group: &str, values: &BTreeMap<String, String>)`
  - `migrate`: `plan_settings_migration(old: &HashMap<String, String>, new_keys: &HashSet<String>) -> HashMap<String, String>`, `is_legacy_exe(path: &str) -> bool`, `delete_legacy(path: &str, attempts: u32, delay: Duration) -> bool`, `pub fn run(new: &mut dyn KeyValue, old: &dyn KeyValue)`(옛 exe 지우기는 스레드에서, 끝나면 `legacy_exe` 를 지운다. 어떤 실패도 밖으로 내지 않는다)

- [ ] **Step 1:** `tests/test_migrate.py`(8), `tests/test_app_info.py`(1)를 옮긴다. `read_group`/`write_group` 왕복 시험을 `MemoryStore` 로. `Registry` 시험은 `Software\QuickMedia-test-<pid>` 에 `REG_DWORD` 값을 직접 써 넣고 `get` 이 `"3"` 을 주는지, `REG_BINARY` 는 `None` 인지 보고 끝에 키를 지운다(Review Focus 5).
- [ ] **Step 2:** 실패 확인 → 구현 → 통과 → 커밋.

### Task 8: 사이트 이름과 아이콘

**Files:**
- Create: `src/site.rs`, `src/http.rs`
- Reference: `ytdl/site.py`, `tests/test_site.py`

**Interfaces:**
- Produces:
  - `http`: `pub fn agent(proxy: &str) -> ureq::Agent`(User-Agent 는 `app_info::user_agent()`), `pub fn get_bytes(agent: &ureq::Agent, url: &str, accept: &str, max: usize, timeout: Duration) -> Result<Vec<u8>, String>`, `pub fn download(agent: &ureq::Agent, url: &str, part: &Path, progress: &dyn Fn(u64, u64)) -> Result<(u64, String), String>`(받은 크기, SHA-256 소문자 hex. 실패하면 `part` 를 지운다)
  - `site`: `domain_of(url: &str) -> String`, `display_name(extractor_key: &str, host: &str) -> String`, `icon_url(host: &str) -> String`, `letter_of(name: &str) -> String`
  - `pub struct IconPixels { pub width: u32, pub height: u32, pub rgba: Vec<u8> }`
  - `pub struct SiteIcons`: `new(cache_dir: PathBuf) -> Self`, `set_proxy(&self, proxy: &str)`, `load(&self, host: &str) -> Option<IconPixels>`(막는다. 캐시 파일 → 없으면 `/favicon.ico` 를 256KB·5초 한도로 받아 32px 로 줄여 PNG 로 저장). `Send + Sync`.

- [ ] **Step 1:** `tests/test_site.py`(9)를 옮긴다. 받기는 `SiteIcons` 에 `fetch: Box<dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync>` 를 넣을 수 있게 해서(`with_fetch`) 네트워크 없이 본다. 실패한 호스트를 다시 받으려 하는 간격 등은 `ytdl/site.py` 의 규칙 그대로.
- [ ] **Step 2:** 실패 확인 → 구현 → 통과 → 커밋.

### Task 9: 앱 업데이트

**Files:**
- Create: `src/updater.rs`
- Reference: `ytdl/updater.py`, `tests/test_updater.py`, `android/.../update/Releases.kt`(`noteLines`)

**Interfaces:**
- Consumes: Task 7 `app_info`, Task 8 `http`
- Produces:
  - `pub fn parse_version(text: &str) -> Vec<u32>`(끝의 0 은 뗀다), `pub fn is_newer(latest: &str, current: &str) -> bool`
  - `pub struct Release { pub version: String, pub notes: String, pub asset_url: String, pub asset_size: u64, pub digest: String }`
  - `pub fn parse_release(json: &str, current: &str, is_asset: &dyn Fn(&str) -> bool) -> Option<Release>`
  - `pub fn is_setup_name(name: &str) -> bool`(`^QuickMedia-Setup-.+\.exe$`, 대소문자 무시)
  - `pub fn fetch_latest() -> Result<Option<Release>, String>`(404 는 `Ok(None)`. 문구는 Python 과 같다)
  - `pub fn verify(size: u64, sha256_hex: &str, r: &Release) -> Result<(), String>`
  - `pub fn download(r: &Release, progress: &dyn Fn(u64, u64)) -> Result<PathBuf, String>`(`%TEMP%\QuickMedia-update\QuickMedia-Setup-<버전>.exe`)
  - `pub fn can_self_update() -> bool`(`!cfg!(debug_assertions)`)
  - `pub fn apply_command(script: &Path, pid: u32, setup: &Path, exe: &Path) -> Vec<String>`, `pub fn launch_apply(setup: &Path) -> Result<(), String>`(스크립트는 BOM 있는 UTF-8, 내용은 `_APPLY_SCRIPT` 그대로. `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP`, KillGroup 에 넣지 않는다)
  - `pub struct NoteLine { pub text: String, pub heading: bool }`, `pub fn note_lines(markdown: &str) -> Vec<NoteLine>`

- [ ] **Step 1:** `tests/test_updater.py`(6)와 `android/app/src/test/.../update/ReleasesTest.kt` 의 `noteLines`·`verify` 경우를 옮긴다. `parse_release` 에 자산이 없는 릴리즈·`digest` 가 null 인 자산·`tag_name` 이 `V2.0` 인 경우를 더한다.
- [ ] **Step 2:** 실패 확인 → 구현 → 통과 → 커밋.

### Task 10: yt-dlp 갱신

**Files:**
- Create: `src/ytdlp_update.rs`

**Interfaces:**
- Consumes: Task 6 `ytdlp_version`, `KillGroup`; Task 8 `http`; Task 9 `parse_version`, `parse_release`, `verify`, `Release`
- Produces:
  - `pub fn update_file_name(version: &str) -> String`(`yt-dlp-<버전>.exe`), `pub fn version_of_file(name: &str) -> Option<String>`
  - `pub struct Candidate { pub path: PathBuf, pub version: String }`
  - `pub fn choose(bundled: Option<Candidate>, updates: Vec<Candidate>) -> (Option<Candidate>, Vec<PathBuf>)`: 버전이 가장 높은 것(같으면 `bundled`)과 지울 갱신본 목록.
  - `pub fn locate(install_dir: &Path, bin_dir: &Path, kv: &mut dyn KeyValue, group: &KillGroup) -> Option<Candidate>`: 기본 exe 버전은 `ytdlp/bundled_version`·`ytdlp/bundled_stamp`(크기와 수정 시각)에 기억하고 같으면 다시 실행하지 않는다. 낮은 갱신본과 `.part` 를 지운다.
  - `pub fn fetch_latest(current: &str) -> Result<Option<Release>, String>`(`yt-dlp/yt-dlp`, 자산 이름 `yt-dlp.exe`)
  - `pub fn install(r: &Release, bin_dir: &Path, group: &KillGroup, progress: &dyn Fn(u64, u64)) -> Result<Candidate, String>`: `.part` 로 받고 → `verify` → `ytdlp_version` 이 `r.version` 과 같은지 → 이름 바꾸기. 틀리면 지우고 `"받은 yt-dlp 가 실행되지 않아요. 다시 시도해 주세요."`

- [ ] **Step 1:** 시험:

```rust
#[test]
fn newer_update_wins_and_older_updates_are_removed() {
    let b = Candidate { path: "app/yt-dlp.exe".into(), version: "2026.10.01".into() };
    let u = vec![
        Candidate { path: "bin/yt-dlp-2026.09.01.exe".into(), version: "2026.09.01".into() },
        Candidate { path: "bin/yt-dlp-2026.11.05.exe".into(), version: "2026.11.05".into() },
    ];
    let (chosen, remove) = choose(Some(b), u);
    assert_eq!(chosen.unwrap().version, "2026.11.05");
    assert_eq!(remove, vec![PathBuf::from("bin/yt-dlp-2026.09.01.exe")]);
}
#[test]
fn bundled_wins_ties_and_after_app_update() { /* 같은 버전·더 높은 bundled → bundled, 갱신본은 모두 지울 목록 */ }
#[test]
fn nothing_installed_gives_none() { assert!(choose(None, vec![]).0.is_none()); }
#[test]
fn file_name_round_trip() {
    assert_eq!(version_of_file(&update_file_name("2026.11.05")), Some("2026.11.05".into()));
    assert_eq!(version_of_file("yt-dlp.exe"), None);
    assert_eq!(version_of_file("yt-dlp-2026.11.05.exe.part"), None);
}
```

  `tests/ytdlp_live.rs` 에 `#[ignore] fn ytdlp_update_installs`: 빈 임시 `bin` 에 `fetch_latest("0")` → `install` → 돌려준 경로가 있고 `ytdlp_version` 이 같다.
- [ ] **Step 2:** 실패 확인 → 구현 → 통과(무시 시험 포함) → 커밋.

### Task 11: 창 뼈대, 테마, 왼쪽 메뉴

**Files:**
- Create: `build.rs`, `ui/app.slint`, `ui/theme.slint`, `ui/sidebar.slint`, `ui/widgets.slint`, `src/ui/mod.rs`, `assets/*.svg`·`icon.png`(`ytdl/assets/` 에서 복사)
- Modify: `Cargo.toml`(slint, slint-build, rfd), `src/main.rs`
- Reference: `ytdl/theme.py`, `ytdl/sidebar.py`, `tests/test_sidebar.py`, `app.py`

**Interfaces:**
- Produces:
  - Slint: `export component AppWindow inherits Window` (920×640, 최소 690×480, 제목 `QuickMedia`), `export global Theme`(`ytdl/theme.py` 의 LIGHT·DARK 토큰을 같은 이름의 `color` 속성으로, `dark: bool` 에 따라), `export global Nav { in-out property <string> page; in-out property <bool> settings-open; in property <bool> update-dot; callback go(string); }`. 화면 키: `download`, `login`, `network`, `file`, `advanced`, `update`.
  - `widgets.slint`: 뒤 화면들이 쓰는 `Button`(보통·강조), `LineInput`, `Combo`, `Check`, `Radio`, `Bar`(진행률), `ScrollList`. Theme 토큰만 쓴다.
  - Rust: `pub fn nav_after_click(page: &str, open: bool, clicked: &str) -> (String, bool)`: 메뉴를 눌렀을 때의 (화면, 설정 펼침). `pub fn run() -> Result<(), slint::PlatformError>`.
  - `build.rs`: `slint_build::compile("ui/app.slint")`, exe 아이콘(`assets/icon.ico`)과 버전 리소스(CompanyName·FileDescription·ProductName `QuickMedia`, 버전은 Cargo 버전, 언어 0x0412).

- [ ] **Step 1:** `tests/test_sidebar.py` 중 상태에 관한 경우(무엇을 누르면 어느 화면·펼침이 되는지)를 `nav_after_click` 시험으로 옮긴다. 그리기·애니메이션 경우는 옮기지 않는다.
- [ ] **Step 2:** 실패 확인 → 구현. 메뉴: 다운로드 / 설정(펼침 200ms, 화살표 회전) / 하위 4개 / 남는 공간 / 맨 아래 업데이트(빨간 점)와 버전. 항목 높이 34px, 글자 위치 38px. OS 다크 모드는 Slint 의 `Palette.color-scheme` 로 따라가고 바뀌면 바로 반영된다. 각 화면 자리는 빈 `Rectangle`.
- [ ] **Step 3:** `cargo run` 으로 띄워 확인: 창 크기, 메뉴 눌러 화면 키가 바뀜, 설정 펼침, 라이트/다크 전환(Windows 설정을 바꿔 봄), 작업 표시줄 아이콘. 스크린샷을 `build/shots/task11-*.png` 로 남긴다.
- [ ] **Step 4:** `cargo test` 통과, 커밋.

### Task 12: 다운로드 화면

**Files:**
- Create: `ui/download.slint`, `ui/options_panel.slint`, `src/ui/download.rs`, `src/ui/cards.rs`
- Modify: `ui/app.slint`, `src/ui/mod.rs`
- Reference: `ytdl/main_window.py`, `ytdl/options_panel.py`, `tests/test_main_window.py`, `tests/test_option_widgets.py`

**Interfaces:**
- Consumes: Task 2~8 전부, Task 11 의 `widgets.slint`·`Theme`
- Produces:
  - `cards.rs`: `pub enum CardState { Waiting, Loading, Running, Done, Error, Skipped, Cancelled, Playlist }`, `pub struct Card { pub id: u32, pub url: String, pub format: String, pub site: String, pub host: String, pub title: String, pub state: CardState, pub status: String, pub percent: f32, pub path: String }`, `pub fn apply_event(card: &mut Card, ev: &Event)`(문구는 `JobCard` 의 `set_*` 과 같다), `pub fn is_finished(state: CardState) -> bool`, `pub fn playlist_status(added: usize, skipped: u32) -> String`
  - `download.rs`: `pub struct Controller`(UI 스레드 소유): `add_url(&mut self, url: &str) -> Result<(), String>`, `on_event(&mut self, id: u32, ev: Event)`, `clear_finished(&mut self)`, `set_concurrency(&mut self, n: usize)`, `running_count(&self) -> usize`, `shutdown(&mut self)`. 카드 목록은 `slint::VecModel` 로 화면에 준다.
  - `pub const CONFIRM_OVER: usize = 200`
  - Slint `export global Download`: 링크, 프리셋 선택, 요약 문구, 패널 열림, 오류 한 줄, 옵션 필드들(`DownloadOptions` 와 같은 이름을 kebab-case 로), 카드 모델, 저장 폴더, 콜백 `add()`, `paste()`, `pick-folder()`, `preset(string)`, `options-changed()`, `clear-finished()`, `reveal(int)`.

- [ ] **Step 1:** `tests/test_main_window.py` 의 경우 중 화면 없이 볼 수 있는 것을 `cards.rs`·`Controller` 시험으로 옮긴다: 빈 링크, 옵션 오류 시 넣지 않음, 넣은 뒤 패널을 바꿔도 대기 중 작업의 값은 그대로, 재생목록 펼침(순서·자리·제목·`PlaylistContext`), 200개 넘을 때 확인, 건너뛴 개수 문구, 끝난 항목 지우기, 동시 다운로드 수 바꾸기, 닫는 중에는 대기 작업을 시작하지 않음. `Controller` 는 작업을 시작하는 함수를 `Box<dyn FnMut(u32, JobSpec)>` 로 받아 시험에서는 가짜를 넣는다.
- [ ] **Step 2:** 실패 확인 → `cards.rs`, `Controller` 구현 → 통과.
- [ ] **Step 3:** 화면 구현. 구성·문구·켜고 끄는 규칙은 `ytdl/main_window.py:215-435` 와 `ytdl/options_panel.py` 그대로: 링크 칸(Enter 로 넣기)·붙여넣기·다운로드, 프리셋 3개와 "세부 옵션" 버튼(바뀐 항목 요약), 패널 4개 탭(형식·자막·후처리·재생목록)과 고른 값에 따른 칸 숨김, 오류 한 줄, 카드(사이트 아이콘 16px 또는 글자, 사이트 이름, 형식 요약, 제목, 상태, 진행률 막대), 빈 상태 문구, "끝난 항목 지우기", 아래 저장 위치 독(폴더 바꾸기는 `rfd`). 완료 카드를 누르면 `explorer /select,<경로>`.
- [ ] **Step 4:** 잇기: 실제 시작 함수는 스레드를 만들어 `run_job` 을 돌리고 `Event` 를 `slint::invoke_from_event_loop` 로 `Controller::on_event` 에 넘긴다. 스레드 안은 `catch_unwind` 로 싸서 패닉이면 `Failed("알 수 없는 오류가 났어요")`. 아이콘은 `SiteIcons::load` 를 스레드에서 불러 `slint::Image::from_rgba8` 로. 옵션은 넣을 때 레지스트리 `options/*` 에, 폴더는 `folder` 에 저장. 창을 닫으면 `shutdown()`(대기 비우기, `KillGroup` 을 놓아 자식 종료).
- [ ] **Step 5:** `cargo run` 으로 확인하고 스크린샷을 남긴다: 영상 한 건, MP3 한 건, 재생목록, 틀린 링크의 오류 문구, 옵션 오류(WEBM + H.264), 받는 중 창 닫기 뒤 작업 관리자에 `yt-dlp.exe`·`ffmpeg.exe` 가 남지 않음, 한글 입력(IME)으로 링크 칸·구간 칸에 타자.
- [ ] **Step 6:** `cargo test` 통과, 커밋.

### Task 13: 설정 화면 4개

**Files:**
- Create: `ui/settings.slint`, `src/ui/settings.rs`
- Modify: `ui/app.slint`, `src/ui/mod.rs`
- Reference: `ytdl/settings_pages.py`, `tests/test_settings_pages.py`

**Interfaces:**
- Consumes: Task 4 `update_settings`, Task 6 `check_extra_args`, Task 7 store, Task 12 `Controller::set_concurrency`
- Produces:
  - `pub struct SettingsModel`: `new(value: AppSettings) -> Self`, `value(&self) -> &AppSettings`, `update(&mut self, cand: AppSettings, changed: &[&str], extra: ExtraCheck) -> BTreeMap<&'static str, String>`(값이 실제로 바뀌면 true 를 돌려주는 `take_changed(&mut self) -> bool`)
  - `pub fn enabled_rules(s: &AppSettings) -> SettingsEnabled { cookie_browser: bool, cookie_file: bool, custom_template: bool }`
  - Slint `export global Settings`: `AppSettings` 필드(kebab-case), 칸별 오류 문구, 콜백 `changed(string)`(바뀐 칸 이름), `browse-cookie()`, `leave()`.

- [ ] **Step 1:** `tests/test_settings_pages.py`(16) 중 모델·규칙에 관한 경우를 `SettingsModel`·`enabled_rules` 시험으로 옮긴다: 맞는 값은 바로 저장, 틀린 값은 저장 안 하고 오류, 다른 칸의 기존 오류가 저장을 막지 않음, 화면을 떠날 때 틀린 칸은 저장값으로 되돌림.
- [ ] **Step 2:** 실패 확인 → 구현 → 통과.
- [ ] **Step 3:** 화면: 제목 + 설명 + 카드(이름 | 입력, 아래 안내·오류) 틀에 로그인·네트워크·파일·고급. 문구는 `ytdl/settings_pages.py` 그대로. 고급 인자 검사는 입력이 멈춘 뒤 스레드에서 `check_extra_args` 를 돌리고 결과가 왔을 때 칸의 글이 그대로면 반영한다. 바뀌면 레지스트리 `settings/*` 에 쓰고 `concurrency`·`proxy` 를 `Controller`·`SiteIcons` 에 알린다.
- [ ] **Step 4:** `cargo run` 으로 네 화면을 확인(틀린 속도 제한 `5X`, 없는 쿠키 파일, 틀린 고급 인자 `--nope`, 앱을 껐다 켜도 값 유지), 스크린샷, `cargo test`, 커밋.

### Task 14: 업데이트 화면

**Files:**
- Create: `ui/update.slint`, `src/ui/update.rs`
- Modify: `ui/app.slint`, `src/ui/mod.rs`, `src/main.rs`
- Reference: `ytdl/update_page.py`, `tests/test_update_page.py`

**Interfaces:**
- Consumes: Task 9, Task 10, Task 12 `Controller::running_count`
- Produces:
  - `pub enum AppUpdate { Idle, Checking, UpToDate { at: String }, CheckFailed(String), Available { release: Release, phase: Phase } }`, `pub enum Phase { Ready, Downloading(u8), Failed(String), Restarting }`
  - `pub struct UpdateView { pub title: String, pub sub: String, pub sub_error: bool, pub button: String, pub enabled: bool, pub primary: bool, pub status: String, pub status_error: bool, pub bar: Option<u8>, pub notes: Vec<NoteLine> }`
  - `pub fn app_view(state: &AppUpdate, can_self_update: bool) -> UpdateView`
  - `pub enum YtdlpUpdate { Idle, Checking, Downloading(u8), UpToDate, Updated(String), Failed(String) }`, `pub struct YtdlpView { pub version: String, pub status: String, pub status_error: bool, pub enabled: bool }`, `pub fn ytdlp_view(current: Option<&str>, state: &YtdlpUpdate) -> YtdlpView`
  - Slint `export global Update`: 위 두 view 의 필드, 콜백 `app-action()`, `ytdlp-action()`.

- [ ] **Step 1:** `tests/test_update_page.py`(13)의 상태별 문구 경우를 `app_view` 시험으로 옮긴다(디버그 빌드 문구는 `"개발용 빌드라 자동 업데이트는 설치한 앱에서만 돼요."`). `ytdlp_view` 시험:

```rust
assert_eq!(ytdlp_view(Some("2026.10.01"), &YtdlpUpdate::Idle).version, "2026.10.01");
assert_eq!(ytdlp_view(None, &YtdlpUpdate::Idle).version, "찾지 못했어요");
assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::Checking).status, "확인하는 중…");
assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::Downloading(40)).status, "받는 중… 40%");
assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::UpToDate).status, "최신이에요");
assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::Updated("2026.11.05".into())).status, "2026.11.05 로 갱신했어요");
assert!(!ytdlp_view(Some("x"), &YtdlpUpdate::Downloading(1)).enabled);
assert!(ytdlp_view(Some("x"), &YtdlpUpdate::Failed("e".into())).status_error);
```

- [ ] **Step 2:** 실패 확인 → 구현 → 통과.
- [ ] **Step 3:** 화면과 잇기: 앱을 켤 때 스레드에서 `updater::fetch_latest()`, 새 버전이면 `Nav.update-dot`. "지금 업데이트" 는 받는 중인 영상이 있으면 묻고(`_confirm_update` 문구) → `download` → `launch_apply` → 앱 종료. 릴리즈 설명은 `note_lines` 를 최대 220px 높이 안에서 스크롤. 아래 "yt-dlp" 영역: 버전, `갱신` 버튼 → `ytdlp_update::fetch_latest` → `install` → 이후 작업의 `Tools.ytdlp` 를 새 경로로. 맨 아래 작은 글씨 `Made with Slint`. `main.rs` 순서: 이전 처리 → `ytdlp_update::locate` → 창 → 업데이트 확인.
- [ ] **Step 4:** `cargo run` 으로 확인: 최신일 때, GitHub 에 못 붙을 때(네트워크를 끊고), yt-dlp 갱신(`%APPDATA%\QuickMedia\bin` 에 파일이 생기고 버전 표시가 바뀜, 다시 켜도 그 버전). 스크린샷, `cargo test`, 커밋.

### Task 15: 빌드, 설치 파일, 워크플로

**Files:**
- Modify: `build.ps1`, `installer/QuickMedia.iss`, `.github/workflows/release.yml`, `.gitignore`
- Reference: `tests/test_installer.py`

**Interfaces:**
- Consumes: `scripts/fetch-tools.ps1`
- Produces: `./build.ps1 [-FFmpeg <경로>] [-YtDlp <경로>]` → `dist/QuickMedia-Setup-<버전>.exe`

- [ ] **Step 1:** `build.ps1`: 버전을 `Cargo.toml` 에서 읽는다 → `cargo build --release` → `dist\QuickMedia\` 를 비우고 `target\release\QuickMedia.exe`, yt-dlp, ffmpeg 를 복사(인자가 없으면 `scripts/fetch-tools.ps1` 로 받은 것) → ISCC. PyInstaller·`tools/version_info.py` 부분은 지운다.
- [ ] **Step 2:** `QuickMedia.iss`: 머리 주석을 고친다. `[InstallDelete]` 의 `{app}\_internal` 은 남기고 이유 주석을 "1.x(PyInstaller) 에서 올라올 때 옛 파일을 지운다" 로. AppId·이름·`[Run]`·`[Code]` 는 그대로.
- [ ] **Step 3:** `release.yml`: `build` 작업에서 Python 단계를 지우고 `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, "버전 반영"(태그 버전을 `Cargo.toml` 의 `version = "…"` 줄에), `cargo test`, "yt-dlp 받기"(`$env:YTDLP`), `./build.ps1 -FFmpeg $env:FFMPEG -YtDlp $env:YTDLP`. `android` 작업과 머리 주석의 APK 줄을 지운다.
- [ ] **Step 4:** `./build.ps1` 실행 → 설치 파일이 나오는지, 크기를 적는다. 깨끗한 폴더에 설치해 실행·영상 한 건 받기. 1.2.0 설치본이 있으면 그 위에 설치해 `_internal` 이 지워지고 설정이 남는지 본다. `tests/test_installer.py` 가 보던 것(AppId 가 그대로)을 `tests/installer.rs` 로 옮긴다.
- [ ] **Step 5:** 커밋.

### Task 16: Python·안드로이드 지우기, 문서

**Files:**
- Delete: `ytdl/`, `tests/*.py`, `app.py`, `requirements.txt`, `run.bat`, `tools/`, `android/`
- Modify: `.gitignore`, `.gitattributes`, `README.md`
- Create: `release-notes/v2.0.0.md`

- [ ] **Step 1:** 위 파일을 지운다. `.gitignore` 에서 Python·안드로이드 줄을 지우고 `target/` 을 남긴다. `.gitattributes` 에서 `android/gradlew`·`*.so` 줄을 지운다.
- [ ] **Step 2:** `README.md`: 첫 줄을 "Windows 앱" 으로, "안드로이드" 절과 APK·키스토어 문단을 지운다. "개발" 절을 `cargo run` / `cargo test` / `./build.ps1`(Rust 와 Inno Setup 6 필요)로. 기능 목록에 "yt-dlp 를 앱 안에서 갱신" 을 더한다.
- [ ] **Step 3:** `release-notes/v2.0.0.md`(이 저장소 릴리즈 노트의 말투로): 앱을 다시 만들어 더 가볍고 빨리 켜진다 / 업데이트 화면에서 yt-dlp 를 갱신할 수 있다 / 설정과 받은 기록은 그대로 / 안드로이드 앱 지원을 끝냈다(설치한 앱은 계속 쓸 수 있지만 새 버전은 나오지 않는다).
- [ ] **Step 4:** `git grep -n -i "android\|pyside\|pyinstaller\|python" -- . ":!docs" ":!release-notes/v1*"` 로 남은 흔적을 확인하고 정리한다(`_internal` 주석은 남는다).
- [ ] **Step 5:** `cargo test`, `cargo clippy -- -D warnings`, `./build.ps1` 통과. 스펙 "시험" 절의 손으로 확인 목록을 설치본으로 한 번 돈다. 커밋.
