#![deny(unsafe_code)]

#[cfg(target_os = "ios")]
mod app;
#[cfg(target_os = "ios")]
mod model;
#[cfg(target_os = "ios")]
mod motion;
#[cfg(target_os = "ios")]
mod screens;
#[cfg(target_os = "ios")]
mod widgets;

/// winit starts `UIApplicationMain`, so this is the whole entry point: the
/// window is sized by UIKit, so no explicit size is requested.
#[cfg(target_os = "ios")]
fn main() {
    use cranpose::AppLauncher;

    if let Err(error) = AppLauncher::new().try_run(app::OrbitApp) {
        eprintln!("Failed to launch Cranpose Orbit: {error}");
        std::process::exit(1);
    }
}

// The `cranpose-orbit-ios` binary only runs on iOS. The `ios` feature can
// still be enabled on other targets (for example `cargo ... --all-features`
// checks), where this binary has no entry point to call.
#[cfg(not(target_os = "ios"))]
fn main() {}
