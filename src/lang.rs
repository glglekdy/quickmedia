//! 화면에 보이는 글의 언어. 한국어가 기본이고 영어를 고를 수 있다.
//! 글은 쓰는 자리에서 두 언어를 나란히 적는다: `tr("저장", "Save")`.

use std::sync::atomic::{AtomicBool, Ordering};

static ENGLISH: AtomicBool = AtomicBool::new(false);

/// 레지스트리 `ui/lang` 에 두는 값.
pub const STORE_KEY: &str = "ui/lang";

pub fn set_english(on: bool) {
    ENGLISH.store(on, Ordering::Relaxed);
}

pub fn english() -> bool {
    ENGLISH.load(Ordering::Relaxed)
}

/// 지금 언어의 글.
pub fn tr(ko: &'static str, en: &'static str) -> &'static str {
    if english() {
        en
    } else {
        ko
    }
}

/// `format!` 의 두 언어판: `trf!("영상 {n}개", "{n} videos")`.
#[macro_export]
macro_rules! trf {
    ($ko:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        if $crate::lang::english() {
            format!($en $(, $arg)*)
        } else {
            format!($ko $(, $arg)*)
        }
    };
}

/// 저장된 값에서 영어를 쓸지. 고른 적이 없으면 Windows 표시 언어를 따른다.
pub fn english_from(saved: Option<&str>, system_is_korean: bool) -> bool {
    match saved {
        Some("en") => true,
        Some("ko") => false,
        _ => !system_is_korean,
    }
}

pub fn code(english: bool) -> &'static str {
    if english {
        "en"
    } else {
        "ko"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_choice_wins_over_the_system_language() {
        assert!(english_from(Some("en"), true));
        assert!(!english_from(Some("ko"), false));
    }

    #[test]
    fn without_a_choice_the_system_language_decides() {
        assert!(!english_from(None, true));
        assert!(english_from(None, false));
        assert!(english_from(Some("??"), false));
    }

    #[test]
    fn korean_is_the_default() {
        assert_eq!(tr("저장", "Save"), "저장");
        let n = 3;
        assert_eq!(trf!("영상 {n}개", "{n} videos"), "영상 3개");
        assert_eq!(code(false), "ko");
    }
}
