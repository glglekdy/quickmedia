//! yt-dlp 가 찍은 글을 읽는다. 프로세스도 화면도 모른다.

use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::options::DownloadOptions;

/// 받을 때 덧붙이는 인자. 앱이 진행률을 그리고 결과를 알 수 있게 정해진 형식의 줄을 찍게 한다.
///
/// - `QMN|243+251`: 받을 스트림(형식 ID). 받기 직전에 한 번.
/// - `QMP|<형식 ID>|<받은 바이트>|<전체>|<추정 전체>|<속도>`: 진행률. 자막은 형식 ID 가 `NA` 다.
/// - `QMF|<경로>`: 병합·변환을 마친 최종 파일.
pub const PRINT_ARGS: &[&str] = &[
    "--newline",
    "--progress", // --print 는 진행률을 끈다
    "--no-warnings",
    "--encoding",
    "utf-8",
    "--progress-template",
    "download:QMP|%(info.format_id)s|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress._speed_str)s",
    "--print",
    "before_dl:QMN|%(format_id)s",
    "--print",
    "after_move:QMF|%(filepath)s",
    "--no-simulate",
];

/// 정보를 조회할 때 덧붙이는 인자. `QMFN|<받을 파일 이름>` 줄 뒤에 JSON 한 줄이 나온다.
pub const PROBE_ARGS: &[&str] = &[
    "--no-warnings",
    "--encoding",
    "utf-8",
    "--print",
    "QMFN|%(filename)s",
    "-J",
    "--flat-playlist",
];

