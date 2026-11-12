//! 고른 값을 yt-dlp 명령줄 인자로 바꾼다.

use super::model::{AppSettings, DownloadOptions, NAME_TEMPLATES};

/// 재생목록에서 펼친 항목 하나. 항목 작업은 영상 하나만 받으므로 재생목록 값을 직접 넣는다.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaylistContext {
    pub title: String,
    pub index: u32,
    pub count: u32,
}

const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// 파일 이름 규칙(yt-dlp 출력 템플릿).
pub fn name_template(s: &AppSettings) -> String {
    if s.name_template == "custom" {
        return s.custom_template.clone();
    }
    let found = NAME_TEMPLATES.iter().find(|(key, _)| *key == s.name_template);
    found.unwrap_or(&NAME_TEMPLATES[0]).1.to_string()
}

/// yt-dlp `sanitize_filename` 처럼 Windows 가 못 쓰는 글자를 닮은 전각 글자로 바꾼다.
fn sanitize(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    for ch in title.chars() {
        match ch {
            '\n' => out.push(' '),
            c if (c as u32) < 32 || c as u32 == 127 => {}
            '/' => out.push('⧸'),
            '\\' => out.push('⧹'),
            ':' => out.push('：'),
            '?' => out.push('？'),
            '"' => out.push('＂'),
            '*' => out.push('＊'),
            '<' => out.push('＜'),
            '>' => out.push('＞'),
            '|' => out.push('｜'),
            c => out.push(c),
        }
    }
    out
}

/// 재생목록 제목으로 만든 폴더 이름. 제목은 남이 정한 값이라 저장 폴더 밖을 가리키면 안 된다.
pub fn safe_folder(title: &str) -> String {
    // Windows 는 끝의 점·공백을 버리므로 ".." 같은 이름은 여기서 비게 된다
    let cleaned = sanitize(title);
    let folder = cleaned.trim().trim_end_matches([' ', '.']);
    if folder.is_empty() {
        return "playlist".into();
    }
    let stem = folder.split('.').next().unwrap_or("").to_uppercase();
    if RESERVED.contains(&stem.as_str()) {
        return format!("_{folder}");
    }
    folder.to_string()
}

fn playlist_prefix(opts: &DownloadOptions, playlist: &PlaylistContext) -> String {
    let mut prefix = String::new();
    if opts.playlist_folder {
        prefix.push_str(&safe_folder(&playlist.title).replace('%', "%%"));
        prefix.push('/');
    }
    if opts.number_prefix {
        let width = playlist.count.to_string().len().max(2);
        prefix.push_str(&format!("{:0width$} - ", playlist.index));
    }
    prefix
}

fn embed_subs(opts: &DownloadOptions) -> bool {
    opts.subs && opts.sub_mode == "embed" && opts.kind != "audio"
}

fn push(args: &mut Vec<String>, items: &[&str]) {
    args.extend(items.iter().map(|s| s.to_string()));
}

