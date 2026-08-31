//! Headless visual QA for the real app screens.
//!
//! Renders the actual production `ShowcaseApp` composable (not an isolated
//! test bed) through Cranpose's `Robot` driver, so shader integration is
//! checked against the real Liquid Glass cards, fixed gradient crown, and scroll
//! behavior it ships inside. Needs no visible window and no OS
//! screen-recording permission.
//!
//! Run with:
//! `cargo run --bin robot-app-screens --features robot-preview`
//!
//! `ORBIT_ROBOT_OUT_DIR` picks the output directory (default `/tmp`).

#[path = "../src/app.rs"]
mod app;
#[path = "../src/model.rs"]
mod model;
#[path = "../src/motion.rs"]
mod motion;
#[path = "../src/screens/mod.rs"]
mod screens;
#[path = "../src/widgets/mod.rs"]
mod widgets;

use std::time::Duration;

use cranpose::Robot;

fn save(robot: &Robot, out_dir: &str, name: &str) {
    let shot = robot.screenshot().expect("screenshot");
    let path = std::path::Path::new(out_dir).join(name);
    let image = image::RgbaImage::from_raw(shot.width, shot.height, shot.pixels.clone())
        .expect("valid screenshot buffer");
    image.save(&path).expect("save screenshot");
    println!("wrote {}", path.display());
}

fn main() {
    let _ = env_logger::try_init();
    let out_dir = std::env::var("ORBIT_ROBOT_OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let headless = std::env::var("ORBIT_ROBOT_HEADLESS").as_deref() != Ok("0");

    app::create_app()
        .with_headless(headless)
        .with_test_driver(move |robot: Robot| {
            std::thread::sleep(Duration::from_millis(500));
            let _ = robot.pump_frames(6);
            save(&robot, &out_dir, "app-explore-list.png");

            if let Ok(Some((x, y, w, h))) = robot.find_text_bounds("Earth") {
                let _ = robot.click(x + w * 0.5, y + h * 0.5);
                std::thread::sleep(Duration::from_millis(650));
                let _ = robot.pump_frames(6);
                save(&robot, &out_dir, "app-earth-detail.png");
                let _ = robot.click(28.0, 25.0);
                std::thread::sleep(Duration::from_millis(650));
                let _ = robot.pump_frames(6);
            } else {
                println!("could not find an Earth row to open the detail screen");
            }

            let _ = robot.drag(200.0, 700.0, 200.0, 500.0);
            let _ = robot.pump_frames(4);
            let _ = robot.drag(200.0, 700.0, 200.0, 500.0);
            let _ = robot.pump_frames(8);
            save(&robot, &out_dir, "app-explore-scrolled.png");
            let _ = robot.drag(200.0, 300.0, 200.0, 900.0);
            let _ = robot.pump_frames(4);
            let _ = robot.drag(200.0, 300.0, 200.0, 900.0);
            let _ = robot.pump_frames(8);

            save(&robot, &out_dir, "app-tabbar-unpressed.png");
            if let Ok(Some((x, y, w, h))) = robot.find_text_bounds("Saved") {
                let cx = x + w * 0.5;
                let cy = y + h * 0.5;
                let _ = robot.touch_down(cx, cy);
                let _ = robot.pump_frames(10);
                save(&robot, &out_dir, "app-tabbar-pressed.png");
                let _ = robot.touch_up(cx, cy);
                let _ = robot.pump_frames(4);
            } else {
                println!("could not find the Saved tab to test the press state");
            }

            let _ = robot.exit();
        })
        .run(app::ShowcaseApp);
}
