//! 설정 화면 4개(로그인 / 네트워크 / 파일 / 고급)의 동작. 바꾸면 바로 저장한다.
//!
//! 화면이 보여 주는 값(`view`)과 저장된 값(`saved`)을 따로 둔다. 틀린 값은 화면에만 남고,
//! 화면을 떠날 때 저장된 값으로 되돌아간다.

use std::collections::BTreeMap;

use crate::options::{extra_has_url, update_settings, AppSettings, ExtraCheck};

/// 한 번에 저장하는 칸 묶음.
pub const GROUPS: [&str; 8] = ["cookies", "proxy", "rate_limit", "retries", "concurrency", "template", "overwrite", "extra_args"];

/// 글자를 치는 칸이 든 묶음. Enter 나 다른 곳 클릭 없이 화면을 떠나도 저장을 시도한다.
const TEXT_GROUPS: [&str; 5] = ["cookies", "proxy", "rate_limit", "template", "extra_args"];

pub fn group_fields(group: &str) -> &'static [&'static str] {
    match group {
        "cookies" => &["cookies", "cookie_browser", "cookie_file"],
        "proxy" => &["proxy"],
        "rate_limit" => &["rate_limit"],
        "retries" => &["retries"],
        "concurrency" => &["concurrency"],
        "template" => &["name_template", "custom_template"],
        "overwrite" => &["overwrite"],
        "extra_args" => &["extra_args"],
        _ => &[],
    }
}

/// 오류 키들. 화면은 이 이름으로 칸 아래 문구를 찾는다.
pub const ERROR_KEYS: [&str; 4] = ["cookie_file", "rate_limit", "custom_template", "extra_args"];

/// 오류 키 → 그 오류를 일으키는 칸. (`options::validate` 의 규칙과 같다)
fn error_causes(key: &str) -> &'static [&'static str] {
    match key {
        "rate_limit" => &["rate_limit"],
        "cookie_file" => &["cookies", "cookie_file"],
        "custom_template" => &["name_template", "custom_template"],
        "extra_args" => &["extra_args"],
        _ => &[],
    }
}

/// `src` 의 칸 몇 개를 `dst` 로. 글자 칸은 앞뒤 공백을 뗀다(파일 이름 규칙은 그대로).
fn copy_fields(dst: &mut AppSettings, src: &AppSettings, fields: &[&str]) {
    for field in fields {
        match *field {
            "cookies" => dst.cookies = src.cookies.clone(),
            "cookie_browser" => dst.cookie_browser = src.cookie_browser.clone(),
            "cookie_file" => dst.cookie_file = src.cookie_file.trim().to_string(),
            "proxy" => dst.proxy = src.proxy.trim().to_string(),
            "rate_limit" => dst.rate_limit = src.rate_limit.trim().to_string(),
            "retries" => dst.retries = src.retries,
            "name_template" => dst.name_template = src.name_template.clone(),
            "custom_template" => dst.custom_template = src.custom_template.clone(),
            "overwrite" => dst.overwrite = src.overwrite,
            "concurrency" => dst.concurrency = src.concurrency,
            "extra_args" => dst.extra_args = src.extra_args.trim().to_string(),
            _ => {}
        }
    }
}

/// 고급 옵션 칸을 저장하기 전에 할 일.
#[derive(Clone, Debug, PartialEq)]
pub enum ExtraStep {
    /// 저장된 값과 같다.
    Unchanged,
    /// yt-dlp 없이 판단할 수 있다(비웠거나 링크가 들어 있다). 곧바로 `commit` 한다.
    Local,
    /// 이 글을 yt-dlp 로 확인한 뒤 `apply_extra` 로 알려 준다.
    Ask(String),
}

pub struct SettingsForm {
    saved: AppSettings,
    view: AppSettings,
    errors: BTreeMap<&'static str, String>,
}

impl SettingsForm {
    pub fn new(saved: AppSettings) -> Self {
        Self { view: saved.clone(), saved, errors: BTreeMap::new() }
    }

