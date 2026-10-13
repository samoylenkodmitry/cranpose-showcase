//! Drives the watch app headless, the way a person uses it on a watch: the
//! list, the crown, a body's page and the swipe back.

use cranpose_showcase::watch::WatchShowcase;
use cranpose_testing::{create_headless_robot_test, RobotTestRule, TestRenderer};

/// A 454 pixel round Wear OS screen at density 2, in points.
const SIDE: u32 = 227;
const MIDDLE: f32 = SIDE as f32 / 2.0;

/// One second of frames, long enough for any screen change to finish.
const SECOND_OF_FRAMES: usize = 60;
const FRAME_NANOS: u64 = 16_666_667;

fn shows(texts: &[String], text: &str) -> bool {
    texts.iter().any(|shown| shown == text)
}

/// Lets a screen change run to its end. A crossfade keeps the screen it
/// leaves on screen until it ends.
fn settle(robot: &mut RobotTestRule<TestRenderer>) {
    for _ in 0..SECOND_OF_FRAMES {
        robot.advance_time(FRAME_NANOS);
    }
    robot.wait_for_idle();
}

#[test]
fn the_crown_scrolls_the_list() {
    let mut robot = create_headless_robot_test(SIDE, SIDE, WatchShowcase);
    let texts = robot.get_all_text();
    assert!(shows(&texts, "Star chart"), "no header in {texts:?}");
    assert!(
        shows(&texts, "Sun"),
        "the first body is missing from {texts:?}"
    );
    assert!(
        !shows(&texts, "Proxima Centauri b"),
        "the last body is composed before the list scrolls"
    );

    robot.move_to(MIDDLE, MIDDLE);
    for detent in 0..12 {
        robot
            .shell_mut()
            .rotary_scrolled_by_detents(-1.0, 100 * detent);
        robot.wait_for_idle();
    }

    let texts = robot.get_all_text();
    assert!(
        shows(&texts, "Proxima Centauri b"),
        "the crown did not bring the last body into view: {texts:?}"
    );
}

#[test]
fn a_body_opens_and_the_swipe_goes_back() {
    let mut robot = create_headless_robot_test(SIDE, SIDE, WatchShowcase);
    assert!(robot.find_by_text("Sun").click(), "the Sun row took no tap");
    settle(&mut robot);

    let tagline = "The star that lights the solar system";
    let texts = robot.get_all_text();
    assert!(
        shows(&texts, tagline) && !shows(&texts, "Star chart"),
        "the Sun's page did not take the list's place: {texts:?}"
    );

    // From the left edge to the right, as on a watch.
    robot.drag(2.0, MIDDLE, SIDE as f32 - 10.0, MIDDLE);
    settle(&mut robot);

    let texts = robot.get_all_text();
    assert!(
        !shows(&texts, tagline) && shows(&texts, "Star chart"),
        "the swipe did not return to the list: {texts:?}"
    );
}
