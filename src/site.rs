//! 다운로드 카드에 보여 줄 사이트 이름과 아이콘.
//!
//! 아이콘은 그 사이트의 /favicon.ico 에서 직접 받는다. 어차피 영상을 받으려고 접속하는 곳이라
//! 제3자 서비스에 주소를 보내지 않는다. 받은 아이콘은 앱 데이터 폴더에 저장해 두고 다시 쓴다.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use crate::http;

/// yt-dlp extractor_key 접두어 → 보기 좋은 이름
const NAMES: &[(&str, &str)] = &[
    ("youtube", "YouTube"), ("twitter", "X"), ("instagram", "Instagram"), ("tiktok", "TikTok"),
    ("facebook", "Facebook"), ("twitch", "Twitch"), ("vimeo", "Vimeo"), ("soundcloud", "SoundCloud"),
    ("dailymotion", "Dailymotion"), ("reddit", "Reddit"), ("bilibili", "bilibili"),
    ("niconico", "니코니코"), ("naver", "네이버 TV"), ("chzzk", "치지직"), ("afreecatv", "SOOP"),
    ("soop", "SOOP"), ("kakao", "카카오TV"),
];

/// 저장·그리기 크기. 화면에는 16px 로 (고배율 화면에서 선명하게)
pub const ICON_PX: u32 = 32;
const MAX_BYTES: usize = 256 * 1024;
const MAX_AGE: Duration = Duration::from_secs(30 * 86400);

/// 링크의 호스트 이름. `www.`·`m.` 은 뗀다. 링크가 아니면 빈 글.
pub fn domain_of(url: &str) -> String {
    let Some((_, rest)) = url.trim().split_once("://") else {
        return String::new();
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority.rsplit('@').next().unwrap_or("");
    let host = if host.starts_with('[') { host } else { host.split(':').next().unwrap_or("") };
    let mut host = host.to_lowercase();
    for prefix in ["www.", "m."] {
        if let Some(stripped) = host.strip_prefix(prefix) {
            host = stripped.to_string();
        }
    }
    host
}

pub fn display_name(extractor_key: &str, host: &str) -> String {
    let base = extractor_key.split(':').next().unwrap_or("");
    let low = base.to_lowercase();
    if base.is_empty() || low == "generic" {
        return host.to_string();
    }
    match NAMES.iter().find(|(prefix, _)| low.starts_with(prefix)) {
        Some((_, name)) => name.to_string(),
        None => base.to_string(),
    }
}

pub fn icon_url(host: &str) -> String {
    format!("https://{host}/favicon.ico")
}

/// 아이콘이 없을 때 대신 보일 글자.
pub fn letter_of(name: &str) -> String {
    name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_else(|| "?".to_string())
}

#[derive(Clone, Debug, PartialEq)]
pub struct IconPixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// (주소, 프록시) → 받은 바이트. 못 받으면 `None`.
pub type Fetch = Box<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>;

fn fetch_bytes(url: &str, proxy: &str) -> Option<Vec<u8>> {
    http::get_bytes(&http::agent(proxy), url, "*/*", MAX_BYTES, Duration::from_secs(5)).ok()
}

pub struct SiteIcons {
    cache_dir: PathBuf,
    fetch: Fetch,
    proxy: Mutex<String>,
    /// 이번 실행에 알아본 곳. 실패한 곳(`None`)은 다시 받지 않는다.
    memory: Mutex<HashMap<String, Option<IconPixels>>>,
}

impl SiteIcons {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self::with_fetch(cache_dir, Box::new(fetch_bytes))
    }

    pub fn with_fetch(cache_dir: PathBuf, fetch: Fetch) -> Self {
        Self { cache_dir, fetch, proxy: Mutex::new(String::new()), memory: Mutex::new(HashMap::new()) }
    }

    /// 설정 화면의 프록시.
    pub fn set_proxy(&self, proxy: &str) {
        *self.proxy.lock().unwrap() = proxy.to_string();
    }

    /// 그 사이트의 아이콘. 저장해 둔 것이 없으면 받아 온다(막는다). 없으면 `None`.
    pub fn load(&self, host: &str) -> Option<IconPixels> {
        // 호스트 이름이 그대로 파일 이름이 된다
        let plain = !host.is_empty()
            && host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
            && !host.starts_with('.')
            && !host.contains("..");
        if !plain {
            return None;
        }
        if let Some(known) = self.memory.lock().unwrap().get(host) {
            return known.clone();
        }
        let icon = self.cached(host).or_else(|| self.fetched(host));
        self.memory.lock().unwrap().insert(host.to_string(), icon.clone());
        icon
    }

    fn path(&self, host: &str) -> PathBuf {
        self.cache_dir.join(format!("{host}.png"))
    }

    fn cached(&self, host: &str) -> Option<IconPixels> {
        let path = self.path(host);
        let age = path.metadata().ok()?.modified().ok()?.elapsed().unwrap_or_default();
        if age >= MAX_AGE {
            return None;
        }
        pixels(&image::load_from_memory(&std::fs::read(path).ok()?).ok()?)
    }

    fn fetched(&self, host: &str) -> Option<IconPixels> {
        let proxy = self.proxy.lock().unwrap().clone();
        let bytes = (self.fetch)(&icon_url(host), &proxy)?;
        let image = image::load_from_memory(&bytes).ok()?;
        let image = image.resize(ICON_PX, ICON_PX, image::imageops::FilterType::Lanczos3);
        if std::fs::create_dir_all(&self.cache_dir).is_ok() {
            let _ = image.save_with_format(self.path(host), image::ImageFormat::Png);
        }
        pixels(&image)
    }
}

