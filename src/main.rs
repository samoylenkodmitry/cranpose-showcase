#![deny(unsafe_code)]

mod app;
mod model;
mod motion;
mod screens;
mod widgets;

fn main() {
    #[cfg(feature = "logging")]
    let _ = env_logger::try_init();
    if let Err(error) = app::create_app().try_run(app::ShowcaseApp) {
        eprintln!("Failed to launch Showcase Cranpose: {error}");
        std::process::exit(1);
    }
}
