//! 입력값 검사. 오류의 키는 칸 이름, 값은 화면 문구.

use std::collections::BTreeMap;
use std::path::Path;

use super::args::split_extra;
use super::model::{AppSettings, DownloadOptions};

/// 고급 인자 문자열을 yt-dlp 로 확인해 오류 문구를 돌려주는 함수. 맞으면 `None`.
pub type ExtraCheck<'a> = &'a dyn Fn(&str) -> Option<String>;

/// `90`, `1:30`, `1:02:03`, 소수 초 → 초.
pub fn parse_duration(text: &str) -> Option<f64> {
    let parts: Vec<&str> = text.trim().split(':').collect();
    if parts.len() > 3 {
        return None;
    }
    let mut total = 0.0;
    for (i, part) in parts.iter().enumerate() {
        let last = i == parts.len() - 1;
        let digits_only = !part.is_empty() && part.chars().all(|c| c.is_ascii_digit() || (last && c == '.'));
        if !digits_only || part.matches('.').count() > 1 || *part == "." {
            return None;
        }
        total = total * 60.0 + part.parse::<f64>().ok()?;
    }
    Some(total)
}

/// `500K`, `5M`, `1.5M`, `2G` → 바이트. 단위가 없으면 바이트.
pub fn parse_bytes(text: &str) -> Option<u64> {
    let (number, unit) = match text.chars().last()? {
        c if c.is_ascii_alphabetic() => (&text[..text.len() - 1], c.to_ascii_uppercase()),
        _ => (text, 'B'),
    };
    let power = "BKMGTPEZY".find(unit)? as i32;
    let valid = !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit() || c == '.')
        && number.matches('.').count() <= 1
        && number.starts_with(|c: char| c.is_ascii_digit())
        && number.ends_with(|c: char| c.is_ascii_digit());
    if !valid {
        return None;
    }
    Some((number.parse::<f64>().ok()? * 1024f64.powi(power)).round() as u64)
}

/// 앞에서 부호 있는 정수를 떼어 낸다.
fn take_int(s: &str) -> Option<&str> {
    let body = s.strip_prefix(['+', '-']).unwrap_or(s);
    let digits = body.chars().take_while(|c| c.is_ascii_digit()).count();
    (digits > 0).then(|| &body[digits..])
}

fn segment_ok(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }
    // [시작][(:|-)[끝|inf][:간격]]
    let mut rest = take_int(segment).unwrap_or(segment);
    if rest.is_empty() {
        return true;
    }
    rest = match rest.strip_prefix([':', '-']) {
        Some(r) => r,
        None => return false,
    };
    rest = rest
        .strip_prefix("infinite")
        .or_else(|| rest.strip_prefix("inf"))
        .or_else(|| take_int(rest))
        .unwrap_or(rest);
    if rest.is_empty() {
        return true;
    }
    matches!(rest.strip_prefix(':').and_then(take_int), Some(""))
}

/// yt-dlp `-I` 문법: `1-5,8,10-`, `-3`, `1:10:2`.
pub fn playlist_items_ok(text: &str) -> bool {
    text.split(',').all(segment_ok)
}

/// 패널 값 오류.
pub fn validate_options(o: &DownloadOptions) -> BTreeMap<&'static str, String> {
    let mut errors = BTreeMap::new();
    if o.kind == "video" && o.container == "webm" && o.codec == "h264" {
        errors.insert("codec", crate::lang::tr("WEBM은 H.264를 지원하지 않음. 코덱이나 파일 형식을 바꾸세요", "WEBM can't hold H.264. Change the codec or the container").to_string());
    }
    if !o.section_start.is_empty() || !o.section_end.is_empty() {
        let start = if o.section_start.is_empty() { Some(0.0) } else { parse_duration(&o.section_start) };
        let end = if o.section_end.is_empty() { Some(f64::INFINITY) } else { parse_duration(&o.section_end) };
        match (start, end) {
            (Some(s), Some(e)) if s >= e => {
                errors.insert("section", crate::lang::tr("끝 시간은 시작보다 뒤여야 함", "End time must be after the start").to_string());
            }
            (Some(_), Some(_)) => {}
            _ => {
                errors.insert("section", crate::lang::tr("시간 형식: 1:30", "Time format: 1:30").to_string());
            }
        }
    }
    if o.playlist == "all" && !o.playlist_items.is_empty() && !playlist_items_ok(&o.playlist_items.replace(' ', "")) {
        errors.insert("playlist_items", crate::lang::tr("범위 형식: 1-10, 15", "Range format: 1-10, 15").to_string());
    }
    errors
}

