//! 사용자가 저장한 프리셋: 세부 옵션 한 벌에 이름을 붙인 것.
//! 레지스트리 `presets/list` 에 JSON 으로 둔다.

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::options::{options_from_map, options_to_map, DownloadOptions};

pub const STORE_KEY: &str = "presets/list";
pub const MAX_PRESETS: usize = 20;
pub const MAX_NAME_CHARS: usize = 20;

#[derive(Clone, Debug, PartialEq)]
pub struct UserPreset {
    pub name: String,
    pub opts: DownloadOptions,
}

/// 저장된 글에서 프리셋들. 읽지 못하는 항목은 건너뛴다.
pub fn parse(text: &str) -> Vec<UserPreset> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(text) else { return Vec::new() };
    items
        .iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.trim().to_string();
            let fields = item.get("options")?.as_object()?;
            let map: HashMap<String, String> =
                fields.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect();
            (!name.is_empty()).then(|| UserPreset { name, opts: options_from_map(&map, None) })
        })
        .take(MAX_PRESETS)
        .collect()
}

pub fn to_json(list: &[UserPreset]) -> String {
    let items: Vec<Value> = list.iter().map(|p| json!({ "name": p.name, "options": options_to_map(&p.opts) })).collect();
    Value::Array(items).to_string()
}

/// 지금 값을 `name` 으로 저장한다. 같은 이름이 있으면 그 자리에 덮어쓴다.
pub fn save(list: &mut Vec<UserPreset>, name: &str, opts: &DownloadOptions) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(crate::lang::tr("프리셋 이름을 입력하세요", "Enter a preset name").to_string());
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(crate::trf!("이름은 최대 {MAX_NAME_CHARS}자", "Name can be up to {MAX_NAME_CHARS} characters"));
    }
    let preset = UserPreset { name: name.to_string(), opts: opts.clone() };
    if let Some(existing) = list.iter_mut().find(|p| p.name == name) {
        *existing = preset;
        return Ok(());
    }
    if list.len() >= MAX_PRESETS {
        return Err(crate::trf!("프리셋은 최대 {MAX_PRESETS}개", "Up to {MAX_PRESETS} presets"));
    }
    list.push(preset);
    Ok(())
}

/// 지금 값과 같은 프리셋의 자리.
pub fn matching(list: &[UserPreset], opts: &DownloadOptions) -> Option<usize> {
    list.iter().position(|p| p.opts == *opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::apply_preset;

    fn subs() -> DownloadOptions {
        let mut o = apply_preset(&DownloadOptions::default(), "720p");
        o.subs = true;
        o.sub_langs = "en".into();
        o
    }

    #[test]
    fn saved_presets_survive_a_round_trip() {
        let mut list = Vec::new();
        save(&mut list, "  영어 자막 720p ", &subs()).unwrap();
        save(&mut list, "음원", &apply_preset(&DownloadOptions::default(), "mp3")).unwrap();
        assert_eq!(list[0].name, "영어 자막 720p");
        let again = parse(&to_json(&list));
        assert_eq!(again, list);
        assert!(again[0].opts.subs && again[0].opts.sub_langs == "en");
    }

    #[test]
    fn same_name_overwrites_in_place() {
        let mut list = Vec::new();
        save(&mut list, "내 것", &DownloadOptions::default()).unwrap();
        save(&mut list, "다른 것", &DownloadOptions::default()).unwrap();
        save(&mut list, "내 것", &subs()).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], UserPreset { name: "내 것".into(), opts: subs() });
    }

    #[test]
    fn names_are_checked() {
        let mut list = Vec::new();
        assert_eq!(save(&mut list, "   ", &subs()).unwrap_err(), "프리셋 이름을 입력하세요");
        assert!(save(&mut list, &"가".repeat(MAX_NAME_CHARS + 1), &subs()).unwrap_err().contains("20자"));
        save(&mut list, &"가".repeat(MAX_NAME_CHARS), &subs()).unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn there_is_a_limit_but_overwriting_still_works() {
        let mut list = Vec::new();
        for i in 0..MAX_PRESETS {
            save(&mut list, &format!("p{i}"), &subs()).unwrap();
        }
        assert!(save(&mut list, "하나 더", &subs()).unwrap_err().contains("20개"));
        save(&mut list, "p3", &DownloadOptions::default()).unwrap();
        assert_eq!(list.len(), MAX_PRESETS);
    }

    #[test]
    fn matching_finds_the_preset_with_the_same_values() {
        let mut list = Vec::new();
        save(&mut list, "기본", &DownloadOptions::default()).unwrap();
        save(&mut list, "자막", &subs()).unwrap();
        assert_eq!(matching(&list, &subs()), Some(1));
        let mut other = subs();
        other.sub_langs = "ko".into();
        assert_eq!(matching(&list, &other), None);
    }

    #[test]
    fn broken_storage_gives_what_can_be_read() {
        assert!(parse("").is_empty());
        assert!(parse("{not json").is_empty());
        assert!(parse(r#"{"name":"x"}"#).is_empty());
        let text = r#"[{"name":"","options":{}},{"name":"ok","options":{"kind":"audio"}},{"options":{}},7]"#;
        let list = parse(text);
        assert_eq!(list.len(), 1);
        assert_eq!((list[0].name.as_str(), list[0].opts.kind.as_str()), ("ok", "audio"));
    }
}
