//! Headless check that trimming rows out of a lazy list leaves no ghosts.
//!
//! Stars the first three bodies in Explore, switches to Saved, then un-stars
//! them one at a time. Every removal shifts the rows below it up, and the
//! frame afterwards must match what the same list looks like when it is
//! composed from scratch — a row that left must stop being painted, not stay
//! stacked under the row that took its place.
//!
//! Run with:
//! `cargo run --bin robot-saved-removal --features robot-preview`
//!
//! `SHOWCASE_ROBOT_OUT_DIR` picks where the frames are written (default `/tmp`).

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

use cranpose::{Robot, RobotScreenshot, SemanticElement};

const SAVED: [&str; 3] = ["Sun", "Mercury", "Venus"];
/// The surviving card's text column once the list is one row long. The
/// planet thumbnail and the starfield behind the glass both keep moving, so
/// the comparison stays on the part of the card that should be still.
const CARD_TEXT_RECT: (u32, u32, u32, u32) = (104, 205, 334, 292);
/// Mean per-channel difference the drifting starfield alone can account for.
/// A ghost row under the survivor is worth an order of magnitude more.
const GHOST_TOLERANCE: f32 = 6.0;

fn save(robot: &Robot, out_dir: &str, name: &str) -> RobotScreenshot {
    let shot = robot.screenshot().expect("screenshot");
    let path = std::path::Path::new(out_dir).join(name);
    let image = image::RgbaImage::from_raw(shot.width, shot.height, shot.pixels.clone())
        .expect("valid screenshot buffer");
    image.save(&path).expect("save screenshot");
    println!("wrote {}", path.display());
    shot
}

fn mean_rgb_delta(
    before: &RobotScreenshot,
    after: &RobotScreenshot,
    rect: (u32, u32, u32, u32),
) -> f32 {
    assert_eq!((before.width, before.height), (after.width, after.height));
    let scale_x = before.width as f32 / before.logical_width;
    let scale_y = before.height as f32 / before.logical_height;
    let (left, top, right, bottom) = rect;
    let left = ((left as f32 * scale_x) as u32).min(before.width);
    let right = ((right as f32 * scale_x) as u32).min(before.width);
    let top = ((top as f32 * scale_y) as u32).min(before.height);
    let bottom = ((bottom as f32 * scale_y) as u32).min(before.height);
    let mut delta = 0u64;
    let mut channels = 0u64;
    for y in top..bottom {
        for x in left..right {
            let offset = ((y * before.width + x) * 4) as usize;
            for channel in 0..3 {
                delta +=
                    before.pixels[offset + channel].abs_diff(after.pixels[offset + channel]) as u64;
                channels += 1;
            }
        }
    }
    delta as f32 / channels.max(1) as f32
}

/// The bounds of the one element whose whole label is `name`.
///
/// `Robot::find_text_bounds` matches substrings, and every tagline in this
/// list mentions another body — "Closest to the Sun" would answer for a Sun
/// row that is no longer there.
fn row_title_bounds(robot: &Robot, name: &str) -> Option<(f32, f32, f32, f32)> {
    fn walk(elements: &[SemanticElement], name: &str) -> Option<(f32, f32, f32, f32)> {
        for element in elements {
            if element.text.as_deref() == Some(name) {
                return Some((
                    element.bounds.x,
                    element.bounds.y,
                    element.bounds.width,
                    element.bounds.height,
                ));
            }
            if let Some(found) = walk(&element.children, name) {
                return Some(found);
            }
        }
        None
    }
    walk(&robot.get_semantics().expect("semantics"), name)
}

fn star_center_for_row(robot: &Robot, name: &str) -> (f32, f32) {
    let (_, y, _, height) =
        row_title_bounds(robot, name).unwrap_or_else(|| panic!("row {name} is not present"));
    let elements = robot.get_semantics().expect("semantics");
    let mut best: Option<(f32, f32, f32)> = None;
    fn walk(elements: &[SemanticElement], row_center: f32, best: &mut Option<(f32, f32, f32)>) {
        for element in elements {
            let center_x = element.bounds.x + element.bounds.width * 0.5;
            let center_y = element.bounds.y + element.bounds.height * 0.5;
            let distance = (center_y - row_center).abs();
            if element.clickable
                && element.bounds.width <= 56.0
                && element.bounds.height <= 56.0
                && center_x > 300.0
                && distance <= 45.0
                && best.is_none_or(|(_, _, current)| distance < current)
            {
                *best = Some((center_x, center_y, distance));
            }
            walk(&element.children, row_center, best);
        }
    }
    walk(&elements, y + height * 0.5, &mut best);
    let (x, y, _) = best.unwrap_or_else(|| panic!("no star button found beside row {name}"));
    (x, y)
}

