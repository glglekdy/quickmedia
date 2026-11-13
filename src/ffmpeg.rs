//! ffmpeg 찾기와 받기. 설치 파일에는 ffmpeg 가 들어 있지 않다.
//!
//! 앱을 켤 때 차례로 찾는다:
//! 1. 설치 폴더(1.x 에서 업데이트한 PC 에는 남아 있다)와 전에 받아 둔 `%APPDATA%\QuickMedia\bin`
//! 2. PATH 에 있는 것(PC 에 따로 설치한 ffmpeg). 실제로 돌고 너무 낡지 않았을 때만 쓴다.
//!
//! 어디에도 없으면 앱 릴리즈에 함께 올린 `ffmpeg-win64.zip` 을 받아 `bin` 에 푼다.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::app_info::REPO;
use crate::updater::{self, Release};
use crate::ytdlp::process::run;
use crate::ytdlp::KillGroup;

pub const EXE: &str = "ffmpeg.exe";
/// 앱 릴리즈에 함께 올리는 자산. ffmpeg.exe 하나만 들어 있다.
pub const ASSET: &str = "ffmpeg-win64.zip";
/// 릴리즈에 자산이 없을 때 받는 곳. 빌드가 쓰는 것과 같은 묶음이라 크다(실행 파일 셋).
pub const FALLBACK_URL: &str = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
/// 이보다 낮은 판은 yt-dlp 가 쓰는 기능이 빠져 있다.
const MIN_MAJOR: u32 = 4;

/// `ffmpeg -version` 출력을 보고 쓸 만한지. 날짜·git 이름이 붙은 빌드는 새것으로 본다.
pub fn version_ok(output: &str) -> bool {
    let Some(rest) = output.trim_start().strip_prefix("ffmpeg version ") else { return false };
    let word = rest.split_whitespace().next().unwrap_or("");
    let word = word.strip_prefix('n').unwrap_or(word);
    let Some((major, _)) = word.split_once('.') else { return !word.is_empty() };
    match major.parse::<u32>() {
        Ok(major) => major >= MIN_MAJOR,
        Err(_) => true,
    }
}

/// PATH 에서 `ffmpeg.exe` 가 든 폴더들. 앞에 있는 것이 먼저.
pub fn dirs_on_path(path: &OsStr) -> Vec<PathBuf> {
    std::env::split_paths(path).filter(|dir| !dir.as_os_str().is_empty() && dir.join(EXE).is_file()).collect()
}

/// 실제로 돌고 너무 낡지 않았는지.
pub fn works(exe: &Path, group: &KillGroup) -> bool {
    let mut cmd = Command::new(exe);
    cmd.arg("-version");
    match run(group, &mut cmd, Some(Duration::from_secs(10))) {
        Ok(Some(out)) => out.success && version_ok(&out.stdout),
        _ => false,
    }
}

/// 쓸 ffmpeg 가 든 폴더. `own` 은 앱이 둔 곳(설치 폴더, 받아 둔 곳)이라 있기만 하면 쓴다.
pub fn locate(own: &[PathBuf], path: Option<&OsStr>, group: &KillGroup) -> Option<PathBuf> {
    if let Some(dir) = own.iter().find(|dir| dir.join(EXE).is_file()) {
        return Some(dir.clone());
    }
    dirs_on_path(path?).into_iter().find(|dir| works(&dir.join(EXE), group))
}

/// 압축 파일에서 `ffmpeg.exe` 만 꺼내 `target` 에 쓴다. 묶음 안 어느 폴더에 있든 찾는다.
pub fn extract(zip_path: &Path, target: &Path) -> Result<(), String> {
    let broken = || "ffmpeg 압축 해제 실패. 다시 시도하세요.".to_string();
    let file = std::fs::File::open(zip_path).map_err(|_| broken())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| broken())?;
    let is_ffmpeg = |name: &str| name.rsplit(['/', '\\']).next().is_some_and(|leaf| leaf.eq_ignore_ascii_case(EXE));
    let index = (0..archive.len())
        .find(|&i| archive.by_index(i).is_ok_and(|e| e.is_file() && e.name().is_ok_and(|name| is_ffmpeg(&name))))
        .ok_or_else(broken)?;
    let mut entry = archive.by_index(index).map_err(|_| broken())?;
    let written = std::fs::File::create(target).and_then(|mut out| std::io::copy(&mut entry, &mut out));
    if written.is_err() {
        let _ = std::fs::remove_file(target);
        return Err("ffmpeg 저장 실패. 디스크 공간을 확인하세요.".to_string());
    }
    Ok(())
}

/// 받을 파일. 앱 릴리즈에 자산이 있으면 그것(작고 SHA-256 을 확인한다), 없으면 `FALLBACK_URL`.
pub fn pick_source(latest: Option<Release>) -> Release {
    latest.unwrap_or_else(|| Release {
        version: String::new(),
        notes: String::new(),
        asset_url: FALLBACK_URL.to_string(),
        asset_size: 0,
        sha256: String::new(),
    })
}

pub fn fetch_source() -> Release {
    // 릴리즈 정보를 못 읽어도 받을 곳은 있다
    pick_source(updater::fetch_release(REPO, "0", &|name| name == ASSET).ok().flatten())
}