    /// 지금 저장된 설정.
    pub fn saved(&self) -> &AppSettings {
        &self.saved
    }

    /// 칸에 보이는 값.
    pub fn view(&self) -> &AppSettings {
        &self.view
    }

    /// 화면의 칸 값을 받아 둔다.
    pub fn set_view(&mut self, view: AppSettings) {
        self.view = view;
    }

    /// 칸 아래 보일 오류 문구. 없으면 빈 글.
    pub fn error(&self, key: &str) -> &str {
        self.errors.get(key).map_or("", String::as_str)
    }

    pub fn clear_error(&mut self, key: &str) {
        self.errors.remove(key);
    }

    /// 묶음의 칸을 저장한다. 저장된 값이 실제로 바뀌었으면 true.
    /// 틀리면 저장하지 않고 오류를 남긴다. 이 묶음과 상관없는 오류는 건드리지 않는다.
    pub fn commit(&mut self, group: &str, extra: ExtraCheck) -> bool {
        let fields = group_fields(group);
        let mut cand = self.saved.clone();
        copy_fields(&mut cand, &self.view, fields);
        let (new, errors) = update_settings(&self.saved, cand, fields, extra);
        for key in ERROR_KEYS {
            if error_causes(key).iter().any(|f| fields.contains(f)) {
                match errors.get(key) {
                    Some(message) => self.errors.insert(key, message.clone()),
                    None => self.errors.remove(key),
                };
            }
        }
        let changed = new != self.saved;
        self.saved = new;
        changed
    }

    /// 다른 화면으로 가거나 창을 닫을 때. 맞는 글자는 저장하고, 나머지는 저장된 값으로 되돌린다.
    pub fn leave(&mut self, extra: ExtraCheck) -> bool {
        let mut changed = false;
        for group in TEXT_GROUPS {
            let fields = group_fields(group);
            let mut typed = self.saved.clone();
            copy_fields(&mut typed, &self.view, fields);
            if typed != self.saved {
                changed |= self.commit(group, extra);
            }
        }
        self.view = self.saved.clone();
        self.errors.clear();
        changed
    }

    pub fn extra_step(&self) -> ExtraStep {
        let text = self.view.extra_args.trim();
        if text == self.saved.extra_args {
            ExtraStep::Unchanged
        } else if text.is_empty() || extra_has_url(text) {
            ExtraStep::Local
        } else {
            ExtraStep::Ask(text.to_string())
        }
    }

