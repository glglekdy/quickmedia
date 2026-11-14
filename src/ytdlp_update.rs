//! yt-dlp 갱신. 사이트가 바뀌어 받아지지 않을 때 앱 릴리즈를 기다리지 않고 고친다.
//!
//! 설치 폴더의 `yt-dlp.exe` 는 건드리지 않고, 갱신본을 `%APPDATA%\QuickMedia\bin\yt-dlp-<버전>.exe` 로
//! 받는다. 앱을 켤 때 둘 중 버전이 높은 것을 쓴다. 이름에 버전이 있어 받는 중에도 갱신할 수 있다.

use std::path::{Path, PathBuf};

use crate::store::KeyValue;
use crate::updater::{self, is_newer, Release};
use crate::ytdlp::{ytdlp_version, KillGroup};

const REPO: &str = "yt-dlp/yt-dlp";
const ASSET: &str = "yt-dlp.exe";

/// 쓸 수 있는 yt-dlp 하나.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub path: PathBuf,
    pub version: String,
}

pub fn update_file_name(version: &str) -> String {
    format!("yt-dlp-{version}.exe")
}

/// 갱신본 파일 이름에서 버전. 갱신본이 아니면 `None`.
pub fn version_of_file(name: &str) -> Option<String> {
    let version = name.strip_prefix("yt-dlp-")?.strip_suffix(".exe")?;
    let valid = !version.is_empty() && version.chars().all(|c| c.is_ascii_digit() || c == '.');
    valid.then(|| version.to_string())
}

/// 버전이 가장 높은 것과 지울 갱신본들. 같으면 설치 폴더 것(`bundled`)을 쓴다.
pub fn choose(bundled: Option<Candidate>, updates: Vec<Candidate>) -> (Option<Candidate>, Vec<PathBuf>) {
    let mut best: Option<Candidate> = None;
    for update in &updates {
        if best.as_ref().is_none_or(|b| is_newer(&update.version, &b.version)) {
            best = Some(update.clone());
        }
    }
    let chosen = match (bundled, best) {
        (Some(b), Some(u)) if is_newer(&u.version, &b.version) => Some(u),
        (Some(b), _) => Some(b),
        (None, u) => u,
    };
    let remove = updates.into_iter().map(|u| u.path).filter(|p| chosen.as_ref().is_none_or(|c| c.path != *p)).collect();
    (chosen, remove)
}

/// 설치 폴더 exe 의 버전. 실행해서 알아낸 값을 크기·수정 시각과 함께 기억해 다음부터는 실행하지 않는다.
fn bundled(install_dir: &Path, kv: &mut dyn KeyValue, group: &KillGroup) -> Option<Candidate> {
    let path = install_dir.join(ASSET);
    let meta = path.metadata().ok().filter(|m| m.is_file())?;
    let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
    let stamp = format!("{}-{modified}", meta.len());
    if kv.get("ytdlp/bundled_stamp").as_deref() == Some(&stamp) {
        if let Some(version) = kv.get("ytdlp/bundled_version").filter(|v| !v.is_empty()) {
            return Some(Candidate { path, version });
        }
    }
    let version = ytdlp_version(&path, group)?;
    kv.set("ytdlp/bundled_stamp", &stamp);
    kv.set("ytdlp/bundled_version", &version);
    Some(Candidate { path, version })
}

