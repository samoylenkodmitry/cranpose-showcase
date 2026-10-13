//! The watch app: the bodies of the phone app in a list the crown scrolls,
//! and a page for each body.
//!
//! The same composable runs on both watches. On Wear OS the `watch` Cargo
//! feature makes [`WatchShowcase`] the root of the Android app (see
//! `android/wear`). On Apple Watch, Cranpose's watchOS packager calls it: the
//! `[package.metadata.cranpose.watchos]` table in `Cargo.toml` names it.
//!
//! The Apple Watch host draws through SpriteKit with Cranpose's pixel
//! renderer, which runs none of the phone app's shaders. So the watch app
//! uses no Liquid glass and no planet shader: its bodies are circles and
//! gradients, and it looks the same on both watches.
//!
//! The data and the view models are the phone app's. [`AppServices`] holds
//! the saved bodies and the facts, and a body's page resolves the same
//! `BodyCardViewModel` a card on the phone resolves.

#![allow(non_snake_case)]

mod detail;
mod list;
mod planet;
mod theme;

use cranpose::prelude::*;
use cranpose_animation::prelude::{tween, Easing};
use cranpose_coroflow::rememberHandle;
use cranpose_navigation::{rememberNavController, NavHostWith};
use cranpose_ui::round_scaling_list::CentreAnchor;
use cranpose_ui::widgets::wear::rememberWearScalingListState;

use crate::data::AppServices;
use crate::model::BODIES;

/// The window of the desktop preview, in points: the size of a 454 pixel
/// round Wear OS screen at density 2.
const PREVIEW_SIDE: u32 = 227;

/// Where the watch app can go.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WatchRoute {
    List,
    Body(usize),
}

/// The launcher of the Wear OS app, and of the desktop window that previews
/// the watch app (`cargo run --features watch`).
pub fn create_watch_app() -> AppLauncher {
    AppLauncher::new()
        .with_title("Showcase")
        .with_size(PREVIEW_SIDE, PREVIEW_SIDE)
        .with_fps_counter(false)
        // The inspector's button in debug builds would cover a fifth of a
        // watch screen.
        .with_developer_inspector(false)
}

/// Root composable of the watch app.
#[composable]
pub fn WatchShowcase() {
    let services = rememberHandle(AppServices::default);
    let nav = rememberNavController(WatchRoute::List);
    // The list's position lives here, above the screens, so the list is where
    // it was when a body's page closes. Wear starts a list with its second
    // item on the centre line, which puts the header above the first body.
    let list_position = rememberWearScalingListState(CentreAnchor::default());
    // `--body=saturn` at launch opens that body's page over the list, for
    // screenshots and for checks of one page.
    LaunchedEffect((), move |_| {
        if let Some(index) = launch_body() {
            nav.navigate(WatchRoute::Body(index));
        }
    });
    Box(
        Modifier::empty()
            .fill_max_size()
            .background(theme::BACKGROUND),
        BoxSpec::default(),
        move || {
            planet::StarBackdrop(Modifier::empty().fill_max_size());
            // A watch has no back button, and on Wear OS the app draws the
            // whole screen, so the system's own swipe never reaches it. A
            // swipe from the left edge goes from a body back to the list, and
            // from the list out of the app.
            SwipeToDismissBox(
                Modifier::empty().fill_max_size(),
                move || {
                    if !nav.navigate_up() {
                        request_exit();
                    }
                },
                move || {
                    NavHostWith(nav, tween(220, Easing::EaseOut), move |route| match route {
                        WatchRoute::List => list::BodyList(services, list_position, move |index| {
                            nav.navigate(WatchRoute::Body(index));
                        }),
                        WatchRoute::Body(index) => detail::BodyPage(services, index, move || {
                            nav.navigate_up();
                        }),
                    });
                },
            );
        },
    );
}

/// The body the `body` launch argument names, such as `saturn` or
/// `proxima-centauri-b`.
fn launch_body() -> Option<usize> {
    let args = launch_args();
    let name = args.string("body")?.replace('-', " ");
    BODIES
        .iter()
        .position(|body| body.name.eq_ignore_ascii_case(&name))
}
