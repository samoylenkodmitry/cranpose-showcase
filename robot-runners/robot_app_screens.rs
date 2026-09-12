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

fn save(robot: &Robot, out_dir: &str, name: &str) -> cranpose::RobotScreenshot {
    let shot = robot.screenshot().expect("screenshot");
    let path = std::path::Path::new(out_dir).join(name);
    let image = image::RgbaImage::from_raw(shot.width, shot.height, shot.pixels.clone())
        .expect("valid screenshot buffer");
    image.save(&path).expect("save screenshot");
    println!("wrote {}", path.display());
    shot
}

fn mean_rgb_delta_in_circle(
    before: &cranpose::RobotScreenshot,
    after: &cranpose::RobotScreenshot,
    center_x: f32,
    center_y: f32,
    radius: f32,
) -> f32 {
    assert_eq!((before.width, before.height), (after.width, after.height));
    let radius_squared = radius * radius;
    let mut delta = 0u64;
    let mut channel_count = 0u64;
    for y in 0..before.height {
        for x in 0..before.width {
            let dx = x as f32 + 0.5 - center_x;
            let dy = y as f32 + 0.5 - center_y;
            if dx * dx + dy * dy > radius_squared {
                continue;
            }
            let offset = ((y * before.width + x) * 4) as usize;
            for channel in 0..3 {
                delta +=
                    before.pixels[offset + channel].abs_diff(after.pixels[offset + channel]) as u64;
                channel_count += 1;
            }
        }
    }
    delta as f32 / channel_count.max(1) as f32
}

fn assert_detail_planet_rotates(robot: &Robot, before: &cranpose::RobotScreenshot) {
    std::thread::sleep(Duration::from_millis(900));
    robot.pump_frames(6).expect("advance detail animation");
    let after = robot.screenshot().expect("later detail screenshot");
    let mean_delta =
        mean_rgb_delta_in_circle(before, &after, before.width as f32 * 0.5, 212.0, 58.0);
    assert!(
        mean_delta >= 2.0,
        "detail planet surface stayed static: mean RGB delta was {mean_delta:.3}"
    );
}

fn assert_list_leaves_composition_on_detail(robot: &Robot) {
    assert!(
        robot
            .find_text_bounds("Search the sky")
            .expect("query detail semantics")
            .is_none(),
        "the list screen stayed composed behind the settled detail route"
    );
}

const SETTLE_SLOT: Duration = Duration::from_millis(250);
const SETTLE_DEADLINE: Duration = Duration::from_millis(5_000);

/// Blocks until the composition stops recomposing, so the ambient-motion
/// measurement that follows cannot charge a route change, a tab glide or a
/// scroll settle to the ambient animation. A window that rendered no frames
/// proves nothing — an armed animation is not sampled until something draws —
/// so only a window that both rendered and did not recompose counts as quiet,
/// and two in a row are required before measuring.
fn wait_for_composition_to_settle(robot: &Robot, screen: &str) {
    let slots = SETTLE_DEADLINE.as_millis() / SETTLE_SLOT.as_millis();
    let mut series = Vec::new();
    let mut quiet = 0;
    for _ in 0..slots {
        robot.reset_fps_stats().expect("reset frame statistics");
        std::thread::sleep(SETTLE_SLOT);
        robot.pump_frames(4).expect("advance settling animations");
        let stats = robot.fps_stats().expect("read frame statistics");
        series.push((stats.recompositions, stats.frame_count));
        if stats.recompositions == 0 && stats.frame_count > 0 {
            quiet += 1;
            if quiet == 2 {
                return;
            }
        } else {
            quiet = 0;
        }
    }
    panic!(
        "{screen} never settled within {}ms: (recompositions, frames) per {}ms window were {series:?}",
        SETTLE_DEADLINE.as_millis(),
        SETTLE_SLOT.as_millis()
    );
}

fn assert_ambient_motion_does_not_recompose(robot: &Robot, screen: &str) {
    wait_for_composition_to_settle(robot, screen);
    robot.reset_fps_stats().expect("reset frame statistics");
    std::thread::sleep(Duration::from_millis(1_100));
    robot.pump_frames(8).expect("advance ambient animation");
    let stats = robot.fps_stats().expect("read frame statistics");
    assert!(
        stats.frame_count > 0,
        "{screen} did not render ambient frames"
    );
    assert_eq!(
        stats.recompositions, 0,
        "{screen} ambient animation triggered recomposition"
    );
}

