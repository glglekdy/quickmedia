//! 화면. Slint 는 Rust 가 정한 상태를 그리기만 한다.

pub mod cards;
pub mod download;
pub mod nav;

use slint::ComponentHandle;

use crate::app_info::VERSION;

slint::include_modules!();

/// 창을 띄우고 닫힐 때까지 돈다.
pub fn run() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let nav_global = window.global::<Nav>();
    nav_global.set_version(format!("버전 {VERSION}").into());
    let weak = window.as_weak();
    nav_global.on_go(move |key| {
        let Some(window) = weak.upgrade() else { return };
        let nav_global = window.global::<Nav>();
        let (page, open) = nav::nav_after_click(&nav_global.get_page(), nav_global.get_settings_open(), &key);
        nav_global.set_page(page.into());
        nav_global.set_settings_open(open);
    });

    #[cfg(debug_assertions)]
    debug::hooks(&window);

    window.run()
}

/// 개발용: 화면을 눈으로 확인하려고 그림 파일로 남긴다.
///
/// `QUICKMEDIA_PAGE=network` 로 처음 화면을 정하고, `QUICKMEDIA_SNAPSHOT=<png 경로>` 를 주면
/// 잠깐 뒤 창을 그 파일로 저장하고 끝낸다.
#[cfg(debug_assertions)]
mod debug {
    use super::*;

    pub fn hooks(window: &AppWindow) {
        if let Ok(page) = std::env::var("QUICKMEDIA_PAGE") {
            window.global::<Nav>().invoke_go(page.into());
        }
        let Ok(path) = std::env::var("QUICKMEDIA_SNAPSHOT") else { return };
        let delay = std::env::var("QUICKMEDIA_SNAPSHOT_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(1200);
        let weak = window.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_millis(delay), move || {
            if let Some(window) = weak.upgrade() {
                match window.window().take_snapshot() {
                    Ok(pixels) => {
                        let saved = image::save_buffer(
                            &path,
                            pixels.as_bytes(),
                            pixels.width(),
                            pixels.height(),
                            image::ExtendedColorType::Rgba8,
                        );
                        if let Err(e) = saved {
                            eprintln!("그림을 저장하지 못했다: {e}");
                        }
                    }
                    Err(e) => eprintln!("창을 찍지 못했다: {e}"),
                }
            }
            let _ = slint::quit_event_loop();
        });
    }
}
