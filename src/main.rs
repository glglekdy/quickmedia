//! QuickMedia 진입점.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = quickmedia::ui::run() {
        eprintln!("창을 띄우지 못했다: {e}");
        std::process::exit(1);
    }
}
