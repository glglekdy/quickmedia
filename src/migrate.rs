//! 유튜브 다운로더(v1.0.1 이하)에서 넘어온 사용자 정리.
//!
//! - 옛 설정(저장 위치, 형식)을 새 이름의 설정으로 한 번 옮긴다.
//! - 옛 앱이 자기 자리에 덮어쓴 설치 파일 사본(YoutubeDownloader.exe)을 지운다.
//!   설치 파일이 끝날 때 그 경로를 설정의 `legacy_exe` 에 적어 둔다.
//!
//! 여기서 나는 실패는 앱 시작을 막지 않는다. 다음 실행에 다시 시도한다.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::app_info::LEGACY_EXE;
use crate::store::KeyValue;

/// 새 설정에 쓸 값. 이미 옮겼으면 비어 있다.
pub fn plan_settings_migration(old: &HashMap<String, String>, new_keys: &HashSet<String>) -> HashMap<String, String> {
    if new_keys.contains("migrated") {
        return HashMap::new();
    }
    let mut values = old.clone();
    values.insert("migrated".into(), "true".into());
    values
}

pub fn is_legacy_exe(path: &str) -> bool {
    let name = path.rsplit(['\\', '/']).next().unwrap_or("");
    name.eq_ignore_ascii_case(LEGACY_EXE)
}

/// 정리가 끝났으면 true. 설치 파일이 아직 잡고 있으면 잠깐씩 다시 시도한다.
pub fn delete_legacy(path: &str, attempts: u32, delay: Duration) -> bool {
    if !is_legacy_exe(path) {
        return true;
    }
    for i in 0..attempts {
        match std::fs::remove_file(Path::new(path)) {
            Ok(()) => return true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
            Err(_) if i + 1 < attempts => std::thread::sleep(delay),
            Err(_) => {}
        }
    }
    false
}

/// 옛 설정을 한 번 옮기고, 지울 옛 exe 가 적혀 있으면 뒤에서 지운다(그 스레드를 돌려준다).
pub fn run<S>(new: &mut S, old: &dyn KeyValue) -> Option<JoinHandle<()>>
where
    S: KeyValue + Clone + Send + 'static,
{
    let old_values: HashMap<String, String> = old.keys().into_iter().filter_map(|k| Some((k.clone(), old.get(&k)?))).collect();
    let new_keys: HashSet<String> = new.keys().into_iter().collect();
    for (key, value) in plan_settings_migration(&old_values, &new_keys) {
        new.set(&key, &value);
    }
    let path = new.get("legacy_exe")?;
    let mut store = new.clone();
    Some(std::thread::spawn(move || {
        if delete_legacy(&path, 15, Duration::from_secs(2)) {
            store.remove("legacy_exe");
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;
    use std::os::windows::fs::OpenOptionsExt;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("qm-migrate-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn plan_copies_old_keys_once() {
        let planned = plan_settings_migration(&map(&[("folder", "D:/v"), ("format", "mp3")]), &HashSet::new());
        assert_eq!(planned, map(&[("folder", "D:/v"), ("format", "mp3"), ("migrated", "true")]));
        let keys: HashSet<String> = ["migrated", "folder"].iter().map(|s| s.to_string()).collect();
        assert!(plan_settings_migration(&map(&[("folder", "D:/v")]), &keys).is_empty());
    }

    #[test]
    fn plan_with_empty_old_marks_migrated() {
        assert_eq!(plan_settings_migration(&HashMap::new(), &HashSet::new()), map(&[("migrated", "true")]));
    }

    #[test]
    fn legacy_exe_name() {
        assert!(is_legacy_exe(r"C:\Users\a\Downloads\YoutubeDownloader.exe"));
        assert!(is_legacy_exe("c:/x/youtubedownloader.EXE"));
        assert!(!is_legacy_exe(r"C:\x\QuickMedia-Setup-2.0.0.exe"));
    }

    #[test]
    fn delete_legacy_removes_file() {
        let f = temp("rm").join("YoutubeDownloader.exe");
        std::fs::write(&f, "x").unwrap();
        assert!(delete_legacy(&f.to_string_lossy(), 1, Duration::ZERO));
        assert!(!f.exists());
    }

    #[test]
    fn delete_legacy_skips_other_names() {
        let f = temp("other").join("Other.exe");
        std::fs::write(&f, "x").unwrap();
        assert!(delete_legacy(&f.to_string_lossy(), 1, Duration::ZERO));
        assert!(f.exists());
    }

    #[test]
    fn delete_legacy_missing_file_is_done() {
        let f = temp("missing").join("YoutubeDownloader.exe");
        assert!(delete_legacy(&f.to_string_lossy(), 1, Duration::ZERO));
    }

    #[test]
    fn delete_legacy_retries_while_locked() {
        let f = temp("locked").join("YoutubeDownloader.exe");
        std::fs::write(&f, "x").unwrap();
        // 공유 없이 열면 Windows 에서는 지울 수 없다
        let handle = std::fs::OpenOptions::new().read(true).share_mode(0).open(&f).unwrap();
        assert!(!delete_legacy(&f.to_string_lossy(), 1, Duration::ZERO));
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            drop(handle);
        });
        assert!(delete_legacy(&f.to_string_lossy(), 10, Duration::from_millis(100)));
        assert!(!f.exists());
    }

    #[test]
    fn run_copies_settings_and_cleans_up_pending_exe() {
        let f = temp("run").join("YoutubeDownloader.exe");
        std::fs::write(&f, "x").unwrap();
        let mut old = MemoryStore::default();
        old.set("folder", "D:/v");
        old.set("format", "mp3");
        let mut new = MemoryStore::default();
        new.set("legacy_exe", &f.to_string_lossy());
        run(&mut new, &old).expect("지울 파일이 있으니 스레드가 돈다").join().unwrap();
        assert_eq!(new.get("folder").as_deref(), Some("D:/v"));
        assert_eq!(new.get("format").as_deref(), Some("mp3"));
        assert_eq!(new.get("migrated").as_deref(), Some("true"));
        assert!(!f.exists());
        assert_eq!(new.get("legacy_exe"), None);

        // 두 번째에는 옮기지 않는다. 그 사이 바꾼 값이 덮이면 안 된다.
        new.set("folder", "E:/new");
        assert!(run(&mut new, &old).is_none());
        assert_eq!(new.get("folder").as_deref(), Some("E:/new"));
    }
}