/// 앱을 켤 때: 쓸 yt-dlp 를 고르고, 낡은 갱신본과 받다 만 파일을 지운다.
pub fn locate(install_dir: &Path, bin_dir: &Path, kv: &mut dyn KeyValue, group: &KillGroup) -> Option<Candidate> {
    let mut updates = Vec::new();
    for entry in std::fs::read_dir(bin_dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(version) = version_of_file(&name) {
            updates.push(Candidate { path: entry.path(), version });
        } else if name.starts_with("yt-dlp-") && name.ends_with(".part") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    let (chosen, remove) = choose(bundled(install_dir, kv, group), updates);
    for path in remove {
        // 지금 받는 중인 작업이 쓰고 있으면 못 지운다. 다음에 켤 때 다시 지운다.
        let _ = std::fs::remove_file(path);
    }
    chosen
}

/// yt-dlp 의 최신 릴리즈가 `current` 보다 새것이면 돌려준다.
pub fn fetch_latest(current: &str) -> Result<Option<Release>, String> {
    updater::fetch_release(REPO, current, &|name| name == ASSET)
}

/// 갱신본을 받아 확인하고 `bin_dir` 에 둔다.
pub fn install(r: &Release, bin_dir: &Path, group: &KillGroup, progress: &dyn Fn(u64, u64)) -> Result<Candidate, String> {
    let path = bin_dir.join(update_file_name(&r.version));
    updater::download_to(r, &path, progress)?;
    // 받은 파일이 실제로 도는지 본 뒤에야 쓴다
    if ytdlp_version(&path, group).as_deref() != Some(r.version.as_str()) {
        let _ = std::fs::remove_file(&path);
        return Err(crate::lang::tr("yt-dlp 실행 확인 실패. 다시 시도하세요.", "yt-dlp didn't run. Try again.").to_string());
    }
    Ok(Candidate { path, version: r.version.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    fn cand(path: &str, version: &str) -> Candidate {
        Candidate { path: path.into(), version: version.into() }
    }

    #[test]
    fn newer_update_wins_and_older_updates_are_removed() {
        let b = cand("app/yt-dlp.exe", "2026.10.01");
        let u = vec![cand("bin/yt-dlp-2026.09.01.exe", "2026.09.01"), cand("bin/yt-dlp-2026.11.05.exe", "2026.11.05")];
        let (chosen, remove) = choose(Some(b), u);
        assert_eq!(chosen.unwrap().version, "2026.11.05");
        assert_eq!(remove, vec![PathBuf::from("bin/yt-dlp-2026.09.01.exe")]);
    }

    #[test]
    fn bundled_wins_ties_and_after_app_update() {
        let u = || vec![cand("bin/a.exe", "2026.10.01"), cand("bin/b.exe", "2026.09.01")];
        let (chosen, remove) = choose(Some(cand("app/yt-dlp.exe", "2026.10.01")), u());
        assert_eq!(chosen.unwrap().path, PathBuf::from("app/yt-dlp.exe"));
        assert_eq!(remove.len(), 2);
        let (chosen, remove) = choose(Some(cand("app/yt-dlp.exe", "2026.12.01")), u());
        assert_eq!(chosen.unwrap().version, "2026.12.01");
        assert_eq!(remove.len(), 2);
    }

    #[test]
    fn update_alone_is_used_and_nothing_gives_none() {
        let (chosen, remove) = choose(None, vec![cand("bin/a.exe", "2026.10.01")]);
        assert_eq!(chosen.unwrap().path, PathBuf::from("bin/a.exe"));
        assert!(remove.is_empty());
        assert!(choose(None, vec![]).0.is_none());
        assert_eq!(choose(Some(cand("app/yt-dlp.exe", "1")), vec![]).0.unwrap().version, "1");
    }

    #[test]
    fn file_name_round_trip() {
        assert_eq!(update_file_name("2026.11.05"), "yt-dlp-2026.11.05.exe");
        assert_eq!(version_of_file(&update_file_name("2026.11.05")), Some("2026.11.05".into()));
        for name in ["yt-dlp.exe", "yt-dlp-2026.11.05.exe.part", "yt-dlp-.exe", "yt-dlp-abc.exe", "other-2026.1.1.exe", "yt-dlp-1.2.txt"] {
            assert_eq!(version_of_file(name), None, "{name}");
        }
    }

    #[test]
    fn locate_picks_newest_update_and_cleans_bin() {
        let root = std::env::temp_dir().join(format!("qm-ytdlp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (install, bin) = (root.join("app"), root.join("bin"));
        std::fs::create_dir_all(&bin).unwrap();
        for name in ["yt-dlp-2026.09.01.exe", "yt-dlp-2026.11.05.exe", "yt-dlp-2026.12.01.exe.part", "notes.txt"] {
            std::fs::write(bin.join(name), "x").unwrap();
        }
        let group = KillGroup::new().unwrap();
        let mut kv = MemoryStore::default();
        // 설치 폴더에 yt-dlp 가 없어도(개발 중) 갱신본을 쓴다
        let found = locate(&install, &bin, &mut kv, &group).unwrap();
        assert_eq!(found, Candidate { path: bin.join("yt-dlp-2026.11.05.exe"), version: "2026.11.05".into() });
        let mut left: Vec<String> =
            std::fs::read_dir(&bin).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        assert_eq!(left, ["notes.txt", "yt-dlp-2026.11.05.exe"]);
        // 아무것도 없으면 None. bin 폴더가 없어도 된다.
        assert_eq!(locate(&install, &root.join("nope"), &mut kv, &group), None);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