    /// yt-dlp 가 `text` 를 확인한 결과를 반영한다. 저장된 값이 바뀌었으면 true.
    pub fn apply_extra(&mut self, text: &str, error: Option<String>) -> bool {
        match error {
            Some(message) => {
                // 그사이 칸을 고쳤으면 늦게 온 오류는 버린다
                if self.view.extra_args.trim() == text {
                    self.errors.insert("extra_args", message);
                }
                false
            }
            None => {
                let changed = self.saved.extra_args != text;
                // 화면을 떠나 칸이 저장된 값으로 되돌려져 있었으면 새 값으로 채운다
                if self.view.extra_args.trim() == self.saved.extra_args {
                    self.view.extra_args = text.to_string();
                }
                self.saved.extra_args = text.to_string();
                self.errors.remove("extra_args");
                changed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(_: &str) -> Option<String> {
        None
    }

    fn form() -> SettingsForm {
        SettingsForm::new(AppSettings::default())
    }

    /// 화면의 칸을 고친다.
    fn edit(form: &mut SettingsForm, f: impl FnOnce(&mut AppSettings)) {
        let mut view = form.view().clone();
        f(&mut view);
        form.set_view(view);
    }

    #[test]
    fn groups_cover_every_field_once() {
        let mut fields: Vec<&str> = GROUPS.iter().flat_map(|g| group_fields(g).iter().copied()).collect();
        fields.sort();
        let mut all = AppSettings::FIELDS.to_vec();
        all.sort();
        // name_template 과 custom_template 은 한 묶음, 쿠키 세 칸도 한 묶음이다
        assert_eq!(fields, all);
        assert!(group_fields("nope").is_empty());
    }

    #[test]
    fn checkbox_and_spin_save_immediately() {
        let mut f = form();
        edit(&mut f, |v| v.overwrite = true);
        assert!(f.commit("overwrite", &ok));
        assert!(f.saved().overwrite);
        edit(&mut f, |v| v.concurrency = 4);
        assert!(f.commit("concurrency", &ok));
        assert_eq!(f.saved().concurrency, 4);
        // 같은 값을 다시 저장하면 바뀐 게 없다
        assert!(!f.commit("concurrency", &ok));
    }

    #[test]
    fn bad_rate_shows_error_and_is_not_saved() {
        let mut f = form();
        edit(&mut f, |v| v.rate_limit = "fast".into());
        assert!(!f.commit("rate_limit", &ok));
        assert_eq!(f.error("rate_limit"), "속도 형식: 5M, 500K");
        assert_eq!(f.saved().rate_limit, "");
        edit(&mut f, |v| v.rate_limit = " 5M ".into());
        assert!(f.commit("rate_limit", &ok));
        assert_eq!(f.error("rate_limit"), "");
        assert_eq!(f.saved().rate_limit, "5M");
    }

    #[test]
    fn error_clears_when_reverted_to_saved() {
        let mut f = form();
        edit(&mut f, |v| v.rate_limit = "fast".into());
        f.commit("rate_limit", &ok);
        edit(&mut f, |v| v.rate_limit = "".into());
        assert!(!f.commit("rate_limit", &ok));
        assert_eq!(f.error("rate_limit"), "");
    }

    #[test]
    fn cookie_file_mode_needs_a_file() {
        let mut f = form();
        edit(&mut f, |v| v.cookies = "file".into());
        assert!(!f.commit("cookies", &ok));
        assert_eq!(f.error("cookie_file"), "쿠키 파일 없음");
        assert_eq!(f.saved().cookies, "none");
        // 화면에는 고른 그대로 남는다
        assert_eq!(f.view().cookies, "file");

        let file = std::env::temp_dir().join(format!("qm-settings-{}.txt", std::process::id()));
        std::fs::write(&file, "x").unwrap();
        edit(&mut f, |v| v.cookie_file = file.to_string_lossy().into_owned());
        assert!(f.commit("cookies", &ok));
        assert_eq!((f.saved().cookies.as_str(), f.error("cookie_file")), ("file", ""));
        assert_eq!(f.saved().cookie_file, file.to_string_lossy());
        std::fs::remove_file(file).unwrap();
    }

    #[test]
    fn custom_template_saved_with_name() {
        let mut f = form();
        edit(&mut f, |v| {
            v.custom_template = "%(id)s.%(ext)s".into();
            v.name_template = "custom".into();
        });
        assert!(f.commit("template", &ok));
        assert_eq!((f.saved().name_template.as_str(), f.saved().custom_template.as_str()), ("custom", "%(id)s.%(ext)s"));
        edit(&mut f, |v| v.custom_template = " ".into());
        assert!(!f.commit("template", &ok));
        assert_eq!(f.error("custom_template"), "파일 이름 규칙을 입력하세요");
    }

    #[test]
    fn other_groups_do_not_touch_unrelated_view_values() {
        // 프록시를 저장할 때 아직 저장 안 한 속도 제한까지 딸려 가면 안 된다
        let mut f = form();
        edit(&mut f, |v| {
            v.proxy = "http://p:1".into();
            v.rate_limit = "fast".into();
        });
        assert!(f.commit("proxy", &ok));
        assert_eq!((f.saved().proxy.as_str(), f.saved().rate_limit.as_str()), ("http://p:1", ""));
        assert_eq!(f.error("rate_limit"), "");
    }

    #[test]
    fn leave_saves_valid_pending_text() {
        let mut f = form();
        edit(&mut f, |v| v.proxy = "http://x:1".into()); // Enter 도, 다른 곳 클릭도 없이
        assert!(f.leave(&ok));
        assert_eq!(f.saved().proxy, "http://x:1");
        assert!(!f.leave(&ok));
    }

    #[test]
    fn leave_reverts_invalid_text_and_clears_errors() {
        let mut f = SettingsForm::new(AppSettings { rate_limit: "1M".into(), ..Default::default() });
        edit(&mut f, |v| v.rate_limit = "fast".into());
        f.commit("rate_limit", &ok);
        assert!(!f.leave(&ok));
        assert_eq!((f.view().rate_limit.as_str(), f.saved().rate_limit.as_str(), f.error("rate_limit")), ("1M", "1M", ""));
    }

    #[test]
    fn leave_restores_unsaved_cookie_mode() {
        let mut f = form();
        edit(&mut f, |v| v.cookies = "file".into());
        f.commit("cookies", &ok);
        f.leave(&ok);
        assert_eq!((f.view().cookies.as_str(), f.error("cookie_file")), ("none", ""));
    }

    #[test]
    fn stale_saved_error_hidden_until_edited() {
        // 이미 틀린 저장값은 건드리기 전에는 보여 주지 않는다
        let f = SettingsForm::new(AppSettings { rate_limit: "fast".into(), ..Default::default() });
        assert_eq!(f.error("rate_limit"), "");
    }

    #[test]
    fn extra_args_are_checked_before_saving() {
        let bad = |text: &str| text.contains("--nope").then(|| "고급 옵션 오류: no such option: --nope".to_string());
        let mut f = form();
        edit(&mut f, |v| v.extra_args = " --nope ".into());
        assert!(!f.commit("extra_args", &bad));
        assert!(f.error("extra_args").contains("--nope"));
        edit(&mut f, |v| v.extra_args = "--no-mtime".into());
        assert!(f.commit("extra_args", &bad));
        assert_eq!((f.saved().extra_args.as_str(), f.error("extra_args")), ("--no-mtime", ""));
    }

    #[test]
    fn extra_args_step_decides_whether_yt_dlp_is_needed() {
        let mut f = form();
        assert_eq!(f.extra_step(), ExtraStep::Unchanged);
        edit(&mut f, |v| v.extra_args = "https://x.com/v".into());
        // 링크는 yt-dlp 를 부르지 않고 막는다
        assert_eq!(f.extra_step(), ExtraStep::Local);
        edit(&mut f, |v| v.extra_args = " --no-mtime ".into());
        assert_eq!(f.extra_step(), ExtraStep::Ask("--no-mtime".into()));
        edit(&mut f, |v| v.extra_args = "  ".into());
        assert_eq!(f.extra_step(), ExtraStep::Unchanged);
    }

    #[test]
    fn late_extra_check_result_is_applied() {
        // 통과: 그사이 화면을 떠나 칸이 되돌려졌어도 저장한다
        let mut f = form();
        edit(&mut f, |v| v.extra_args = "--no-mtime".into());
        f.leave(&|_| None); // 고급 인자는 아직 확인 중이라 떠날 때는 저장하지 않는 쪽으로 부른다
        let mut f = form();
        edit(&mut f, |v| v.extra_args = "--no-mtime".into());
        assert!(f.apply_extra("--no-mtime", None));
        assert_eq!((f.saved().extra_args.as_str(), f.view().extra_args.as_str()), ("--no-mtime", "--no-mtime"));

        // 실패: 칸이 그 글 그대로일 때만 오류를 보인다
        let mut f = form();
        edit(&mut f, |v| v.extra_args = "--nope".into());
        assert!(!f.apply_extra("--nope", Some("고급 옵션 오류: x".into())));
        assert_eq!((f.saved().extra_args.as_str(), f.error("extra_args")), ("", "고급 옵션 오류: x"));
        edit(&mut f, |v| v.extra_args = "--other".into());
        f.clear_error("extra_args");
        assert!(!f.apply_extra("--nope", Some("늦게 온 오류".into())));
        assert_eq!(f.error("extra_args"), "");
    }
}