#[derive(Clone, Debug, PartialEq)]
pub struct PlaylistEntry {
    pub url: String,
    pub title: String,
    pub index: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Probe {
    Playlist { title: String, site_key: String, webpage_url: String, entries: Vec<PlaylistEntry>, skipped: u32 },
    Single { title: String, site_key: String, webpage_url: String, filename: Option<String>, archive_id: String },
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// 조회 결과(stdout 전체)를 읽는다. `archived` 는 받은 기록의 `"<추출기 소문자> <id>"` 를 받는다.
pub fn parse_probe(stdout: &str, url: &str, archived: &dyn Fn(&str) -> bool) -> Result<Probe, String> {
    let json = stdout.lines().rev().find(|l| l.trim_start().starts_with('{')).ok_or("영상 정보 읽기 실패")?;
    let info: Value = serde_json::from_str(json.trim()).map_err(|_| "영상 정보 읽기 실패")?;
    let site_key = text(&info, "extractor_key").to_string();
    let webpage_url = or(text(&info, "webpage_url"), url).to_string();

    if text(&info, "_type") == "playlist" {
        let mut entries = Vec::new();
        let mut skipped = 0;
        let list = info.get("entries").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        for (pos, entry) in list.iter().enumerate() {
            if !entry.is_object() {
                continue;
            }
            let entry_url = or(text(entry, "url"), text(entry, "webpage_url"));
            if entry_url.is_empty() {
                continue;
            }
            let (key, id) = (text(entry, "ie_key"), text(entry, "id"));
            if !key.is_empty() && !id.is_empty() && archived(&format!("{} {id}", key.to_lowercase())) {
                skipped += 1;
                continue;
            }
            // 순번이 없으면 원래 목록에서의 위치를 쓴다
            let index = entry.get("playlist_index").and_then(Value::as_u64).unwrap_or(pos as u64 + 1) as u32;
            entries.push(PlaylistEntry { url: entry_url.into(), title: or(text(entry, "title"), entry_url).into(), index });
        }
        let title = or(or(text(&info, "title"), text(&info, "id")), url).to_string();
        return Ok(Probe::Playlist { title, site_key, webpage_url, entries, skipped });
    }

    let filename = stdout.lines().find_map(|l| l.strip_prefix("QMFN|")).map(|s| s.trim_end().to_string());
    Ok(Probe::Single {
        title: or(text(&info, "title"), url).to_string(),
        archive_id: format!("{} {}", site_key.to_lowercase(), text(&info, "id")),
        site_key,
        webpage_url,
        filename: filename.filter(|f| !f.is_empty()),
    })
}

/// 받은 기록 파일(`archive.txt`)의 줄들.
pub fn archive_ids(text: &str) -> HashSet<String> {
    text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

/// 변환을 마친 파일의 확장자.
pub fn final_ext(o: &DownloadOptions) -> String {
    if o.kind == "audio" { o.audio_format.clone() } else { o.container.clone() }
}

const EXT: &str = ".%(ext)s";

/// 같은 이름의 파일이 있으면 yt-dlp 는 받지 않고 넘어간다. 브라우저처럼 `이름 (1).mp4` 로 새로 받게
/// 출력 템플릿에 번호를 넣는다. `filename` 은 yt-dlp 가 알려 준 받을 이름, `final_ext` 는 변환 뒤 확장자.
pub fn numbered_template(template: &str, filename: &str, final_ext: &str, exists: &dyn Fn(&Path) -> bool) -> String {
    let Some(stem) = template.strip_suffix(EXT) else {
        return template.to_string();
    };
    let name_start = filename.rfind(['\\', '/']).map_or(0, |i| i + 1);
    let Some(dot) = filename[name_start..].rfind('.') else {
        return template.to_string();
    };
    let (base, ext) = (&filename[..name_start + dot], &filename[name_start + dot + 1..]);
    // yt-dlp 가 "이미 받음" 으로 보는 경로와 같게: 받은 그대로와 변환(mp3 등)을 마친 이름
    let taken = |suffix: &str| {
        [ext, final_ext].iter().any(|e| exists(Path::new(&format!("{base}{suffix}.{e}"))))
    };
    if !taken("") {
        return template.to_string();
    }
    let n = (1..10_000).find(|n| !taken(&format!(" ({n})"))).unwrap_or(10_000);
    format!("{stem} ({n}){EXT}")
}

/// yt-dlp 가 찍은 한 줄.
#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    /// 받을 스트림의 형식 ID 들.
    Parts(Vec<String>),
    Progress { format: String, done: u64, total: Option<u64>, speed: String },
    /// 최종 파일 경로.
    File(String),
    Other,
}

pub fn parse_line(line: &str) -> Line {
    let line = line.trim_end_matches(['\r', '\n']);
    if let Some(ids) = line.strip_prefix("QMN|") {
        let parts: Vec<String> = ids.split('+').filter(|s| !s.is_empty() && *s != "NA").map(str::to_string).collect();
        return if ids.is_empty() { Line::Other } else { Line::Parts(parts) };
    }
    if let Some(path) = line.strip_prefix("QMF|") {
        return if path.is_empty() { Line::Other } else { Line::File(path.to_string()) };
    }
    if let Some(rest) = line.strip_prefix("QMP|") {
        let fields: Vec<&str> = rest.splitn(5, '|').collect();
        if let [format, done, total, estimate, speed] = fields[..] {
            if let Ok(done) = done.trim().parse::<u64>() {
                let total = total.trim().parse::<u64>().ok()
                    .or_else(|| estimate.trim().parse::<f64>().ok().filter(|n| *n >= 0.0).map(|n| n as u64));
                let speed = speed.trim();
                let speed = if speed == "NA" { "" } else { speed };
                return Line::Progress { format: format.to_string(), done, total, speed: speed.to_string() };
            }
        }
    }
    Line::Other
}

/// 영상·음성을 따로 받으면 스트림이 여러 개다. 전체를 하나의 진행률로 합친다.
pub struct ProgressMeter {
    parts: Vec<String>,
    last_pct: f32,
    last_emit: Option<Instant>,
    finishing: bool,
}

impl Default for ProgressMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressMeter {
    pub fn new() -> Self {
        Self { parts: Vec::new(), last_pct: 0.0, last_emit: None, finishing: false }
    }

    /// 화면에 알릴 (퍼센트, 상태 문구). 알릴 게 없으면 `None`.
    pub fn on_line(&mut self, line: &Line, now: Instant) -> Option<(f32, String)> {
        match line {
            Line::Parts(parts) => {
                self.parts = parts.clone();
                None
            }
            Line::Progress { format, done, total, speed } => {
                // 자막처럼 스트림 목록에 없는 것은 세지 않는다
                let index = self.parts.iter().position(|p| p == format)?;
                let total = (*total).filter(|t| *t > 0)?;
                if self.finishing {
                    return None;
                }
                let count = self.parts.len();
                if index == count - 1 && *done >= total {
                    self.finishing = true;
                    self.last_pct = 100.0;
                    return Some((100.0, "파일 정리 중…".to_string()));
                }
                let frac = (*done as f64 / total as f64).min(1.0);
                // 크기 추정치가 바뀌어도 뒤로 가지 않게
                let pct = (((index as f64 + frac) / count as f64 * 100.0) as f32).max(self.last_pct);
                // 초당 8번까지만 알린다
                if let Some(last) = self.last_emit {
                    if now.duration_since(last) < Duration::from_millis(120) && pct - self.last_pct < 5.0 {
                        return None;
                    }
                }
                self.last_pct = pct;
                self.last_emit = Some(now);
                let status = if speed.is_empty() { "받는 중".to_string() } else { format!("받는 중 · {speed}") };
                Some((pct, status))
            }
            _ => None,
        }
    }
}

/// 위에서부터 맞춰 본다. 쿠키 실패 메시지에도 "Sign in" 이 섞일 수 있어 쿠키가 먼저다.
const ERRORS: &[(&[&str], &str)] = &[
    (&["unsupported url", "is not a valid url"], "지원하지 않는 링크"),
    (
        &["cookies database", "failed to decrypt", "cookie database"],
        "브라우저 쿠키 읽기 실패. 브라우저를 닫고 다시 시도하거나 cookies.txt를 사용하세요",
    ),
    (
        &["sign in", "login required", "private video", "members-only"],
        "로그인이 필요한 영상. 설정 → 로그인에서 쿠키를 켜세요",
    ),
    (
        &["video unavailable", "video is unavailable", "this video is not available"],
        "삭제되었거나 볼 수 없는 영상",
    ),
    (&["requested format is not available"], "선택한 화질·형식 없음. 세부 옵션을 바꾸세요"),
    (&["getaddrinfo", "timed out", "unable to connect", "proxyerror"], "인터넷 연결이나 프록시 설정을 확인하세요"),
];

const UPDATE_HINT: &str = " · 설정 → 업데이트에서 yt-dlp를 갱신하세요";

/// yt-dlp 의 오류 출력을 화면에 보일 문구로.
pub fn friendly_error(stderr: &str) -> String {
    let errors: Vec<&str> = stderr.lines().map(str::trim).filter(|l| l.starts_with("ERROR:")).collect();
    let all = if errors.is_empty() { stderr.trim().to_string() } else { errors.join("\n") };
    let low = all.to_lowercase();
    if (low.contains("ffmpeg") || low.contains("ffprobe")) && (low.contains("not found") || low.contains("not installed")) {
        return "ffmpeg를 찾을 수 없음".to_string();
    }
    for (needles, message) in ERRORS {
        if needles.iter().any(|n| low.contains(n)) {
            return message.to_string();
        }
    }
    let last = errors.last().copied().unwrap_or_else(|| all.lines().last().unwrap_or(""));
    let short: String = last.replace("ERROR: ", "").chars().take(160).collect();
    if short.trim().is_empty() {
        return format!("알 수 없는 오류{UPDATE_HINT}");
    }
    format!("{short}{UPDATE_HINT}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const SINGLE: &str = include_str!("../../tests/fixtures/probe_single.txt");
    const PLAYLIST: &str = include_str!("../../tests/fixtures/probe_playlist.txt");
    const TWO: &str = include_str!("../../tests/fixtures/download_two_streams.txt");
    const UNKNOWN: &str = include_str!("../../tests/fixtures/download_unknown_size.txt");
    const UNAVAILABLE: &str = include_str!("../../tests/fixtures/error_unavailable.txt");

    fn never(_: &str) -> bool {
        false
    }

    #[test]
    fn friendly_error_table() {
        let cookies = "브라우저 쿠키 읽기 실패. 브라우저를 닫고 다시 시도하거나 cookies.txt를 사용하세요";
        let login = "로그인이 필요한 영상. 설정 → 로그인에서 쿠키를 켜세요";
        let net = "인터넷 연결이나 프록시 설정을 확인하세요";
        let cases = [
            ("ERROR: Unsupported URL: https://x", "지원하지 않는 링크"),
            ("ERROR: could not find chrome cookies database in ...", cookies),
            ("ERROR: Could not copy Chrome cookie database. See ...", cookies),
            ("ERROR: Failed to decrypt with DPAPI", cookies),
            ("ERROR: Sign in to confirm your age", login),
            ("ERROR: Join this channel to get access to members-only content", login),
            ("ERROR: Video unavailable", "삭제되었거나 볼 수 없는 영상"),
            ("ERROR: [youtube] aaaaaaaaaaa: This video is unavailable", "삭제되었거나 볼 수 없는 영상"),
            ("ERROR: Requested format is not available", "선택한 화질·형식 없음. 세부 옵션을 바꾸세요"),
            ("ERROR: Postprocessing: ffprobe and ffmpeg not found", "ffmpeg를 찾을 수 없음"),
            ("ERROR: Unable to connect to proxy", net),
            ("ERROR: [Errno 11001] getaddrinfo failed", net),
        ];
        for (msg, expected) in cases {
            assert_eq!(friendly_error(msg), expected, "{msg}");
        }
        assert_eq!(friendly_error(UNAVAILABLE), "삭제되었거나 볼 수 없는 영상");
        assert!(!friendly_error("ERROR: 'x' is not a valid URL").contains("유튜브"));
    }

    #[test]
    fn unknown_error_is_shortened_and_points_to_ytdlp_update() {
        assert_eq!(friendly_error("ERROR: something odd"), "something odd · 설정 → 업데이트에서 yt-dlp를 갱신하세요");
        // 여러 줄이면 마지막 ERROR 줄을 쓴다
        assert!(friendly_error("WARNING: x\nERROR: first\nERROR: last one\n").starts_with("last one · "));
        let long = format!("ERROR: {}", "가".repeat(400));
        let text = friendly_error(&long);
        assert_eq!(text.chars().take_while(|c| *c == '가').count(), 160);
        // 오류 줄이 없으면 글 전체의 끝을 쓴다. 비어 있어도 문구는 있다.
        assert!(friendly_error("").starts_with("알 수 없는 오류"));
    }

    #[test]
    fn probe_single_video() {
        let Probe::Single { title, site_key, webpage_url, filename, archive_id } = parse_probe(SINGLE, "u", &never).unwrap() else {
            panic!("단일 영상이어야 한다");
        };
        assert_eq!(title, "1분 기초영어회화  | 안녕하세요! hi there #Shorts");
        assert_eq!(site_key, "Youtube");
        assert_eq!(webpage_url, "https://www.youtube.com/watch?v=x8dS6_7Fuxg");
        assert_eq!(filename.as_deref(), Some("D:\\받기\\1분 기초영어회화  ｜ 안녕하세요! hi there #Shorts.mkv"));
        assert_eq!(archive_id, "youtube x8dS6_7Fuxg");
    }

    #[test]
    fn probe_playlist_uses_list_position_and_counts_archived() {
        let Probe::Playlist { title, site_key, webpage_url, entries, skipped } = parse_probe(PLAYLIST, "u", &never).unwrap() else {
            panic!("재생목록이어야 한다");
        };
        assert_eq!((title.as_str(), site_key.as_str()), ("Google Search Stories", "YoutubeTab"));
        assert_eq!(webpage_url, "https://www.youtube.com/playlist?list=PLBCF2DAC6FFB574DE");
        assert_eq!((entries.len(), skipped), (13, 0));
        assert_eq!(entries[0], PlaylistEntry {
            url: "https://www.youtube.com/watch?v=GvgqDSnpRQM".into(),
            title: "Andrew Willis, Skatepark Engineer".into(),
            index: 1,
        });
        assert_eq!(entries[12].index, 13);

        let archived = |id: &str| id == "youtube GvgqDSnpRQM" || id == "youtube V4DDt30Aat4";
        let Probe::Playlist { entries, skipped, .. } = parse_probe(PLAYLIST, "u", &archived).unwrap() else { panic!() };
        assert_eq!((entries.len(), skipped), (11, 2));
        assert_eq!(entries[0].index, 2); // 순번은 원래 목록에서의 위치
    }

    #[test]
    fn probe_playlist_entries_edge_cases() {
        let json = r#"{"_type":"playlist","id":"pl","entries":[
            {"url":"https://y/1","title":"one","playlist_index":4,"id":"1","ie_key":"Youtube"},
            {"title":"no url"},
            null,
            {"webpage_url":"https://y/3","id":"3"},
            {"url":"https://y/4","title":"done","id":"4","ie_key":"Youtube"}]}"#
            .replace('\n', " ");
        let Probe::Playlist { title, webpage_url, entries, skipped, .. } =
            parse_probe(&json, "https://y/list", &|id| id == "youtube 4").unwrap() else { panic!() };
        assert_eq!(entries, [
            PlaylistEntry { url: "https://y/1".into(), title: "one".into(), index: 4 },
            PlaylistEntry { url: "https://y/3".into(), title: "https://y/3".into(), index: 4 },
        ]);
        assert_eq!(skipped, 1);
        assert_eq!((title.as_str(), webpage_url.as_str()), ("pl", "https://y/list"));
        let Probe::Playlist { entries, skipped, .. } = parse_probe(r#"{"_type":"playlist"}"#, "u", &never).unwrap() else { panic!() };
        assert!(entries.is_empty() && skipped == 0);
    }

    #[test]
    fn probe_without_json_is_an_error() {
        assert!(parse_probe("QMFN|a.mp4\n", "u", &never).is_err());
        assert!(parse_probe("", "u", &never).is_err());
        assert!(parse_probe("{not json", "u", &never).is_err());
    }

    #[test]
    fn archive_ids_reads_lines() {
        let ids = archive_ids("youtube abc\r\n\n  youtube def  \nvimeo 1\n");
        assert_eq!(ids.len(), 3);
        assert!(ids.contains("youtube abc") && ids.contains("youtube def") && ids.contains("vimeo 1"));
    }

    #[test]
    fn final_ext_by_kind() {
        let mut o = DownloadOptions { container: "mkv".into(), ..Default::default() };
        assert_eq!(final_ext(&o), "mkv");
        o.kind = "audio".into();
        o.audio_format = "flac".into();
        assert_eq!(final_ext(&o), "flac");
    }

    fn numbered(template: &str, filename: &str, ext: &str, existing: &[&str]) -> String {
        let existing: Vec<String> = existing.iter().map(|s| s.to_string()).collect();
        numbered_template(template, filename, ext, &|p| existing.iter().any(|e| Path::new(e) == p))
    }

    #[test]
    fn numbering_keeps_name_when_free() {
        assert_eq!(numbered("%(title)s.%(ext)s", "D:\\v\\Me.mp4", "mp4", &[]), "%(title)s.%(ext)s");
    }

    #[test]
    fn numbering_adds_next_free_number() {
        assert_eq!(numbered("%(title)s.%(ext)s", "D:\\v\\Me.mp4", "mp4", &["D:\\v\\Me.mp4"]), "%(title)s (1).%(ext)s");
        assert_eq!(numbered("%(title)s.%(ext)s", "D:\\v\\Me.mp4", "mp4", &["D:\\v\\Me.mp4", "D:\\v\\Me (1).mp4"]),
                   "%(title)s (2).%(ext)s");
    }

    #[test]
    fn numbering_checks_converted_file() {
        // 음성은 m4a 로 받아 mp3 로 바꾼다. 남아 있는 건 mp3 다.
        assert_eq!(numbered("%(title)s.%(ext)s", "D:\\v\\Me.m4a", "mp3", &["D:\\v\\Me.mp3"]), "%(title)s (1).%(ext)s");
        // 번호를 붙인 이름도 두 확장자를 모두 본다
        assert_eq!(numbered("%(title)s.%(ext)s", "D:\\v\\Me.m4a", "mp3", &["D:\\v\\Me.mp3", "D:\\v\\Me (1).m4a"]),
                   "%(title)s (2).%(ext)s");
    }

    #[test]
    fn numbering_stem_is_the_template_not_the_file_name() {
        assert_eq!(numbered("%(title)s.%(ext)s", "D:\\a\\100%.webm", "mp4", &["D:\\a\\100%.mp4"]), "%(title)s (1).%(ext)s");
        assert_eq!(numbered("목록/01 - %(title)s [%(id)s].%(ext)s", "D:\\a\\목록\\01 - x [i].mp4", "mp4", &["D:\\a\\목록\\01 - x [i].mp4"]),
                   "목록/01 - %(title)s [%(id)s] (1).%(ext)s");
    }

    #[test]
    fn numbering_leaves_other_templates_alone() {
        assert_eq!(numbered("%(title)s", "D:\\v\\Me", "mp4", &["D:\\v\\Me"]), "%(title)s");
        assert_eq!(numbered("%(id)s.%(ext)s.bak", "D:\\v\\x.mp4.bak", "mp4", &["D:\\v\\x.mp4.bak"]), "%(id)s.%(ext)s.bak");
    }

    #[test]
    fn parse_line_kinds() {
        assert_eq!(parse_line("QMN|243+251"), Line::Parts(vec!["243".into(), "251".into()]));
        assert_eq!(parse_line("QMN|18"), Line::Parts(vec!["18".into()]));
        assert_eq!(parse_line("QMP|243|1024|251356|NA|  11.90KiB/s"),
                   Line::Progress { format: "243".into(), done: 1024, total: Some(251356), speed: "11.90KiB/s".into() });
        assert_eq!(parse_line("QMP|18|1024|NA|2048.5|1MiB/s"),
                   Line::Progress { format: "18".into(), done: 1024, total: Some(2048), speed: "1MiB/s".into() });
        assert_eq!(parse_line("QMP|18|1024|NA|NA|NA"),
                   Line::Progress { format: "18".into(), done: 1024, total: None, speed: String::new() });
    }

    #[test]
    fn parse_line_ignores_what_it_does_not_know() {
        for line in ["", "WARNING: x", "QMP|1|2", "QMP|a|b|c|d|e", "QMP|18|x|10|NA|s", "QMX|1", "[download] 50%", "QMN|", "QMF|"] {
            assert_eq!(parse_line(line), Line::Other, "{line:?}");
        }
    }

    #[test]
    fn parse_line_strips_line_ends_and_keeps_korean_paths() {
        assert_eq!(parse_line("QMF|C:\\받기\\제목 | 둘.mp4\r\n"), Line::File("C:\\받기\\제목 | 둘.mp4".into()));
        assert_eq!(parse_line("QMN|243+251\r"), Line::Parts(vec!["243".into(), "251".into()]));
    }

    /// 줄마다 0.2초씩 지난 것으로 보고 알림을 모은다.
    fn replay(text: &str) -> (Vec<(f32, String)>, Option<String>) {
        let mut meter = ProgressMeter::new();
        let mut now = Instant::now();
        let mut out = Vec::new();
        let mut file = None;
        for raw in text.lines() {
            now += Duration::from_millis(200);
            let line = parse_line(raw);
            if let Line::File(path) = &line {
                file = Some(path.clone());
            }
            if let Some(update) = meter.on_line(&line, now) {
                out.push(update);
            }
        }
        (out, file)
    }

    #[test]
    fn two_streams_make_one_rising_progress() {
        let (updates, file) = replay(TWO);
        assert!(updates.len() > 5);
        assert!(updates.windows(2).all(|w| w[0].0 <= w[1].0), "{updates:?}");
        assert_eq!(updates.last().unwrap(), &(100.0, "파일 정리 중…".to_string()));
        assert_eq!(updates.iter().filter(|u| u.1 == "파일 정리 중…").count(), 1);
        assert!(updates[0].1.starts_with("받는 중 · ") && updates[0].1.ends_with("B/s"));
        // 첫 스트림만으로는 절반을 넘지 않는다
        let first_half = updates.iter().take_while(|u| u.0 <= 50.0).count();
        assert!(first_half >= 3, "{updates:?}");
        assert_eq!(file.as_deref(),
                   Some("C:\\Users\\kimji\\orca\\quickmedia\\build\\spike\\한글 폴더\\1분 기초영어회화  ｜ 안녕하세요! hi there #Shorts.mp4"));
    }

    #[test]
    fn unknown_size_and_subtitle_lines_are_skipped() {
        let (updates, file) = replay(UNKNOWN);
        assert!(updates.is_empty(), "{updates:?}");
        assert_eq!(file.as_deref(), Some("D:\\받기\\라이브.mp4"));
    }

    #[test]
    fn progress_is_throttled_and_never_goes_back() {
        let mut m = ProgressMeter::new();
        let t = Instant::now();
        let line = |done: u64, total: u64| Line::Progress { format: "18".into(), done, total: Some(total), speed: "1MiB/s".into() };
        assert_eq!(m.on_line(&Line::Parts(vec!["18".into()]), t), None);
        assert_eq!(m.on_line(&line(10, 100), t), Some((10.0, "받는 중 · 1MiB/s".to_string())));
        // 0.12초 안이고 5%p 미만이면 알리지 않는다
        assert_eq!(m.on_line(&line(12, 100), t + Duration::from_millis(50)), None);
        // 5%p 이상이면 바로 알린다
        assert_eq!(m.on_line(&line(20, 100), t + Duration::from_millis(60)).unwrap().0, 20.0);
        // 크기 추정치가 커져도 뒤로 가지 않는다
        assert_eq!(m.on_line(&line(20, 400), t + Duration::from_secs(1)).unwrap().0, 20.0);
        // 받은 양이 전체를 넘어도 100 을 넘지 않는다
        assert_eq!(m.on_line(&line(500, 400), t + Duration::from_secs(2)), Some((100.0, "파일 정리 중…".to_string())));
        assert_eq!(m.on_line(&line(500, 400), t + Duration::from_secs(3)), None);
    }

    #[test]
    fn progress_without_parts_line_treats_it_as_one_stream() {
        let mut m = ProgressMeter::new();
        let line = Line::Progress { format: "NA".into(), done: 50, total: Some(100), speed: String::new() };
        assert_eq!(m.on_line(&line, Instant::now()), None);
        let mut m = ProgressMeter::new();
        m.on_line(&Line::Parts(vec![]), Instant::now());
        assert_eq!(m.on_line(&line, Instant::now()), None);
    }
}
