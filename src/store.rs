//! 설정 저장소. 1.x(QSettings)가 쓰던 레지스트리 위치와 키 이름을 그대로 쓴다.
//!
//! 키는 `folder`, `options/kind` 처럼 `/` 로 하위 키를 나눈다.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use winreg::enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

pub trait KeyValue {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
    fn remove(&mut self, key: &str);
    /// 들어 있는 모든 키.
    fn keys(&self) -> Vec<String>;
}

/// `group/` 아래 값들을 이름 → 값으로.
pub fn read_group(kv: &dyn KeyValue, group: &str) -> HashMap<String, String> {
    let prefix = format!("{group}/");
    kv.keys()
        .into_iter()
        .filter_map(|key| {
            let name = key.strip_prefix(&prefix)?.to_string();
            Some((name, kv.get(&key)?))
        })
        .collect()
}

pub fn write_group(kv: &mut dyn KeyValue, group: &str, values: &BTreeMap<String, String>) {
    for (name, value) in values {
        kv.set(&format!("{group}/{name}"), value);
    }
}

/// 메모리에만 두는 저장소. 복사본끼리 같은 값을 본다.
#[derive(Clone, Default)]
pub struct MemoryStore {
    values: Arc<Mutex<BTreeMap<String, String>>>,
}

impl KeyValue for MemoryStore {
    fn get(&self, key: &str) -> Option<String> {
        self.values.lock().unwrap().get(key).cloned()
    }
    fn set(&mut self, key: &str, value: &str) {
        self.values.lock().unwrap().insert(key.to_string(), value.to_string());
    }
    fn remove(&mut self, key: &str) {
        self.values.lock().unwrap().remove(key);
    }
    fn keys(&self) -> Vec<String> {
        self.values.lock().unwrap().keys().cloned().collect()
    }
}

/// `HKEY_CURRENT_USER\<path>`. 읽고 쓸 때마다 키를 연다.
#[derive(Clone, Debug)]
pub struct Registry {
    path: String,
}

/// `options/kind` → (`<path>\options`, `kind`)
fn split(path: &str, key: &str) -> (String, String) {
    match key.rsplit_once('/') {
        Some((group, name)) => (format!("{path}\\{}", group.replace('/', "\\")), name.to_string()),
        None => (path.to_string(), key.to_string()),
    }
}

/// QSettings 는 글자를 `REG_SZ`, 정수를 `REG_DWORD`/`REG_QWORD` 로 쓴다. 다른 형은 없는 것으로 본다.
fn read_value(key: &RegKey, name: &str) -> Option<String> {
    let raw = key.get_raw_value(name).ok()?;
    match raw.vtype {
        RegType::REG_SZ | RegType::REG_EXPAND_SZ => key.get_value::<String, _>(name).ok(),
        RegType::REG_DWORD => key.get_value::<u32, _>(name).ok().map(|n| n.to_string()),
        RegType::REG_QWORD => key.get_value::<u64, _>(name).ok().map(|n| n.to_string()),
        _ => None,
    }
}

impl Registry {
    pub fn open(path: &str) -> Self {
        Self { path: path.to_string() }
    }
}

impl KeyValue for Registry {
    fn get(&self, key: &str) -> Option<String> {
        let (subkey, name) = split(&self.path, key);
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(subkey, KEY_READ).ok()?;
        read_value(&key, &name)
    }

    fn set(&mut self, key: &str, value: &str) {
        let (subkey, name) = split(&self.path, key);
        if let Ok((key, _)) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(subkey) {
            let _ = key.set_value(name, &value);
        }
    }

    fn remove(&mut self, key: &str) {
        let (subkey, name) = split(&self.path, key);
        if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(subkey, KEY_SET_VALUE) {
            let _ = key.delete_value(name);
        }
    }

    fn keys(&self) -> Vec<String> {
        let Ok(root) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(&self.path, KEY_READ) else {
            return Vec::new();
        };
        let mut keys: Vec<String> = root.enum_values().flatten().map(|(name, _)| name).collect();
        for group in root.enum_keys().flatten() {
            if let Ok(sub) = root.open_subkey_with_flags(&group, KEY_READ) {
                keys.extend(sub.enum_values().flatten().map(|(name, _)| format!("{group}/{name}")));
            }
        }
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winreg::enums::{RegType, HKEY_CURRENT_USER};
    use winreg::{RegKey, RegValue};

    #[test]
    fn groups_round_trip() {
        let mut kv = MemoryStore::default();
        let values = BTreeMap::from([("kind".to_string(), "audio".to_string()), ("subs".to_string(), "true".to_string())]);
        write_group(&mut kv, "options", &values);
        kv.set("folder", "D:\\받기");
        kv.set("settings/proxy", "p");
        assert_eq!(kv.get("options/kind").as_deref(), Some("audio"));
        let read = read_group(&kv, "options");
        assert_eq!(read.len(), 2);
        assert_eq!(read["subs"], "true");
        assert!(read_group(&kv, "nothing").is_empty());
        let mut keys = kv.keys();
        keys.sort();
        assert_eq!(keys, ["folder", "options/kind", "options/subs", "settings/proxy"]);
        kv.remove("folder");
        assert_eq!(kv.get("folder"), None);
    }

    #[test]
    fn memory_store_clones_share_values() {
        let a = MemoryStore::default();
        let mut b = a.clone();
        b.set("x", "1");
        assert_eq!(a.get("x").as_deref(), Some("1"));
    }

    /// 시험이 끝나면 키를 지운다.
    struct TestKey(String);

    impl Drop for TestKey {
        fn drop(&mut self) {
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
        }
    }

    #[test]
    fn registry_round_trip() {
        let key = TestKey(format!(r"Software\QuickMedia-test-{}-a", std::process::id()));
        let mut reg = Registry::open(&key.0);
        assert_eq!(reg.get("folder"), None);
        assert!(reg.keys().is_empty());
        reg.set("folder", "D:\\받기 🎵");
        reg.set("options/kind", "audio");
        reg.set("options/subs", "true");
        assert_eq!(reg.get("folder").as_deref(), Some("D:\\받기 🎵"));
        assert_eq!(read_group(&reg, "options").len(), 2);
        let mut keys = reg.keys();
        keys.sort();
        assert_eq!(keys, ["folder", "options/kind", "options/subs"]);
        reg.remove("options/kind");
        reg.remove("options/nope");
        assert_eq!(reg.get("options/kind"), None);
        // 같은 곳을 가리키는 다른 객체도 같은 값을 본다
        assert_eq!(Registry::open(&key.0).get("options/subs").as_deref(), Some("true"));
    }

    #[test]
    fn registry_reads_numbers_and_ignores_other_types() {
        let key = TestKey(format!(r"Software\QuickMedia-test-{}-b", std::process::id()));
        let (raw, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(format!(r"{}\settings", key.0)).unwrap();
        raw.set_value("retries", &3u32).unwrap();
        raw.set_value("big", &5_000_000_000u64).unwrap();
        raw.set_raw_value("blob", &RegValue { bytes: vec![1, 2, 3], vtype: RegType::REG_BINARY }).unwrap();
        let reg = Registry::open(&key.0);
        assert_eq!(reg.get("settings/retries").as_deref(), Some("3"));
        assert_eq!(reg.get("settings/big").as_deref(), Some("5000000000"));
        assert_eq!(reg.get("settings/blob"), None);
        let group = read_group(&reg, "settings");
        assert_eq!(group.len(), 2);
        assert_eq!(crate::options::settings_from_map(&group).retries, 3);
    }
}
