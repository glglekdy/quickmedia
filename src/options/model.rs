//! 세부 옵션 패널 값(`DownloadOptions`)과 설정 화면 값(`AppSettings`).
//!
//! 저장소(레지스트리)에는 필드 이름 그대로 문자열로 들어간다. 되읽을 때 목록 밖의 값이나
//! 범위 밖의 숫자는 기본값으로 둔다.

use std::collections::{BTreeMap, HashMap};

pub const QUALITIES: &[&str] = &["best", "2160", "1440", "1080", "720", "480", "360"];
pub const CONTAINERS: &[&str] = &["mp4", "mkv", "webm"];
pub const CODECS: &[&str] = &["auto", "h264", "vp9", "av1"];
pub const AUDIO_FORMATS: &[&str] = &["mp3", "m4a", "opus", "flac", "wav"];
pub const AUDIO_QUALITIES: &[&str] = &["best", "320", "192", "128"];
pub const SUB_MODES: &[&str] = &["embed", "file"];
pub const SUB_FORMATS: &[&str] = &["srt", "vtt", "ass"];
pub const PLAYLIST_MODES: &[&str] = &["single", "all"];
pub const COOKIE_MODES: &[&str] = &["none", "browser", "file"];
pub const BROWSERS: &[&str] = &["firefox", "edge", "chrome", "whale", "brave", "opera", "vivaldi", "chromium"];
pub const KINDS: &[&str] = &["video", "audio"];
pub const NAME_TEMPLATES: &[(&str, &str)] = &[
    ("title", "%(title)s.%(ext)s"),
    ("title_id", "%(title)s [%(id)s].%(ext)s"),
    ("uploader_title", "%(uploader)s - %(title)s.%(ext)s"),
    ("date_title", "%(upload_date>%Y-%m-%d)s %(title)s.%(ext)s"),
];
pub const NAME_TEMPLATE_KEYS: &[&str] = &["title", "title_id", "uploader_title", "date_title", "custom"];
pub const LOSSLESS: &[&str] = &["flac", "wav"];
pub const PRESET_KEYS: [&str; 3] = ["best", "720p", "mp3"];

/// 저장된 글자를 필드 값으로. 틀리면 `None` (= 기본값을 쓴다).
trait Stored: Sized {
    fn load(text: &str, choices: Option<&[&str]>, range: Option<(u32, u32)>) -> Option<Self>;
    fn store(&self) -> String;
}

impl Stored for String {
    fn load(text: &str, choices: Option<&[&str]>, _: Option<(u32, u32)>) -> Option<Self> {
        match choices {
            Some(list) if !list.contains(&text) => None,
            _ => Some(text.to_string()),
        }
    }
    fn store(&self) -> String {
        self.clone()
    }
}

impl Stored for bool {
    fn load(text: &str, _: Option<&[&str]>, _: Option<(u32, u32)>) -> Option<Self> {
        match text.trim().to_lowercase().as_str() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        }
    }
    fn store(&self) -> String {
        self.to_string()
    }
}

impl Stored for u32 {
    fn load(text: &str, _: Option<&[&str]>, range: Option<(u32, u32)>) -> Option<Self> {
        let n: u32 = text.trim().parse().ok()?;
        match range {
            Some((lo, hi)) if n < lo || n > hi => None,
            _ => Some(n),
        }
    }
    fn store(&self) -> String {
        self.to_string()
    }
}

