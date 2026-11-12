//! 왼쪽 메뉴의 상태. 다운로드 / 설정(로그인·네트워크·파일·고급) / 업데이트.

pub const SETTINGS_KEYS: [&str; 4] = ["login", "network", "file", "advanced"];

pub fn is_settings_page(key: &str) -> bool {
    SETTINGS_KEYS.contains(&key)
}

/// 메뉴 항목 `clicked` 를 눌렀을 때의 (화면, 설정 묶음 펼침).
///
/// 설정 화면으로 가면 묶음을 펼치고, 다른 화면으로 가면 접는다. '설정' 자체는 묶음만 여닫는다.
pub fn nav_after_click(page: &str, open: bool, clicked: &str) -> (String, bool) {
    if clicked != "settings" {
        return (clicked.to_string(), is_settings_page(clicked));
    }
    if open {
        return (page.to_string(), false);
    }
    if is_settings_page(page) {
        // 접어 둔 채 보던 설정 화면은 그대로 둔다
        return (page.to_string(), true);
    }
    ("login".to_string(), true)
}

/// '설정' 항목의 선택 표시: 보던 설정 화면이 접혀 숨었을 때만.
pub fn settings_item_checked(page: &str, open: bool) -> bool {
    is_settings_page(page) && !open
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 처음 상태에서 차례로 누른 뒤의 (화면, 펼침).
    fn click(keys: &[&str]) -> (String, bool) {
        keys.iter().fold(("download".to_string(), false), |(page, open), key| nav_after_click(&page, open, key))
    }

    fn state(page: &str, open: bool) -> (String, bool) {
        (page.to_string(), open)
    }

    #[test]
    fn settings_expands_to_login() {
        assert_eq!(click(&["settings"]), state("login", true));
    }

    #[test]
    fn settings_again_collapses_but_keeps_the_page() {
        // 접기만 하고 화면은 그대로. 보던 설정 화면이 숨었으니 '설정'에 선택 표시
        assert_eq!(click(&["settings", "settings"]), state("login", false));
        assert!(settings_item_checked("login", false));
        assert!(!settings_item_checked("login", true));
        assert!(!settings_item_checked("download", false));
    }

    #[test]
    fn category_and_download() {
        assert_eq!(click(&["settings", "network"]), state("network", true));
        assert_eq!(click(&["settings", "network", "download"]), state("download", false));
    }

    #[test]
    fn reexpand_keeps_settings_page() {
        assert_eq!(click(&["settings", "file", "settings", "settings"]), state("file", true));
    }

    #[test]
    fn update_collapses_settings() {
        assert_eq!(click(&["settings", "update"]), state("update", false));
        assert_eq!(click(&["update", "settings"]), state("login", true));
    }

    #[test]
    fn selecting_a_settings_page_directly_expands() {
        assert_eq!(nav_after_click("download", false, "advanced"), state("advanced", true));
        assert!(SETTINGS_KEYS.iter().all(|k| is_settings_page(k)));
        assert!(!is_settings_page("download") && !is_settings_page("update") && !is_settings_page("settings"));
    }
}
