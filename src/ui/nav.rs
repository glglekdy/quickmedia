//! 화면 전환. 아래 독의 다운로드 / 설정, 설정 화면 위의 탭(일반·로그인·네트워크·파일·고급·업데이트).

pub const SETTINGS_KEYS: [&str; 5] = ["general", "login", "network", "file", "advanced"];
/// 설정 화면 위에 놓이는 탭. 업데이트도 여기에 있다.
pub const TAB_KEYS: [&str; 6] = ["general", "login", "network", "file", "advanced", "update"];

/// 값을 고쳐 저장하는 설정 화면인지(업데이트 화면은 아니다).
pub fn is_settings_page(key: &str) -> bool {
    SETTINGS_KEYS.contains(&key)
}

/// 독의 '설정' 쪽 화면인지.
pub fn is_tab(key: &str) -> bool {
    TAB_KEYS.contains(&key)
}

/// `clicked` 를 눌렀을 때 갈 화면. `last_tab` 은 설정 쪽에서 마지막으로 보던 탭.
///
/// 독의 '설정'(`"settings"`)은 보던 탭으로 돌아간다. 이미 설정 쪽이면 그대로 둔다.
pub fn page_after_click(page: &str, last_tab: &str, clicked: &str) -> String {
    if clicked != "settings" {
        return if clicked == "download" || is_tab(clicked) { clicked.to_string() } else { page.to_string() };
    }
    if is_tab(page) {
        return page.to_string();
    }
    if is_tab(last_tab) { last_tab.to_string() } else { "general".to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 처음 상태에서 차례로 누른 뒤의 (화면, 마지막 탭).
    fn click(keys: &[&str]) -> (String, String) {
        keys.iter().fold(("download".to_string(), String::new()), |(page, last), key| {
            let next = page_after_click(&page, &last, key);
            let last = if is_tab(&next) { next.clone() } else { last };
            (next, last)
        })
    }

    #[test]
    fn settings_opens_general_first() {
        assert_eq!(click(&["settings"]).0, "general");
    }

    #[test]
    fn settings_returns_to_the_tab_last_seen() {
        assert_eq!(click(&["settings", "file", "download", "settings"]).0, "file");
        assert_eq!(click(&["settings", "update", "download", "settings"]).0, "update");
    }

    #[test]
    fn settings_while_in_settings_stays_put() {
        assert_eq!(click(&["settings", "network", "settings"]).0, "network");
    }

    #[test]
    fn tabs_and_download_go_where_they_say() {
        assert_eq!(click(&["settings", "advanced"]).0, "advanced");
        assert_eq!(click(&["settings", "advanced", "download"]).0, "download");
        assert_eq!(page_after_click("download", "", "update"), "update");
    }

    #[test]
    fn unknown_keys_change_nothing() {
        assert_eq!(page_after_click("file", "file", "nope"), "file");
        assert_eq!(page_after_click("download", "nope", "settings"), "general");
    }

    #[test]
    fn which_pages_save_settings() {
        assert!(SETTINGS_KEYS.iter().all(|k| is_settings_page(k) && is_tab(k)));
        assert!(is_tab("update") && !is_settings_page("update"));
        assert!(!is_tab("download") && !is_tab("settings") && !is_settings_page("download"));
    }
}