/// 받아서 `bin_dir\ffmpeg.exe` 로 둔다. 돌려주는 값은 그 폴더.
pub fn install(source: &Release, bin_dir: &Path, group: &KillGroup, progress: &dyn Fn(u64, u64)) -> Result<PathBuf, String> {
    let zip_path = bin_dir.join("ffmpeg.zip");
    updater::download_to(source, &zip_path, progress).map_err(|message| message.replace("업데이트 파일", "ffmpeg"))?;
    let part = bin_dir.join("ffmpeg.exe.part");
    let extracted = extract(&zip_path, &part);
    let _ = std::fs::remove_file(&zip_path);
    extracted?;
    // 받은 파일이 실제로 도는지 본 뒤에야 쓴다
    if !works(&part, group) {
        let _ = std::fs::remove_file(&part);
        return Err("ffmpeg 실행 확인 실패. 다시 시도하세요.".to_string());
    }
    let target = bin_dir.join(EXE);
    let _ = std::fs::remove_file(&target);
    std::fs::rename(&part, &target).map_err(|_| {
        let _ = std::fs::remove_file(&part);
        "ffmpeg 저장 실패. 다시 시도하세요.".to_string()
    })?;
    Ok(bin_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("qm-ffmpeg-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let mut writer = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        for (name, body) in entries {
            writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            writer.write_all(body).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn version_check() {
        for ok in [
            "ffmpeg version 7.1-full_build-www.gyan.dev Copyright (c) 2000-2024",
            "ffmpeg version N-118193-g5f38c82536-20250101 Copyright",
            "ffmpeg version 2024-08-01-git-bcf08c1171-full_build-www.gyan.dev",
            "ffmpeg version n4.4.1 Copyright",
            "ffmpeg version 4.0 Copyright",
            "\r\nffmpeg version 6.1.1\r\nbuilt with gcc",
        ] {
            assert!(version_ok(ok), "{ok}");
        }
        for old in ["ffmpeg version 3.4.2 Copyright", "ffmpeg version n2.8.1", "ffmpeg version 0.11.5", "", "ffmpeg version ", "command not found"] {
            assert!(!version_ok(old), "{old}");
        }
    }

    #[test]
    fn path_dirs_with_ffmpeg_in_order() {
        let root = temp("path");
        let (a, b, c) = (root.join("a"), root.join("b"), root.join("c"));
        for dir in [&a, &b, &c] {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(b.join("ffmpeg.exe"), "x").unwrap();
        std::fs::write(c.join("FFMPEG.EXE"), "x").unwrap();
        std::fs::create_dir_all(a.join("ffmpeg.exe")).unwrap(); // 폴더는 아니다
        let path = std::env::join_paths([a.clone(), root.join("missing"), b.clone(), c.clone()]).unwrap();
        assert_eq!(dirs_on_path(&path), vec![b, c]);
        assert!(dirs_on_path(OsStr::new("")).is_empty());
    }

    #[test]
    fn own_folders_win_and_broken_path_ffmpeg_is_skipped() {
        let root = temp("locate");
        let (install, bin, sys) = (root.join("app"), root.join("bin"), root.join("sys"));
        for dir in [&install, &bin, &sys] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let group = KillGroup::new().unwrap();
        let own = [install.clone(), bin.clone()];
        // PATH 의 것이 실행되지 않으면 없는 셈
        std::fs::write(sys.join("ffmpeg.exe"), "not an exe").unwrap();
        assert_eq!(locate(&own, Some(sys.as_os_str()), &group), None);
        assert_eq!(locate(&own, None, &group), None);
        // 앱이 둔 것은 돌려 보지 않고 쓴다. 설치 폴더가 먼저.
        std::fs::write(bin.join("ffmpeg.exe"), "x").unwrap();
        assert_eq!(locate(&own, Some(sys.as_os_str()), &group), Some(bin.clone()));
        std::fs::write(install.join("ffmpeg.exe"), "x").unwrap();
        assert_eq!(locate(&own, None, &group), Some(install));
    }

    #[test]
    fn extract_finds_ffmpeg_anywhere_in_the_archive() {
        let root = temp("zip");
        let target = root.join("out.exe");
        let flat = root.join("flat.zip");
        make_zip(&flat, &[("ffmpeg.exe", b"flat")]);
        extract(&flat, &target).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"flat");

        let nested = root.join("nested.zip");
        make_zip(
            &nested,
            &[
                ("ffmpeg-master-latest-win64-gpl/bin/ffprobe.exe", b"probe"),
                ("ffmpeg-master-latest-win64-gpl/doc/ffmpeg.exe.txt", b"doc"),
                ("ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe", b"nested"),
            ],
        );
        extract(&nested, &target).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"nested");
    }

    #[test]
    fn extract_reports_bad_archives() {
        let root = temp("badzip");
        let target = root.join("out.exe");
        let empty = root.join("empty.zip");
        make_zip(&empty, &[("readme.txt", b"hi")]);
        assert!(extract(&empty, &target).unwrap_err().contains("압축 해제 실패"));
        let garbage = root.join("garbage.zip");
        std::fs::write(&garbage, "not a zip").unwrap();
        assert!(extract(&garbage, &target).is_err());
        assert!(extract(&root.join("missing.zip"), &target).is_err());
        assert!(!target.exists());
    }

    #[test]
    fn source_prefers_the_release_asset() {
        let own = Release { version: "2.0.0".into(), notes: String::new(), asset_url: "https://x/ffmpeg-win64.zip".into(), asset_size: 9, sha256: "ab".into() };
        assert_eq!(pick_source(Some(own.clone())), own);
        let fallback = pick_source(None);
        assert_eq!(fallback.asset_url, FALLBACK_URL);
        assert_eq!((fallback.asset_size, fallback.sha256.as_str()), (0, ""));
    }
}
