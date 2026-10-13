#![deny(unsafe_code)]

// The phone app's own modules. An Android build with the `watch` feature is
// the Wear OS app, which leaves them out.
#[cfg(any(
    all(target_os = "android", not(feature = "watch")),
    all(feature = "web", target_arch = "wasm32")
))]
mod app;
pub mod data;
pub mod model;
#[cfg(any(
    all(target_os = "android", not(feature = "watch")),
    all(feature = "web", target_arch = "wasm32")
))]
mod motion;
pub mod presentation;
#[cfg(any(
    all(target_os = "android", not(feature = "watch")),
    all(feature = "web", target_arch = "wasm32")
))]
mod screens;
pub mod watch;
#[cfg(any(
    all(target_os = "android", not(feature = "watch")),
    all(feature = "web", target_arch = "wasm32")
))]
mod widgets;

// The phone and tablet app.
#[cfg(not(feature = "watch"))]
cranpose::android_main! {
    launcher: app::create_app(),
    content: app::ShowcaseApp,
}

// The Wear OS app (`android/wear` turns the `watch` feature on).
#[cfg(feature = "watch")]
cranpose::android_main! {
    launcher: watch::create_watch_app(),
    content: watch::WatchShowcase,
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
use wasm_bindgen::prelude::*;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
#[wasm_bindgen(start)]
pub fn web_init() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
#[wasm_bindgen]
pub async fn run_app() -> Result<(), JsValue> {
    app::create_app()
        .run_web("cranpose-showcase-canvas", app::ShowcaseApp)
        .await
}