/// 고른 값을 yt-dlp 명령줄 인자로.
pub fn build_args(
    opts: &DownloadOptions,
    s: &AppSettings,
    folder: &str,
    archive: &str,
    playlist: Option<&PlaylistContext>,
) -> Vec<String> {
    let mut template = name_template(s);
    if let Some(ctx) = playlist {
        template = playlist_prefix(opts, ctx) + &template;
    }
    let mut args = Vec::new();
    push(&mut args, &["-P", folder, "-o", &template]);

    match s.cookies.as_str() {
        "browser" => push(&mut args, &["--cookies-from-browser", &s.cookie_browser]),
        "file" => push(&mut args, &["--cookies", &s.cookie_file]),
        _ => {}
    }
    if !s.proxy.is_empty() {
        push(&mut args, &["--proxy", &s.proxy]);
    }
    if !s.rate_limit.is_empty() {
        push(&mut args, &["-r", &s.rate_limit]);
    }
    push(&mut args, &["-R", &s.retries.to_string()]);
    if s.overwrite {
        push(&mut args, &["--force-overwrites"]);
    }

    if playlist.is_some() || opts.playlist == "single" {
        push(&mut args, &["--no-playlist"]);
    } else {
        push(&mut args, &["--yes-playlist"]);
        if !opts.playlist_items.is_empty() {
            push(&mut args, &["-I", &opts.playlist_items.replace(' ', "")]);
        }
    }
    if opts.skip_archived {
        push(&mut args, &["--download-archive", archive]);
    }

    if opts.kind == "audio" {
        let quality = if opts.audio_quality == "best" { "0".to_string() } else { format!("{}K", opts.audio_quality) };
        push(&mut args, &["-x", "--audio-format", &opts.audio_format, "--audio-quality", &quality]);
    } else {
        let mut sort: Vec<String> = Vec::new();
        if opts.quality != "best" {
            sort.push(format!("res:{}", opts.quality));
        }
        match opts.codec.as_str() {
            "h264" => sort.push("vcodec:h264".into()),
            "vp9" => sort.push("vcodec:vp9".into()),
            "av1" => sort.push("vcodec:av01".into()),
            _ => {}
        }
        match opts.container.as_str() {
            "mp4" => sort.push("ext:mp4:m4a".into()),
            "webm" => sort.push("ext:webm:webm".into()),
            _ => {}
        }
        push(&mut args, &["-f", "bv*+ba/b"]);
        if !sort.is_empty() {
            push(&mut args, &["-S", &sort.join(",")]);
        }
        push(&mut args, &["--merge-output-format", &opts.container]);
    }

    if opts.subs {
        if opts.auto_subs {
            push(&mut args, &["--write-auto-subs"]);
        }
        let langs = opts.sub_langs.replace(' ', "");
        push(&mut args, &["--sub-langs", if langs == "all" { "all,-live_chat" } else { &langs }]);
        // 음원 파일에는 자막을 넣을 수 없어 파일로 남긴다
        if embed_subs(opts) {
            // --write-auto-subs 만 있으면 yt-dlp 가 일반 자막을 끄므로 --write-subs 를 함께 준다.
            // no-keep-subs: 넣은 뒤 자막 파일을 남기지 않는다.
            push(&mut args, &["--embed-subs", "--write-subs", "--compat-options", "no-keep-subs"]);
        } else {
            push(&mut args, &["--write-subs", "--convert-subs", &opts.sub_format]);
        }
    }

    if opts.embed_thumbnail {
        push(&mut args, &["--embed-thumbnail"]);
    }
    if opts.save_thumbnail {
        push(&mut args, &["--write-thumbnail", "--convert-thumbnails", "jpg"]);
    }
    if opts.embed_metadata {
        push(&mut args, &["--embed-metadata"]);
    }
    if opts.embed_chapters {
        push(&mut args, &["--embed-chapters"]);
    }
    if opts.split_chapters {
        push(&mut args, &["--split-chapters"]);
    }
    if !opts.section_start.is_empty() || !opts.section_end.is_empty() {
        let start = if opts.section_start.is_empty() { "0" } else { &opts.section_start };
        let end = if opts.section_end.is_empty() { "inf" } else { &opts.section_end };
        push(&mut args, &["--download-sections", &format!("*{start}-{end}"), "--force-keyframes-at-cuts"]);
    }
    args
}

/// 고급 옵션 칸을 인자로 나눈다. 따옴표로 묶을 수 있고, Windows 경로의 백슬래시는 그대로 둔다.
pub fn split_extra(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;
    for ch in text.chars() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                current.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
            started = true;
        } else if ch.is_whitespace() {
            if started {
                parts.push(std::mem::take(&mut current));
                started = false;
            }
        } else {
            current.push(ch);
            started = true;
        }
    }
    if started {
        parts.push(current);
    }
    parts
}

