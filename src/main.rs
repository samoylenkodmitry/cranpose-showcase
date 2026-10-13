#![deny(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
// `--features watch` previews the watch app, which leaves the phone app's
// modules unused.
#![cfg_attr(feature = "watch", allow(dead_code, unused_imports))]

mod app;
mod data;
mod model;
mod motion;
mod presentation;
mod screens;
mod widgets;

fn main() {
    #[cfg(feature = "logging")]
    let _ = env_logger::try_init();
    #[cfg(not(feature = "watch"))]
    let result = app::create_app().try_run(app::ShowcaseApp);
    #[cfg(feature = "watch")]
    let result = cranpose_showcase::watch::create_watch_app()
        .try_run(cranpose_showcase::watch::WatchShowcase);
    if let Err(error) = result {
        eprintln!("Failed to launch Showcase Cranpose: {error}");
        std::process::exit(1);
    }
}