/// 필드 · 형 · 기본값 · 고를 수 있는 값 · 범위 표에서 구조체와 저장·되읽기를 만든다.
macro_rules! stored_struct {
    ($(#[$meta:meta])* $name:ident { $($field:ident : $ty:ty = $default:expr, $choices:expr, $range:expr;)* }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name { $(pub $field: $ty,)* }

        impl Default for $name {
            fn default() -> Self {
                Self { $($field: $default,)* }
            }
        }

        impl $name {
            /// 저장소에 쓰는 필드 이름.
            pub const FIELDS: &[&str] = &[$(stringify!($field),)*];

            fn to_map(&self) -> BTreeMap<String, String> {
                let mut m = BTreeMap::new();
                $(m.insert(stringify!($field).to_string(), Stored::store(&self.$field));)*
                m
            }

            fn from_map(d: &HashMap<String, String>) -> Self {
                let mut v = Self::default();
                $(
                    if let Some(text) = d.get(stringify!($field)) {
                        if let Some(value) = <$ty as Stored>::load(text, $choices, $range) {
                            v.$field = value;
                        }
                    }
                )*
                v
            }
        }
    };
}

const NONE: Option<&[&str]> = None;
const ANY: Option<(u32, u32)> = None;

stored_struct! {
    /// 세부 옵션 패널 값.
    DownloadOptions {
        kind: String = "video".into(), Some(KINDS), ANY;
        quality: String = "best".into(), Some(QUALITIES), ANY;
        container: String = "mp4".into(), Some(CONTAINERS), ANY;
        codec: String = "auto".into(), Some(CODECS), ANY;
        audio_format: String = "mp3".into(), Some(AUDIO_FORMATS), ANY;
        audio_quality: String = "192".into(), Some(AUDIO_QUALITIES), ANY;
        subs: bool = false, NONE, ANY;
        sub_langs: String = "ko".into(), NONE, ANY;
        auto_subs: bool = false, NONE, ANY;
        sub_mode: String = "embed".into(), Some(SUB_MODES), ANY;
        sub_format: String = "srt".into(), Some(SUB_FORMATS), ANY;
        embed_thumbnail: bool = false, NONE, ANY;
        save_thumbnail: bool = false, NONE, ANY;
        embed_metadata: bool = false, NONE, ANY;
        embed_chapters: bool = false, NONE, ANY;
        split_chapters: bool = false, NONE, ANY;
        section_start: String = String::new(), NONE, ANY;
        section_end: String = String::new(), NONE, ANY;
        playlist: String = "single".into(), Some(PLAYLIST_MODES), ANY;
        playlist_items: String = String::new(), NONE, ANY;
        number_prefix: bool = false, NONE, ANY;
        playlist_folder: bool = false, NONE, ANY;
        skip_archived: bool = false, NONE, ANY;
    }
}

stored_struct! {
    /// 설정 화면 값.
    AppSettings {
        cookies: String = "none".into(), Some(COOKIE_MODES), ANY;
        cookie_browser: String = "firefox".into(), Some(BROWSERS), ANY;
        cookie_file: String = String::new(), NONE, ANY;
        proxy: String = String::new(), NONE, ANY;
        rate_limit: String = String::new(), NONE, ANY;
        retries: u32 = 10, NONE, Some((0, 50));
        name_template: String = "title".into(), Some(NAME_TEMPLATE_KEYS), ANY;
        custom_template: String = "%(title)s.%(ext)s".into(), NONE, ANY;
        overwrite: bool = false, NONE, ANY;
        concurrency: u32 = 2, NONE, Some((1, 5));
        extra_args: String = String::new(), NONE, ANY;
    }
}

pub fn preset_label(key: &str) -> &'static str {
    match key {
        "best" => "최고 화질",
        "720p" => "720p",
        "mp3" => "MP3 음원",
        _ => "",
    }
}

/// 프리셋이 정하는 형식 값만 바꾼다. 나머지 옵션은 그대로.
pub fn apply_preset(opts: &DownloadOptions, key: &str) -> DownloadOptions {
    let mut o = opts.clone();
    match key {
        "best" | "720p" => {
            o.kind = "video".into();
            o.quality = if key == "best" { "best" } else { "720" }.into();
            o.container = "mp4".into();
            o.codec = "auto".into();
        }
        "mp3" => {
            o.kind = "audio".into();
            o.audio_format = "mp3".into();
            o.audio_quality = "192".into();
        }
        _ => {}
    }
    o
}

/// 형식 값이 같은 프리셋. 없으면 `None` (= 사용자 지정).
pub fn matching_preset(opts: &DownloadOptions) -> Option<&'static str> {
    PRESET_KEYS.into_iter().find(|key| apply_preset(opts, key) == *opts)
}

pub fn options_to_map(o: &DownloadOptions) -> BTreeMap<String, String> {
    o.to_map()
}