/// yt-dlp 에 넘길 인자 전체. 고급 옵션은 맨 뒤에 붙여 같은 옵션이면 사용자 값이 이긴다.
pub fn full_args(
    opts: &DownloadOptions,
    s: &AppSettings,
    folder: &str,
    archive: &str,
    playlist: Option<&PlaylistContext>,
) -> Vec<String> {
    let mut args = vec!["--ignore-config".to_string()];
    args.extend(build_args(opts, s, folder, archive, playlist));
    args.extend(split_extra(&s.extra_args));
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(f: impl FnOnce(&mut DownloadOptions)) -> DownloadOptions {
        let mut o = DownloadOptions::default();
        f(&mut o);
        o
    }

    fn args(o: &DownloadOptions) -> Vec<String> {
        build_args(o, &AppSettings::default(), "D:/v", "A:/archive.txt", None)
    }

    fn args_s(s: &AppSettings) -> Vec<String> {
        build_args(&DownloadOptions::default(), s, "D:/v", "A:/archive.txt", None)
    }

    fn args_p(o: &DownloadOptions, title: &str, index: u32, count: u32) -> Vec<String> {
        let ctx = PlaylistContext { title: title.into(), index, count };
        build_args(o, &AppSettings::default(), "D:/v", "A:/archive.txt", Some(&ctx))
    }

    /// `flag` 바로 뒤의 값.
    fn after<'a>(a: &'a [String], flag: &str) -> &'a str {
        let i = a.iter().position(|x| x == flag).unwrap_or_else(|| panic!("{flag} 없음: {a:?}"));
        &a[i + 1]
    }

    fn has(a: &[String], flag: &str) -> bool {
        a.iter().any(|x| x == flag)
    }

    #[test]
    fn default_args_exact() {
        assert_eq!(args(&DownloadOptions::default()), [
            "-P", "D:/v", "-o", "%(title)s.%(ext)s", "-R", "10", "--no-playlist",
            "-f", "bv*+ba/b", "-S", "ext:mp4:m4a", "--merge-output-format", "mp4",
        ]);
    }

    #[test]
    fn video_sort() {
        let cases = [
            (opts(|o| o.quality = "720".into()), "res:720,ext:mp4:m4a", "mp4"),
            (opts(|o| { o.quality = "1080".into(); o.codec = "h264".into() }), "res:1080,vcodec:h264,ext:mp4:m4a", "mp4"),
            (opts(|o| { o.container = "webm".into(); o.codec = "vp9".into() }), "vcodec:vp9,ext:webm:webm", "webm"),
            (opts(|o| { o.container = "mkv".into(); o.codec = "av1".into() }), "vcodec:av01", "mkv"),
        ];
        for (o, sort, merge) in cases {
            let a = args(&o);
            assert_eq!(after(&a, "-S"), sort);
            assert_eq!(after(&a, "--merge-output-format"), merge);
        }
    }

    #[test]
    fn mkv_auto_has_no_sort() {
        assert!(!has(&args(&opts(|o| o.container = "mkv".into())), "-S"));
    }

    #[test]
    fn audio() {
        let a = args(&opts(|o| { o.kind = "audio".into(); o.audio_format = "opus".into(); o.audio_quality = "best".into() }));
        assert!(!has(&a, "-f") && !has(&a, "--merge-output-format"));
        let i = a.iter().position(|x| x == "-x").unwrap();
        assert_eq!(a[i..i + 5], ["-x", "--audio-format", "opus", "--audio-quality", "0"]);
        let a = args(&opts(|o| { o.kind = "audio".into(); o.audio_quality = "320".into() }));
        assert_eq!(after(&a, "--audio-quality"), "320K");
    }

    #[test]
    fn subtitles() {
        let a = args(&opts(|o| { o.subs = true; o.sub_langs = "all".into(); o.auto_subs = true }));
        assert_eq!(after(&a, "--sub-langs"), "all,-live_chat");
        assert!(has(&a, "--write-auto-subs") && has(&a, "--embed-subs"));
        let a = args(&opts(|o| { o.subs = true; o.sub_mode = "file".into(); o.sub_format = "ass".into() }));
        assert_eq!(after(&a, "--convert-subs"), "ass");
        assert!(has(&a, "--write-subs") && !has(&a, "--embed-subs"));
        // 음원에는 넣을 수 없어 파일로
        let a = args(&opts(|o| { o.kind = "audio".into(); o.subs = true }));
        assert!(!has(&a, "--embed-subs"));
        assert_eq!(after(&a, "--convert-subs"), "srt");
        assert!(!has(&args(&opts(|o| o.sub_langs = "en".into())), "--write-subs"));
    }

    #[test]
    fn embedded_subs_fetch_manual_subs_and_leave_no_files() {
        // --write-auto-subs 만 있으면 yt-dlp 가 일반 자막을 끄므로 --write-subs 를 함께 준다.
        // 그러면 넣은 뒤에도 자막 파일이 남는데, no-keep-subs 가 그것을 지우게 한다.
        for auto in [false, true] {
            let a = args(&opts(|o| { o.subs = true; o.auto_subs = auto }));
            assert!(has(&a, "--embed-subs") && has(&a, "--write-subs"), "{a:?}");
            assert_eq!(after(&a, "--compat-options"), "no-keep-subs");
            assert!(!has(&a, "--convert-subs"));
        }
        let a = args(&opts(|o| { o.subs = true; o.sub_mode = "file".into() }));
        assert!(!has(&a, "--compat-options") && !has(&a, "--write-auto-subs"));
    }

    #[test]
    fn postprocessing_flags() {
        let a = args(&opts(|o| {
            o.embed_thumbnail = true; o.save_thumbnail = true; o.embed_metadata = true;
            o.embed_chapters = true; o.split_chapters = true;
        }));
        for f in ["--embed-thumbnail", "--write-thumbnail", "--embed-metadata", "--embed-chapters", "--split-chapters"] {
            assert!(has(&a, f), "{f}");
        }
        assert_eq!(after(&a, "--convert-thumbnails"), "jpg");
    }

    #[test]
    fn sections() {
        for (start, end, spec) in [("1:30", "3:00", "*1:30-3:00"), ("", "3:00", "*0-3:00"), ("1:30", "", "*1:30-inf")] {
            let a = args(&opts(|o| { o.section_start = start.into(); o.section_end = end.into() }));
            assert_eq!(after(&a, "--download-sections"), spec);
            assert!(has(&a, "--force-keyframes-at-cuts"));
        }
    }

    #[test]
    fn playlist_and_archive() {
        let a = args(&opts(|o| { o.playlist = "all".into(); o.playlist_items = "1-10,15".into(); o.skip_archived = true }));
        assert!(has(&a, "--yes-playlist") && !has(&a, "--no-playlist"));
        assert_eq!(after(&a, "-I"), "1-10,15");
        assert_eq!(after(&a, "--download-archive"), "A:/archive.txt");
        assert!(!has(&args(&opts(|o| { o.playlist = "single".into(); o.playlist_items = "1-3".into() })), "-I"));
    }

    #[test]
    fn network_and_files() {
        let s = AppSettings {
            cookies: "browser".into(), cookie_browser: "edge".into(), proxy: "http://p:1".into(),
            rate_limit: "5M".into(), retries: 3, name_template: "date_title".into(), overwrite: true,
            ..Default::default()
        };
        let a = args_s(&s);
        assert_eq!(after(&a, "--cookies-from-browser"), "edge");
        assert_eq!(after(&a, "--proxy"), "http://p:1");
        assert_eq!(after(&a, "-r"), "5M");
        assert_eq!(after(&a, "-R"), "3");
        assert!(has(&a, "--force-overwrites"));
        assert_eq!(after(&a, "-o"), "%(upload_date>%Y-%m-%d)s %(title)s.%(ext)s");
        let s = AppSettings {
            cookies: "file".into(), cookie_file: "C:/c.txt".into(), name_template: "custom".into(),
            custom_template: "%(id)s.%(ext)s".into(), ..Default::default()
        };
        let a = args_s(&s);
        assert_eq!(after(&a, "--cookies"), "C:/c.txt");
        assert_eq!(after(&a, "-o"), "%(id)s.%(ext)s");
    }

    #[test]
    fn playlist_prefix_sanitizes_title() {
        let o = opts(|o| { o.playlist = "all".into(); o.number_prefix = true; o.playlist_folder = true });
        let a = args_p(&o, "100% 노래/모음: 2?", 7, 120);
        let (folder, name) = after(&a, "-o").split_once('/').unwrap();
        assert!(!folder.contains("%(") && folder.contains("%%"));
        assert!(!folder.contains('/') && !folder.contains(':') && !folder.contains('?'));
        assert_eq!(name, "007 - %(title)s.%(ext)s");
        assert!(has(&a, "--no-playlist") && !has(&a, "--yes-playlist") && !has(&a, "-I"));
    }

    #[test]
    fn percent_in_playlist_title_is_escaped() {
        let o = opts(|o| o.playlist_folder = true);
        let ctx = PlaylistContext { title: "100% 모음".into(), index: 1, count: 3 };
        let a = build_args(&o, &AppSettings::default(), "D:\\받기", "a.txt", Some(&ctx));
        assert_eq!(a[3], "100%% 모음/%(title)s.%(ext)s");
    }

    #[test]
    fn playlist_prefix_min_width_and_off() {
        let a = args_p(&opts(|o| o.number_prefix = true), "p", 3, 5);
        assert_eq!(after(&a, "-o"), "03 - %(title)s.%(ext)s");
        let a = args_p(&DownloadOptions::default(), "p", 3, 5);
        assert_eq!(after(&a, "-o"), "%(title)s.%(ext)s");
    }

    #[test]
    fn split_extra_keeps_windows_paths() {
        assert_eq!(
            split_extra("--limit-rate 1M  -o \"C:\\내 폴더\\%(id)s.%(ext)s\" --x 'a b'"),
            ["--limit-rate", "1M", "-o", "C:\\내 폴더\\%(id)s.%(ext)s", "--x", "a b"]
        );
        assert!(split_extra("   ").is_empty());
        assert_eq!(split_extra("--a \"\""), ["--a", ""]);
    }

    #[test]
    fn full_args_puts_extra_last_so_it_wins() {
        let s = AppSettings { rate_limit: "5M".into(), extra_args: "--limit-rate 1M --no-mtime".into(), ..Default::default() };
        let a = full_args(&DownloadOptions::default(), &s, "D:/v", "A", None);
        assert_eq!(a[0], "--ignore-config");
        assert_eq!(a[a.len() - 3..], ["--limit-rate", "1M", "--no-mtime"]);
        assert!(a.iter().position(|x| x == "-r").unwrap() < a.len() - 3);
    }

    #[test]
    fn playlist_items_spaces_removed() {
        // 검사는 "1-10, 15" 를 받아 주므로 yt-dlp 에 넘길 때도 공백을 빼야 한다
        let a = args(&opts(|o| { o.playlist = "all".into(); o.playlist_items = "1-10, 15".into() }));
        assert_eq!(after(&a, "-I"), "1-10,15");
    }

    #[test]
    fn playlist_folder_stays_inside_download_folder() {
        // 재생목록 제목은 남이 정한 값이다. ".." 이면 저장 폴더 밖에 쓰게 된다.
        let cases = [("..", "playlist"), (".", "playlist"), ("", "playlist"), ("  ...목록.  ", "...목록"),
                     ("CON", "_CON"), ("nul.txt", "_nul.txt"), ("a\\b|c\"d<e>f*g", "a⧹b｜c＂d＜e＞f＊g"),
                     ("줄\n바꿈\u{1}", "줄 바꿈")];
        for (title, folder) in cases {
            let a = args_p(&opts(|o| o.playlist_folder = true), title, 1, 3);
            assert_eq!(after(&a, "-o"), format!("{folder}/%(title)s.%(ext)s"), "{title:?}");
        }
    }

    #[test]
    fn sub_langs_spaces_removed() {
        // "ko, en" 처럼 쓰면 yt-dlp 가 " en" 을 못 알아봐 영어 자막이 조용히 빠진다
        let a = args(&opts(|o| { o.subs = true; o.sub_langs = "ko, en".into() }));
        assert_eq!(after(&a, "--sub-langs"), "ko,en");
    }
}