fn pixels(image: &image::DynamicImage) -> Option<IconPixels> {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    (width > 0 && height > 0).then(|| IconPixels { width, height, rgba: rgba.into_raw() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn domain_of_cases() {
        assert_eq!(domain_of("https://www.youtube.com/watch?v=x"), "youtube.com");
        assert_eq!(domain_of("https://m.youtube.com/watch?v=x"), "youtube.com");
        assert_eq!(domain_of("https://tv.naver.com/v/1"), "tv.naver.com");
        assert_eq!(domain_of("not a url"), "");
        assert_eq!(domain_of("  HTTPS://User:pw@WWW.Example.COM:8443/a?b#c  "), "example.com");
        assert_eq!(domain_of("https://youtu.be"), "youtu.be");
        assert_eq!(domain_of(""), "");
    }

    #[test]
    fn display_name_cases() {
        assert_eq!(display_name("Youtube", "youtube.com"), "YouTube");
        assert_eq!(display_name("YoutubeTab", "youtube.com"), "YouTube");
        assert_eq!(display_name("Twitch:vod", "twitch.tv"), "Twitch");
        assert_eq!(display_name("Twitter", "x.com"), "X");
        assert_eq!(display_name("SoopLive", "sooplive.co.kr"), "SOOP");
        assert_eq!(display_name("Naver", "tv.naver.com"), "네이버 TV");
        assert_eq!(display_name("Generic", "example.com"), "example.com");
        assert_eq!(display_name("", "example.com"), "example.com");
        assert_eq!(display_name("Rumble", "rumble.com"), "Rumble");
    }

    #[test]
    fn icon_url_and_letter() {
        assert_eq!(icon_url("youtube.com"), "https://youtube.com/favicon.ico");
        assert_eq!(letter_of("youtube.com"), "Y");
        assert_eq!(letter_of("네이버 TV"), "네");
        assert_eq!(letter_of(""), "?");
    }

    fn png(size: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(size, size, image::Rgba([0, 255, 0, 255]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        img.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("qm-site-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// 부른 주소를 세는 가짜 받기.
    fn counting(body: Option<Vec<u8>>) -> (Arc<AtomicUsize>, Fetch) {
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = calls.clone();
        let fetch: Fetch = Box::new(move |url, _proxy| {
            assert!(url.ends_with("/favicon.ico"), "{url}");
            seen.fetch_add(1, Ordering::SeqCst);
            body.clone()
        });
        (calls, fetch)
    }

    #[test]
    fn fetches_scales_and_caches() {
        let dir = temp("cache");
        let (calls, fetch) = counting(Some(png(48)));
        let icons = SiteIcons::with_fetch(dir.clone(), fetch);
        let icon = icons.load("youtube.com").unwrap();
        assert_eq!((icon.width, icon.height), (32, 32));
        assert_eq!(icon.rgba.len(), 32 * 32 * 4);
        assert_eq!(icon.rgba[..4], [0, 255, 0, 255]);
        assert!(dir.join("youtube.com.png").exists());
        // 이번 실행 안에서는 다시 받지 않는다
        icons.load("youtube.com").unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        // 다음 실행은 디스크에서 읽는다
        let (calls, fetch) = counting(None);
        assert!(SiteIcons::with_fetch(dir, fetch).load("youtube.com").is_some());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn stale_cache_refetches() {
        let dir = temp("stale");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.com.png");
        std::fs::write(&file, png(32)).unwrap();
        let old = std::time::SystemTime::now() - Duration::from_secs(31 * 86400);
        std::fs::File::options().write(true).open(&file).unwrap().set_modified(old).unwrap();
        let (calls, fetch) = counting(Some(png(16)));
        assert!(SiteIcons::with_fetch(dir, fetch).load("a.com").is_some());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn html_response_gives_no_icon_and_is_not_retried() {
        let dir = temp("html");
        let (calls, fetch) = counting(Some(b"<html>not found</html>".to_vec()));
        let icons = SiteIcons::with_fetch(dir.clone(), fetch);
        assert!(icons.load("b.com").is_none());
        assert!(icons.load("b.com").is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!dir.join("b.com.png").exists());
    }

    #[test]
    fn odd_hosts_are_never_fetched_or_written() {
        let dir = temp("odd");
        let (calls, fetch) = counting(Some(png(16)));
        let icons = SiteIcons::with_fetch(dir, fetch);
        for host in ["", "..", "a/b", "a\\b", "[::1]", "a b.com", "con:1"] {
            assert!(icons.load(host).is_none(), "{host}");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn fetch_gets_the_proxy() {
        // 프록시를 켠 사람의 실제 IP 로 사이트에 직접 붙으면 안 된다
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = seen.clone();
        let icons = SiteIcons::with_fetch(temp("proxy"), Box::new(move |url, proxy| {
            sink.lock().unwrap().push((url.to_string(), proxy.to_string()));
            None
        }));
        icons.set_proxy("http://127.0.0.1:8080");
        icons.load("c.com");
        assert_eq!(*seen.lock().unwrap(), [("https://c.com/favicon.ico".to_string(), "http://127.0.0.1:8080".to_string())]);
    }
}