pub fn settings_to_map(s: &AppSettings) -> BTreeMap<String, String> {
    s.to_map()
}

/// 저장된 패널 값. 아직 저장한 적 없으면 옛 앱의 `format` 값을 프리셋으로 옮긴다.
pub fn options_from_map(d: &HashMap<String, String>, legacy_format: Option<&str>) -> DownloadOptions {
    let opts = DownloadOptions::from_map(d);
    match legacy_format {
        Some(key) if !d.contains_key("kind") && PRESET_KEYS.contains(&key) => apply_preset(&opts, key),
        _ => opts,
    }
}

pub fn settings_from_map(d: &HashMap<String, String>) -> AppSettings {
    AppSettings::from_map(d)
}

/// 카드에 쓸 짧은 요약. 예: `1080p · MKV`, `MP3 · 192k`.
pub fn describe(opts: &DownloadOptions) -> String {
    let mut text = if opts.kind == "audio" {
        let mut t = opts.audio_format.to_uppercase();
        if !LOSSLESS.contains(&opts.audio_format.as_str()) {
            if opts.audio_quality == "best" {
                t.push_str(" · 최고 음질");
            } else {
                t.push_str(&format!(" · {}k", opts.audio_quality));
            }
        }
        t
    } else {
        let quality = if opts.quality == "best" { "최고 화질".to_string() } else { format!("{}p", opts.quality) };
        format!("{quality} · {}", opts.container.to_uppercase())
    };
    if opts.subs {
        text.push_str(" · 자막");
    }
    text
}