fn accent_pixel_fraction(shot: &cranpose::RobotScreenshot, bounds: (f32, f32, f32, f32)) -> f32 {
    let (x, y, width, height) = bounds;
    let left = (x - 44.0).floor().max(0.0) as u32;
    let top = (y - 34.0).floor().max(0.0) as u32;
    let right = (x + width + 44.0).ceil().min(shot.width as f32) as u32;
    let bottom = (y + height + 34.0).ceil().min(shot.height as f32) as u32;
    let mut accent = 0usize;
    let mut total = 0usize;
    for row in top..bottom {
        for column in left..right {
            let index = ((row * shot.width + column) * 4) as usize;
            let red = shot.pixels[index];
            let blue = shot.pixels[index + 2];
            if blue.saturating_sub(red) > 20 && blue > 80 {
                accent += 1;
            }
            total += 1;
        }
    }
    accent as f32 / total.max(1) as f32
}

fn assert_tab_press_does_not_create_opaque_accent_glass(robot: &Robot) {
    let bounds = robot
        .find_button_bounds_exact("Explore")
        .expect("find Explore tab")
        .expect("Explore tab is present");
    let resting = robot.screenshot().expect("resting navbar screenshot");
    let resting_fraction = accent_pixel_fraction(&resting, bounds);
    let (x, y, width, height) = bounds;
    let center_x = x + width * 0.5;
    let center_y = y + height * 0.5;
    robot
        .touch_down(center_x, center_y)
        .expect("press Explore tab");
    robot.pump_frames(12).expect("settle pressed navbar");
    let pressed = robot.screenshot().expect("pressed navbar screenshot");
    let pressed_fraction = accent_pixel_fraction(&pressed, bounds);
    robot
        .touch_up(center_x, center_y)
        .expect("release Explore tab");
    assert!(
        pressed_fraction <= resting_fraction + 0.15,
        "pressed tab became opaque accent glass: resting={resting_fraction:.3}, pressed={pressed_fraction:.3}"
    );
}

fn main() {
    let _ = env_logger::try_init();
    let out_dir = std::env::var("SHOWCASE_ROBOT_OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let headless = std::env::var("SHOWCASE_ROBOT_HEADLESS").as_deref() != Ok("0");

    app::create_app()
        .with_headless(headless)
        .with_test_driver(move |robot: Robot| {
            std::thread::sleep(Duration::from_millis(500));
            robot.pump_frames(6).expect("settle initial screen");
            save(&robot, &out_dir, "app-explore-list.png");
            assert_ambient_motion_does_not_recompose(&robot, "list");

            let (x, y, width, height) = robot
                .find_text_bounds("Earth")
                .expect("query Earth row")
                .expect("Earth row is present");
            robot
                .click(x + width * 0.5, y + height * 0.5)
                .expect("open Earth detail");
            std::thread::sleep(Duration::from_millis(650));
            robot.pump_frames(6).expect("settle Earth detail");
            let detail = save(&robot, &out_dir, "app-earth-detail.png");
            assert_list_leaves_composition_on_detail(&robot);
            assert_detail_planet_rotates(&robot, &detail);
            assert_ambient_motion_does_not_recompose(&robot, "detail");
            robot.click(28.0, 25.0).expect("return to body list");
            std::thread::sleep(Duration::from_millis(650));
            robot.pump_frames(6).expect("settle body list");

            robot
                .drag(200.0, 700.0, 200.0, 500.0)
                .expect("scroll body list");
            robot.pump_frames(4).expect("advance list scroll");
            robot
                .drag(200.0, 700.0, 200.0, 500.0)
                .expect("scroll body list again");
            robot.pump_frames(8).expect("settle list scroll");
            save(&robot, &out_dir, "app-explore-scrolled.png");
            robot
                .drag(200.0, 300.0, 200.0, 900.0)
                .expect("return toward list start");
            robot.pump_frames(4).expect("advance reverse scroll");
            robot
                .drag(200.0, 300.0, 200.0, 900.0)
                .expect("return to list start");
            robot.pump_frames(8).expect("settle list start");

            save(&robot, &out_dir, "app-tabbar-unpressed.png");
            assert_tab_press_does_not_create_opaque_accent_glass(&robot);
            let (x, y, width, height) = robot
                .find_button_bounds_exact("Saved")
                .expect("query Saved tab")
                .expect("Saved tab is present");
            let center_x = x + width * 0.5;
            let center_y = y + height * 0.5;
            robot
                .touch_down(center_x, center_y)
                .expect("press Saved tab");
            robot.pump_frames(10).expect("advance tab press");
            save(&robot, &out_dir, "app-tabbar-pressed.png");
            robot
                .touch_up(center_x, center_y)
                .expect("release Saved tab");
            robot.pump_frames(4).expect("settle Saved tab");
            assert_ambient_motion_does_not_recompose(&robot, "empty saved list");

            robot.exit().expect("exit app screen robot");
        })
        .run(app::ShowcaseApp);
}
