//! Headless visual QA for the split list/detail layout.
//!
//! Runs the real `ShowcaseApp` in a desktop-sized window and checks that the
//! list keeps its own column while a selected body fills the rest, then dumps
//! the frames.
//!
//! Run with:
//! `cargo run --bin robot-wide-layout --features robot-preview`
//!
//! `SHOWCASE_ROBOT_OUT_DIR` picks the output directory (default `/tmp`).

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

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 860;
/// The widest a card may be drawn: the list column's width less its margins.
const MAX_ROW_WIDTH: f32 = 400.0;

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
    let out_dir = std::env::var("SHOWCASE_ROBOT_OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let headless = std::env::var("SHOWCASE_ROBOT_HEADLESS").as_deref() != Ok("0");

    app::create_app()
        .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_headless(headless)
        .with_test_driver(move |robot: Robot| {
            std::thread::sleep(Duration::from_millis(500));
            robot.pump_frames(8).expect("settle split layout");
            save(&robot, &out_dir, "wide-no-selection.png");

            let (_, _, width, _) = robot
                .find_text_bounds("Search the sky")
                .expect("query search field")
                .expect("search field is present");
            assert!(
                width <= MAX_ROW_WIDTH,
                "the list column stretched across the window: search field is {width:.0} wide"
            );

            let (x, y, width, height) = robot
                .find_text_bounds("Earth")
                .expect("query Earth row")
                .expect("Earth row is present");
            assert!(
                x + width < WINDOW_WIDTH as f32 * 0.5,
                "the list column reaches past the middle of a wide window"
            );
            robot
                .click(x + width * 0.5, y + height * 0.5)
                .expect("open Earth detail");
            std::thread::sleep(Duration::from_millis(650));
            robot.pump_frames(10).expect("settle Earth detail");
            save(&robot, &out_dir, "wide-earth-detail.png");

            assert!(
                robot
                    .find_text_bounds("Search the sky")
                    .expect("query search field beside the detail")
                    .is_some(),
                "the split layout dropped the list when a body was opened"
            );
            let (x, _, _, _) = robot
                .find_text_bounds("Feel the gravity")
                .expect("query the detail body")
                .expect("the detail pane is present");
            assert!(
                x > MAX_ROW_WIDTH,
                "the detail pane opened over the list instead of beside it"
            );

            robot.exit().expect("exit wide layout robot");
        })
        .run(app::ShowcaseApp);
}