/// 고급 옵션 칸에 링크(`://`)가 들어 있는가. 링크가 있으면 yt-dlp 가 실제로 받으러 간다.
pub fn extra_has_url(text: &str) -> bool {
    split_extra(text).iter().any(|part| part.contains("://") && !part.starts_with('-'))
}

/// 설정 화면 값 오류.
pub fn validate_settings(s: &AppSettings, extra: ExtraCheck) -> BTreeMap<&'static str, String> {
    let mut errors = BTreeMap::new();
    if !s.rate_limit.is_empty() && parse_bytes(&s.rate_limit).is_none() {
        errors.insert("rate_limit", crate::lang::tr("속도 형식: 5M, 500K", "Speed format: 5M, 500K").to_string());
    }
    if s.cookies == "file" && !Path::new(&s.cookie_file).is_file() {
        errors.insert("cookie_file", crate::lang::tr("쿠키 파일 없음", "Cookie file not found").to_string());
    }
    if s.name_template == "custom" && s.custom_template.trim().is_empty() {
        errors.insert("custom_template", crate::lang::tr("파일 이름 규칙을 입력하세요", "Enter a file name template").to_string());
    }
    if !s.extra_args.trim().is_empty() {
        if extra_has_url(&s.extra_args) {
            errors.insert("extra_args", crate::lang::tr("고급 옵션에 링크는 넣을 수 없음", "Links aren't allowed in advanced options").to_string());
        } else if let Some(msg) = extra(&s.extra_args) {
            errors.insert("extra_args", msg);
        }
    }
    errors
}

/// 오류 키 → 그 오류를 일으키는 칸.
fn causes(error: &str) -> &'static [&'static str] {
    match error {
        "rate_limit" => &["rate_limit"],
        "cookie_file" => &["cookies", "cookie_file"],
        "custom_template" => &["name_template", "custom_template"],
        "extra_args" => &["extra_args"],
        _ => &[],
    }
}