/// 세부 옵션 버튼에 붙일, 기본값과 달라진 항목.
pub fn changed_summary(opts: &DownloadOptions) -> Vec<&'static str> {
    let checks = [
        ("형식", matching_preset(opts).is_none()),
        ("자막", opts.subs),
        ("썸네일", opts.embed_thumbnail || opts.save_thumbnail),
        ("메타데이터", opts.embed_metadata),
        ("챕터", opts.embed_chapters || opts.split_chapters),
        ("구간", !opts.section_start.is_empty() || !opts.section_end.is_empty()),
        (
            "재생목록",
            opts.playlist == "all"
                || !opts.playlist_items.is_empty()
                || opts.number_prefix
                || opts.playlist_folder
                || opts.skip_archived,
        ),
    ];
    checks.into_iter().filter(|(_, changed)| *changed).map(|(name, _)| name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn opts(f: impl FnOnce(&mut DownloadOptions)) -> DownloadOptions {
        let mut o = DownloadOptions::default();
        f(&mut o);
        o
    }

    #[test]
    fn defaults() {
        let o = DownloadOptions::default();
        assert_eq!((o.kind.as_str(), o.quality.as_str(), o.container.as_str(), o.codec.as_str()),
                   ("video", "best", "mp4", "auto"));
        assert_eq!((o.audio_format.as_str(), o.audio_quality.as_str(), o.sub_langs.as_str(), o.playlist.as_str()),
                   ("mp3", "192", "ko", "single"));
        let s = AppSettings::default();
        assert_eq!((s.cookies.as_str(), s.cookie_browser.as_str(), s.retries, s.concurrency, s.name_template.as_str()),
                   ("none", "firefox", 10, 2, "title"));
        assert_eq!(s.custom_template, "%(title)s.%(ext)s");
    }

    #[test]
    fn presets_round_trip() {
        let d = DownloadOptions::default();
        assert_eq!(matching_preset(&d), Some("best"));
        assert_eq!(matching_preset(&apply_preset(&d, "720p")), Some("720p"));
        let mp3 = apply_preset(&opts(|o| o.subs = true), "mp3");
        assert_eq!((mp3.kind.as_str(), mp3.audio_format.as_str(), mp3.audio_quality.as_str(), mp3.subs),
                   ("audio", "mp3", "192", true));
        assert_eq!(matching_preset(&mp3), Some("mp3"));
        assert_eq!(matching_preset(&opts(|o| o.quality = "1080".into())), None);
        assert_eq!(matching_preset(&opts(|o| { o.kind = "audio".into(); o.audio_format = "flac".into() })), None);
        assert_eq!(preset_label("best"), "최고 화질");
        assert_eq!(preset_label("720p"), "720p");
        assert_eq!(preset_label("mp3"), "MP3 음원");
    }

    #[test]
    fn from_map_coerces_registry_strings() {
        let o = options_from_map(&map(&[("subs", "true"), ("embed_chapters", "false"), ("quality", "1080"), ("nope", "x")]), None);
        assert!(o.subs && !o.embed_chapters && o.quality == "1080");
        let s = settings_from_map(&map(&[("retries", "3"), ("concurrency", "4"), ("overwrite", "1")]));
        assert_eq!((s.retries, s.concurrency, s.overwrite), (3, 4, true));
    }

    #[test]
    fn from_map_rejects_bad_values() {
        let o = options_from_map(&map(&[("quality", "999"), ("container", "avi"), ("subs", "maybe")]), None);
        assert_eq!((o.quality.as_str(), o.container.as_str(), o.subs), ("best", "mp4", false));
        let s = settings_from_map(&map(&[("concurrency", "9"), ("retries", "-1"), ("cookie_browser", "netscape")]));
        assert_eq!((s.concurrency, s.retries, s.cookie_browser.as_str()), (2, 10, "firefox"));
    }

    #[test]
    fn garbage_values_fall_back_to_defaults() {
        let d = map(&[("quality", "9999"), ("subs", "maybe"), ("kind", "")]);
        assert_eq!(options_from_map(&d, None), DownloadOptions::default());
        let s = map(&[("retries", "abc"), ("concurrency", "99")]);
        assert_eq!(settings_from_map(&s), AppSettings::default());
    }

    #[test]
    fn legacy_format_only_when_no_saved_options() {
        assert_eq!(options_from_map(&map(&[]), Some("mp3")).kind, "audio");
        assert_eq!(options_from_map(&map(&[]), Some("720p")).quality, "720");
        assert_eq!(options_from_map(&map(&[("kind", "video")]), Some("mp3")).kind, "video");
        assert_eq!(options_from_map(&map(&[]), Some("weird")), DownloadOptions::default());
    }

    #[test]
    fn to_map_round_trip() {
        let o = opts(|o| { o.kind = "audio".into(); o.subs = true; o.section_start = "1:30".into() });
        let m: HashMap<String, String> = options_to_map(&o).into_iter().collect();
        assert_eq!(m["subs"], "true");
        assert_eq!(options_from_map(&m, None), o);
        let s = AppSettings { retries: 3, overwrite: true, proxy: "http://p:1".into(), ..Default::default() };
        let m: HashMap<String, String> = settings_to_map(&s).into_iter().collect();
        assert_eq!(m["retries"], "3");
        assert_eq!(settings_from_map(&m), s);
    }

    #[test]
    fn describe_cases() {
        assert_eq!(describe(&DownloadOptions::default()), "최고 화질 · MP4");
        assert_eq!(describe(&opts(|o| { o.quality = "1080".into(); o.container = "mkv".into() })), "1080p · MKV");
        assert_eq!(describe(&opts(|o| o.kind = "audio".into())), "MP3 · 192k");
        assert_eq!(describe(&opts(|o| { o.kind = "audio".into(); o.audio_format = "flac".into() })), "FLAC");
        assert_eq!(describe(&opts(|o| { o.kind = "audio".into(); o.audio_format = "opus".into(); o.audio_quality = "best".into() })),
                   "OPUS · 최고 음질");
        assert_eq!(describe(&opts(|o| o.subs = true)), "최고 화질 · MP4 · 자막");
    }

    #[test]
    fn changed_summary_cases() {
        assert!(changed_summary(&DownloadOptions::default()).is_empty());
        assert!(changed_summary(&apply_preset(&DownloadOptions::default(), "mp3")).is_empty());
        let o = opts(|o| {
            o.quality = "1080".into(); o.subs = true; o.save_thumbnail = true; o.embed_metadata = true;
            o.split_chapters = true; o.section_end = "3:00".into(); o.playlist = "all".into();
        });
        assert_eq!(changed_summary(&o), vec!["형식", "자막", "썸네일", "메타데이터", "챕터", "구간", "재생목록"]);
    }
}
