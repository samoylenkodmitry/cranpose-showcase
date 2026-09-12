//! Headless check that the floating tab bar keeps absorbing its own presses.
//!
//! The bar hovers over the body list, so a card usually lies right under the
//! pill. Switching tabs rekeys the list, and the rekeyed rows have to stay
//! behind the bar: if they land in front of it the next press reaches the card
//! and opens that body's page instead of switching tabs.
//!
//! Run with:
//! `cargo run --bin robot-tab-switch --features robot-preview`

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

use cranpose::{Robot, SemanticElement};

/// Text only the Explore list shows.
const EXPLORE_MARK: &str = "Search the sky";
/// Text only the empty Saved list shows.
const SAVED_MARK: &str = "No saved worlds yet";
/// Text only a body's page shows.
const DETAIL_MARK: &str = "Feel the gravity";

fn label_bounds(robot: &Robot, name: &str) -> Option<(f32, f32, f32, f32)> {
    fn walk(elements: &[SemanticElement], name: &str) -> Option<(f32, f32, f32, f32)> {
        for element in elements {
            if element.text.as_deref() == Some(name) {
                let bounds = element.bounds;
                return Some((bounds.x, bounds.y, bounds.width, bounds.height));
            }
            if let Some(found) = walk(&element.children, name) {
                return Some(found);
            }
        }
        None
    }
    walk(&robot.get_semantics().expect("semantics"), name)
}

fn shows(robot: &Robot, name: &str) -> bool {
    label_bounds(robot, name).is_some()
}

/// Whether a card title sits within the press's row, which is what makes this
/// test able to fail at all.
fn card_under(robot: &Robot, y: f32) -> bool {
    label_bounds(robot, "Jupiter").is_some_and(|(_, title_y, _, _)| (title_y - y).abs() < 30.0)
}

fn press_tab(robot: &Robot, tab: &str) -> (f32, bool) {
    let (x, y, width, height) = robot
        .find_button_bounds_exact(tab)
        .expect("query the tab bar")
        .unwrap_or_else(|| panic!("the tab bar should offer {tab}"));
    let center = (x + width * 0.5, y + height * 0.5);
    let covered = card_under(robot, center.1);
    robot.click(center.0, center.1).expect("press the tab");
    std::thread::sleep(Duration::from_millis(600));
    robot.pump_frames(12).expect("settle the press");
    (center.1, covered)
}

fn main() {
    let _ = env_logger::try_init();
    app::create_app()
        .with_headless(std::env::var("SHOWCASE_ROBOT_HEADLESS").as_deref() != Ok("0"))
        .with_test_driver(move |robot: Robot| {
            std::thread::sleep(Duration::from_millis(500));
            robot.pump_frames(8).expect("settle the first frame");

            let mut failures: Vec<String> = Vec::new();
            let mut covered_once = false;
            for (step, tab, mark) in [
                (1, "Saved", SAVED_MARK),
                (2, "Explore", EXPLORE_MARK),
                (3, "Saved", SAVED_MARK),
            ] {
                let (_, covered) = press_tab(&robot, tab);
                covered_once |= covered;
                if shows(&robot, DETAIL_MARK) {
                    failures.push(format!(
                        "step {step}: pressing {tab} opened a body's page, so a card took the press"
                    ));
                    break;
                }
                if !shows(&robot, mark) {
                    failures.push(format!("step {step}: pressing {tab} did not show its list"));
                    break;
                }
            }

            robot.exit().ok();
            assert!(
                covered_once,
                "no card ever sat under the tab bar, so this run could not have caught the bug"
            );
            assert!(failures.is_empty(), "{}", failures.join("; "));
        })
        .run(app::ShowcaseApp);
}