/// 칸 몇 개(`changed`)를 바꾼 설정 `cand` 와, 바꾼 칸 때문에 생긴 오류. 오류가 있으면 `s` 를 그대로 돌려준다.
///
/// 이미 저장된 다른 값이 지금은 틀려도(지워진 쿠키 파일 등) 바꾼 칸은 저장된다.
pub fn update_settings(
    s: &AppSettings,
    cand: AppSettings,
    changed: &[&str],
    extra: ExtraCheck,
) -> (AppSettings, BTreeMap<&'static str, String>) {
    let mut errors = validate_settings(&cand, extra);
    errors.retain(|key, _| causes(key).iter().any(|field| changed.contains(field)));
    if errors.is_empty() {
        (cand, errors)
    } else {
        (s.clone(), errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(f: impl FnOnce(&mut DownloadOptions)) -> DownloadOptions {
        let mut o = DownloadOptions::default();
        f(&mut o);
        o
    }

    fn set(f: impl FnOnce(&mut AppSettings)) -> AppSettings {
        let mut s = AppSettings::default();
        f(&mut s);
        s
    }

    /// yt-dlp 를 부르지 않는 가짜 검사: `--nope` 가 들어 있으면 오류.
    fn fake(text: &str) -> Option<String> {
        text.contains("--nope").then(|| "고급 옵션 오류: no such option: --nope".to_string())
    }

    fn vs(s: &AppSettings) -> BTreeMap<&'static str, String> {
        validate_settings(s, &fake)
    }

    #[test]
    fn duration_table() {
        for (text, secs) in [("90", 90.0), ("1:30", 90.0), ("1:02:03", 3723.0), ("0", 0.0), ("1.5", 1.5), ("01:05.5", 65.5), (" 2:00 ", 120.0)] {
            assert_eq!(parse_duration(text), Some(secs), "{text}");
        }
        for text in ["abc", "1:", ":30", "", "1:2:3:4", "-5", "1:x"] {
            assert_eq!(parse_duration(text), None, "{text}");
        }
    }

    #[test]
    fn bytes_table() {
        for (text, n) in [("500K", 512_000), ("5M", 5 * 1024 * 1024), ("1.5M", 1_572_864), ("2g", 2 * 1024 * 1024 * 1024), ("100", 100), ("500k", 512_000)] {
            assert_eq!(parse_bytes(text), Some(n), "{text}");
        }
        for text in ["fast", "5X", "", "M", "1.5.2M", "-1M", "5 M"] {
            assert_eq!(parse_bytes(text), None, "{text}");
        }
    }

    #[test]
    fn playlist_items_table() {
        for text in ["1-5,8,10-", "-3", "1:10:2", "7", "1-10,15", "3-inf", "::2", "5:"] {
            assert!(playlist_items_ok(text), "{text}");
        }
        for text in ["x", "1-,", "", "1,,2", "1-2-3", "a-b", "1:2:3:4"] {
            assert!(!playlist_items_ok(text), "{text}");
        }
    }

    #[test]
    fn valid_defaults() {
        assert!(validate_options(&DownloadOptions::default()).is_empty());
        assert!(vs(&AppSettings::default()).is_empty());
    }

    #[test]
    fn sections() {
        assert!(validate_options(&opts(|o| { o.section_start = "1:30".into(); o.section_end = "3:00".into() })).is_empty());
        assert_eq!(validate_options(&opts(|o| o.section_start = "abc".into()))["section"], "시간 형식: 1:30");
        assert_eq!(validate_options(&opts(|o| { o.section_start = "3:00".into(); o.section_end = "1:00".into() }))["section"],
                   "끝 시간은 시작보다 뒤여야 함");
        assert_eq!(validate_options(&opts(|o| { o.section_start = "1:00".into(); o.section_end = "1:00".into() }))["section"],
                   "끝 시간은 시작보다 뒤여야 함");
        assert!(validate_options(&opts(|o| o.section_end = "0:10".into())).is_empty());
    }

    #[test]
    fn playlist_items_only_when_all() {
        assert!(validate_options(&opts(|o| { o.playlist = "single".into(); o.playlist_items = "x".into() })).is_empty());
        assert!(validate_options(&opts(|o| { o.playlist = "all".into(); o.playlist_items = "1-10, 15".into() })).is_empty());
        assert_eq!(validate_options(&opts(|o| { o.playlist = "all".into(); o.playlist_items = "x".into() }))["playlist_items"],
                   "범위 형식: 1-10, 15");
    }

    #[test]
    fn rate_limit() {
        assert!(vs(&set(|s| s.rate_limit = "500K".into())).is_empty());
        assert_eq!(vs(&set(|s| s.rate_limit = "fast".into()))["rate_limit"], "속도 형식: 5M, 500K");
    }

    #[test]
    fn cookie_file() {
        // 쿠키 모드가 file 이 아닐 때는 보지 않음
        assert!(vs(&set(|s| s.cookie_file = "Z:/none.txt".into())).is_empty());
        let dir = std::env::temp_dir().join(format!("qm-validate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let missing = dir.join("no.txt").to_string_lossy().to_string();
        assert_eq!(vs(&set(|s| { s.cookies = "file".into(); s.cookie_file = missing }))["cookie_file"], "쿠키 파일 없음");
        let file = dir.join("c.txt");
        std::fs::write(&file, "x").unwrap();
        assert!(vs(&set(|s| { s.cookies = "file".into(); s.cookie_file = file.to_string_lossy().to_string() })).is_empty());
        // 폴더는 파일이 아니다
        assert!(!vs(&set(|s| { s.cookies = "file".into(); s.cookie_file = dir.to_string_lossy().to_string() })).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn custom_template() {
        assert_eq!(vs(&set(|s| { s.name_template = "custom".into(); s.custom_template = " ".into() }))["custom_template"],
                   "파일 이름 규칙을 입력하세요");
    }

    #[test]
    fn extra_args() {
        assert!(vs(&set(|s| s.extra_args = "--limit-rate 1M --no-mtime".into())).is_empty());
        let errors = vs(&set(|s| s.extra_args = "--nope".into()));
        assert!(errors["extra_args"].starts_with("고급 옵션 오류: ") && errors["extra_args"].contains("--nope"));
        assert_eq!(vs(&set(|s| s.extra_args = "https://x.com/v".into()))["extra_args"], "고급 옵션에 링크는 넣을 수 없음");
        assert!(vs(&set(|s| s.extra_args = "   ".into())).is_empty());
    }

    #[test]
    fn link_in_extra_args_never_reaches_yt_dlp() {
        let called = std::cell::Cell::new(false);
        let check = |_: &str| -> Option<String> {
            called.set(true);
            None
        };
        let errors = validate_settings(&set(|s| s.extra_args = "--no-mtime \"https://x.com/v\"".into()), &check);
        assert_eq!(errors["extra_args"], "고급 옵션에 링크는 넣을 수 없음");
        assert!(!called.get());
        assert!(extra_has_url("-o x http://a/b") && !extra_has_url("--proxy socks5:host"));
    }

    #[test]
    fn webm_cannot_hold_h264() {
        // yt-dlp 가 H.264 를 골라 WEBM 으로 합치려다 알 수 없는 오류로 실패한다
        assert_eq!(validate_options(&opts(|o| { o.container = "webm".into(); o.codec = "h264".into() }))["codec"],
                   "WEBM은 H.264를 지원하지 않음. 코덱이나 파일 형식을 바꾸세요");
        assert!(validate_options(&opts(|o| { o.container = "webm".into(); o.codec = "h264".into(); o.kind = "audio".into() })).is_empty());
        assert!(validate_options(&opts(|o| { o.container = "mkv".into(); o.codec = "h264".into() })).is_empty());
    }

    #[test]
    fn update_settings_saves_valid() {
        let s = AppSettings::default();
        let (new, errors) = update_settings(&s, set(|c| c.proxy = "http://p:1".into()), &["proxy"], &fake);
        assert!(errors.is_empty());
        assert_eq!(new.proxy, "http://p:1");
    }

    #[test]
    fn update_settings_rejects_bad_rate() {
        let s = AppSettings::default();
        let (new, errors) = update_settings(&s, set(|c| c.rate_limit = "fast".into()), &["rate_limit"], &fake);
        assert_eq!(new, s);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors["rate_limit"], "속도 형식: 5M, 500K");
    }

    #[test]
    fn stale_cookie_file_does_not_block_proxy() {
        // 저장해 둔 쿠키 파일이 나중에 지워져도 다른 칸은 저장돼야 한다
        let s = set(|s| { s.cookies = "file".into(); s.cookie_file = "없는.txt".into() });
        let mut cand = s.clone();
        cand.proxy = "http://p:1".into();
        let (new, errors) = update_settings(&s, cand, &["proxy"], &fake);
        assert!(errors.is_empty());
        assert_eq!(new.proxy, "http://p:1");
    }

    #[test]
    fn cookie_mode_file_checks_path() {
        let s = AppSettings::default();
        let cand = set(|c| { c.cookies = "file".into(); c.cookie_file = "없는.txt".into() });
        let (new, errors) = update_settings(&s, cand, &["cookies", "cookie_file"], &fake);
        assert_eq!(new, s);
        assert_eq!(errors["cookie_file"], "쿠키 파일 없음");
        let s = set(|s| s.cookie_file = "없는.txt".into());
        let mut cand = s.clone();
        cand.cookies = "browser".into();
        let (new, errors) = update_settings(&s, cand, &["cookies"], &fake);
        assert!(errors.is_empty());
        assert_eq!(new.cookies, "browser");
    }

    #[test]
    fn custom_template_checked_with_name() {
        let s = AppSettings::default();
        let cand = set(|c| { c.name_template = "custom".into(); c.custom_template = " ".into() });
        let (new, errors) = update_settings(&s, cand, &["name_template", "custom_template"], &fake);
        assert_eq!(new, s);
        assert_eq!(errors["custom_template"], "파일 이름 규칙을 입력하세요");
    }

    #[test]
    fn bad_extra_args() {
        let s = AppSettings::default();
        let (new, errors) = update_settings(&s, set(|c| c.extra_args = "--nope".into()), &["extra_args"], &fake);
        assert_eq!(new, s);
        assert_eq!(errors.keys().copied().collect::<Vec<_>>(), ["extra_args"]);
    }
}
