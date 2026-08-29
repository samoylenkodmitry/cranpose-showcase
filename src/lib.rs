#![deny(unsafe_code)]

#[cfg(any(target_os = "android", all(feature = "web", target_arch = "wasm32")))]
mod app;
#[cfg(any(target_os = "android", all(feature = "web", target_arch = "wasm32")))]
mod model;
#[cfg(any(target_os = "android", all(feature = "web", target_arch = "wasm32")))]
mod motion;
#[cfg(any(target_os = "android", all(feature = "web", target_arch = "wasm32")))]
mod screens;
#[cfg(any(target_os = "android", all(feature = "web", target_arch = "wasm32")))]
mod widgets;

cranpose::android_main! {
    launcher: app::create_app(),
    content: app::OrbitApp,
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
        .run_web("cranpose-orbit-canvas", app::OrbitApp)
        .await
}
