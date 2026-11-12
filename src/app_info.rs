//! 앱 이름과 저장소. 옛 이름(유튜브 다운로더)은 이전 처리에만 쓴다.

use std::path::PathBuf;

pub const NAME: &str = "QuickMedia";
pub const EXE: &str = "QuickMedia.exe";
pub const REPO: &str = "glglekdy/quickmedia";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// `HKEY_CURRENT_USER` 아래 설정 위치. 1.x(QSettings)와 같다.
pub const SETTINGS_KEY: &str = r"Software\QuickMedia\QuickMedia";
pub const LEGACY_SETTINGS_KEY: &str = r"Software\YoutubeDownloader\YoutubeDownloader";
pub const LEGACY_EXE: &str = "YoutubeDownloader.exe";

/// 받은 기록, 사이트 아이콘, yt-dlp 갱신본을 두는 곳: `%APPDATA%\QuickMedia`.
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join(NAME)
}

pub fn user_agent() -> String {
    format!("{NAME}/{VERSION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(NAME, "QuickMedia");
        assert_eq!(EXE, "QuickMedia.exe");
        assert_eq!(REPO, "glglekdy/quickmedia");
        assert_eq!(SETTINGS_KEY, r"Software\QuickMedia\QuickMedia");
        assert_eq!(LEGACY_SETTINGS_KEY, r"Software\YoutubeDownloader\YoutubeDownloader");
        assert_eq!(LEGACY_EXE, "YoutubeDownloader.exe");
    }

    #[test]
    fn version_and_paths() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert_eq!(user_agent(), format!("QuickMedia/{VERSION}"));
        let dir = data_dir();
        assert!(dir.ends_with("QuickMedia") && dir.is_absolute(), "{dir:?}");
    }
}
