//! 화면. Slint 는 Rust 가 정한 상태를 그리기만 한다.

pub mod app;
pub mod cards;
pub mod download;
pub mod nav;
mod win;

use slint::ComponentHandle;

slint::include_modules!();

/// 창을 띄우고 닫힐 때까지 돈다.
pub fn run() -> Result<(), slint::PlatformError> {
    let window = app::build()?;

    #[cfg(debug_assertions)]
    debug::hooks(&window);

    let result = window.run();
    app::teardown();
    result
}

/// 개발용: 화면을 눈으로 확인하려고 그림 파일로 남긴다.
///
/// - `QUICKMEDIA_PAGE=network`: 처음 화면
/// - `QUICKMEDIA_URL=<링크>`: 그 링크를 넣고 다운로드를 누른다
/// - `QUICKMEDIA_SNAPSHOT=<png 경로>`: `QUICKMEDIA_SNAPSHOT_MS`(기본 1200) 뒤 창을 저장하고 끝낸다
#[cfg(debug_assertions)]
mod debug {
    use super::*;

    fn act(window: &AppWindow, step: &str) {
        use slint::platform::{PointerEventButton, WindowEvent};
        let target = window.window();
        if let Some(text) = step.strip_prefix("type:") {
            for ch in text.chars() {
                let text: slint::SharedString = ch.to_string().into();
                target.dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
                target.dispatch_event(WindowEvent::KeyReleased { text });
            }
            return;
        }
        let Some((x, y)) = step.split_once(',') else { return };
        let (Ok(x), Ok(y)) = (x.trim().parse::<f32>(), y.trim().parse::<f32>()) else { return };
        let position = slint::LogicalPosition::new(x, y);
        target.dispatch_event(WindowEvent::PointerMoved { position });
        target.dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
        target.dispatch_event(WindowEvent::PointerReleased { position, button: PointerEventButton::Left });
    }

    pub fn hooks(window: &AppWindow) {
        if let Ok(page) = std::env::var("QUICKMEDIA_PAGE") {
            window.global::<Nav>().invoke_go(page.into());
        }
        if let Ok(url) = std::env::var("QUICKMEDIA_URL") {
            let dl = window.global::<Download>();
            for link in url.split_whitespace() {
                dl.set_url(link.into());
                dl.invoke_add();
            }
        }
        // QUICKMEDIA_CLICK="358,269;300,340": 그 자리들을 차례로 누른다(0.4초 간격).
        // 항목이 `type:글자` 면 누르는 대신 그 글자를 친다.
        if let Ok(steps) = std::env::var("QUICKMEDIA_CLICK") {
            for (i, step) in steps.split(';').enumerate() {
                let (weak, step) = (window.as_weak(), step.to_string());
                slint::Timer::single_shot(std::time::Duration::from_millis(500 + 400 * i as u64), move || {
                    let Some(window) = weak.upgrade() else { return };
                    act(&window, &step);
                });
            }
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