fn toggle_star(robot: &Robot, name: &str) {
    let (x, y) = star_center_for_row(robot, name);
    robot.click(x, y).expect("toggle the row's star");
    std::thread::sleep(Duration::from_millis(400));
    robot.pump_frames(12).expect("settle the star toggle");
}

fn assert_rows(robot: &Robot, expected: &[&str], stage: &str) {
    for name in SAVED {
        let present = row_title_bounds(robot, name).is_some();
        assert_eq!(
            present,
            expected.contains(&name),
            "{stage}: {name} is {} in the Saved list",
            if present { "present" } else { "missing" }
        );
    }
}

fn open_tab(robot: &Robot, name: &str) {
    let (x, y, width, height) = robot
        .find_button_bounds_exact(name)
        .expect("query tab")
        .unwrap_or_else(|| panic!("{name} tab is present"));
    robot
        .click(x + width * 0.5, y + height * 0.5)
        .expect("open tab");
    std::thread::sleep(Duration::from_millis(500));
    robot.pump_frames(12).expect("settle tab");
}

fn click_chip(robot: &Robot, label: &str) {
    let (x, y, width, height) =
        row_title_bounds(robot, label).unwrap_or_else(|| panic!("chip {label} is present"));
    robot
        .click(x + width * 0.5, y + height * 0.5)
        .expect("pick a category");
    std::thread::sleep(Duration::from_millis(400));
    robot.pump_frames(12).expect("settle the category");
}

/// Leaves Explore for Saved with only one row left in Explore.
///
/// The tab bar floats over the list and a card under it takes the press
/// instead, so the list has to be short enough that nothing is beneath the
/// bar when it is clicked.
fn open_saved(robot: &Robot) {
    click_chip(robot, "Star");
    open_tab(robot, "Saved");
}

fn main() {
    let _ = env_logger::try_init();
    let out_dir = std::env::var("SHOWCASE_ROBOT_OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let headless = std::env::var("SHOWCASE_ROBOT_HEADLESS").as_deref() != Ok("0");

    app::create_app()
        .with_headless(headless)
        .with_test_driver(move |robot: Robot| {
            std::thread::sleep(Duration::from_millis(500));
            robot.pump_frames(8).expect("settle initial screen");
            for name in SAVED {
                toggle_star(&robot, name);
            }

            open_saved(&robot);
            assert_rows(&robot, &SAVED, "after saving three bodies");
            save(&robot, &out_dir, "saved-three.png");

            toggle_star(&robot, "Sun");
            assert_rows(
                &robot,
                &["Mercury", "Venus"],
                "after dropping the first row",
            );
            toggle_star(&robot, "Mercury");
            assert_rows(&robot, &["Venus"], "after dropping the second row");
            std::thread::sleep(Duration::from_millis(400));
            robot.pump_frames(12).expect("settle the trimmed list");
            let trimmed = save(&robot, &out_dir, "saved-one.png");

            // The same one-row list, composed from scratch: switching tabs
            // rebuilds the screen, so nothing a removal left behind survives.
            open_tab(&robot, "Explore");
            open_saved(&robot);
            assert_rows(&robot, &["Venus"], "after recomposing the Saved list");
            std::thread::sleep(Duration::from_millis(400));
            robot.pump_frames(12).expect("settle the recomposed list");
            let rebuilt = save(&robot, &out_dir, "saved-one-rebuilt.png");

            let delta = mean_rgb_delta(&trimmed, &rebuilt, CARD_TEXT_RECT);
            println!("card mean RGB delta after trimming: {delta:.3}");
            robot.exit().ok();
            assert!(
                delta <= GHOST_TOLERANCE,
                "removed rows are still painted under the survivor: the card differs from the \
                 freshly composed list by a mean of {delta:.3} per channel"
            );
        })
        .run(app::ShowcaseApp);
}
