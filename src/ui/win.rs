//! Windows 에 맡기는 일: 클립보드, 탐색기, 물어보는 창, 폴더 고르기.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};

const CF_UNICODETEXT: u32 = 13;

/// 클립보드의 글. 글이 아니거나 열지 못하면 `None`.
pub fn clipboard_text() -> Option<String> {
    // SAFETY: 클립보드를 연 동안에만 핸들을 쓰고, 0 으로 끝나는 UTF-16 글을 끝까지만 읽는다.
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }
        let handle = GetClipboardData(CF_UNICODETEXT);
        let mut text = None;
        if !handle.is_null() {
            let ptr = GlobalLock(handle) as *const u16;
            if !ptr.is_null() {
                let mut len = 0;
                while *ptr.add(len) != 0 {
                    len += 1;
                }
                text = Some(String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)));
                GlobalUnlock(handle);
            }
        }
        CloseClipboard();
        text
    }
}

/// 탐색기에서 파일을 선택한 채로 연다. 파일이 없으면 폴더만.
pub fn reveal(path: &str) {
    let file = Path::new(path);
    if file.exists() {
        // explorer 는 `/select,"경로"` 를 한 덩어리로 받아야 한다
        let _ = Command::new("explorer").raw_arg(format!("/select,\"{path}\"")).spawn();
    } else if let Some(dir) = file.parent() {
        open_folder(&dir.to_string_lossy());
    }
}

pub fn open_folder(folder: &str) {
    let _ = Command::new("explorer").arg(folder).spawn();
}

/// 예/아니요로 묻는다.
pub fn confirm(title: &str, question: &str) -> bool {
    let answer = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(question)
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();
    answer == rfd::MessageDialogResult::Yes
}

pub fn pick_folder(title: &str, start: &str) -> Option<PathBuf> {
    rfd::FileDialog::new().set_title(title).set_directory(start).pick_folder()
}

pub fn pick_file(title: &str, filter_name: &str, extensions: &[&str]) -> Option<PathBuf> {
    rfd::FileDialog::new().set_title(title).add_filter(filter_name, extensions).add_filter("모든 파일", &["*"]).pick_file()
}
