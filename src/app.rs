#![allow(non_snake_case)]

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;
use cranpose_animation::prelude::*;

use crate::model::BODIES;
use crate::motion::AmbientMotion;
use crate::screens::detail_screen::DetailScreen;
use crate::screens::list_screen::{ListScreen, Tab};
use crate::widgets::starfield::Starfield;

/// Builds the launcher used by every target except iOS, whose window is
/// owned and sized by UIKit instead.
#[cfg(not(target_os = "ios"))]
pub fn create_app() -> AppLauncher {
    AppLauncher::new()
        .with_title("Cranpose Orbit")
        .with_size(412, 915)
        .with_fps_counter(false)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Route {
    List,
    Detail(usize),
}

/// Root composable: the Liquid theme, pinned dark so the same star-chart
/// mood renders identically regardless of the platform's system appearance.
#[composable]
pub fn OrbitApp() {
    LiquidTheme(
        LiquidThemeSpec {
            scheme: SchemeMode::Dark,
            accent: Color::from_rgb_u8(142, 124, 255),
            ..LiquidThemeSpec::default()
        },
        RootShell,
    );
}

#[composable]
fn RootShell() {
    let insets = local_safe_area_insets().current();
    let tab = rememberMutableStateOf(|| Tab::Explore);
    let route = rememberMutableStateOf(|| Route::List);
    let favorites = rememberMutableStateOf(|| vec![false; BODIES.len()]);
    let last_detail = rememberMutableStateOf(|| 0usize);

    if let Route::Detail(index) = route.get() {
        if last_detail.get() != index {
            last_detail.set(index);
        }
    }

    let infinite = rememberInfiniteTransition("orbit-ambient");
    let sheen = infinite
        .animateFloat(
            0.0,
            1.0,
            infiniteRepeatable(
                AnimationSpec::tween(5200, Easing::EaseInOut),
                RepeatMode::Reverse,
                StartOffset::default(),
            ),
            "sheen",
        )
        .value();
    let twinkle = infinite
        .animateFloat(
            0.0,
            1.0,
            infiniteRepeatable(
                AnimationSpec::tween(3600, Easing::LinearEasing),
                RepeatMode::Restart,
                StartOffset::default(),
            ),
            "twinkle",
        )
        .value();
    let drift = infinite
        .animateFloat(
            0.0,
            1.0,
            infiniteRepeatable(
                AnimationSpec::tween(48_000, Easing::LinearEasing),
                RepeatMode::Restart,
                StartOffset::default(),
            ),
            "drift",
        )
        .value();
    let ambient = AmbientMotion {
        sheen,
        drift,
        twinkle,
    };

    let showing_detail = matches!(route.get(), Route::Detail(_));

    Box(
        Modifier::empty().fill_max_size(),
        BoxSpec::default(),
        move || {
            Starfield(Modifier::empty().fill_max_size(), drift, twinkle);

            Box(
                Modifier::empty().fill_max_size().padding_each(
                    insets.left,
                    insets.top,
                    insets.right,
                    insets.bottom,
                ),
                BoxSpec::default(),
                move || {
                    let current_tab = tab.get();
                    let route_for_open = route;
                    let on_open = move |index| route_for_open.set(Route::Detail(index));
                    ListScreen(current_tab, favorites, ambient, on_open);

                    let tab_bar_progress = animateFloatAsState(
                        if showing_detail { 0.0 } else { 1.0 },
                        spring(Spring::DampingRatioNoBouncy, Spring::StiffnessMedium),
                        "tab-bar-visibility",
                    )
                    .value();
                    Box(
                        Modifier::empty()
                            .fill_max_width()
                            .align(Alignment::new(
                                HorizontalAlignment::CenterHorizontally,
                                VerticalAlignment::Bottom,
                            ))
                            .graphics_layer_block(move |layer| {
                                layer.alpha = tab_bar_progress;
                                layer.translation_y = (1.0 - tab_bar_progress) * 48.0;
                            }),
                        BoxSpec::default().content_alignment(Alignment::new(
                            HorizontalAlignment::CenterHorizontally,
                            VerticalAlignment::Bottom,
                        )),
                        move || {
                            LiquidTabBar(
                                Modifier::empty(),
                                LiquidTabBarSpec::default(),
                                match current_tab {
                                    Tab::Explore => 0,
                                    Tab::Saved => 1,
                                },
                                move |index| {
                                    tab.set(if index == 0 { Tab::Explore } else { Tab::Saved });
                                },
                                |scope| {
                                    scope.tab(icons::ROCKET, "Explore");
                                    scope.tab(icons::BOOKMARK, "Saved");
                                },
                            );
                        },
                    );

                    let enter = (fade_in() + slide_in_vertically(0.05)).with_animation(spring(
                        Spring::DampingRatioLowBouncy,
                        Spring::StiffnessMediumLow,
                    ));
                    let exit = (fade_out() + slide_out_vertically(0.04)).with_animation(spring(
                        Spring::DampingRatioNoBouncy,
                        Spring::StiffnessMedium,
                    ));
                    let route_for_back = route;
                    let favorites_for_detail = favorites;
                    let route_for_toggle = route;
                    AnimatedVisibility(showing_detail, enter, exit, move || {
                        let index = last_detail.get();
                        let is_favorite = favorites_for_detail
                            .get()
                            .get(index)
                            .copied()
                            .unwrap_or(false);
                        let on_back = move || route_for_back.set(Route::List);
                        let on_toggle_favorite = move || {
                            let mut current = favorites_for_detail.get();
                            if let Some(flag) = current.get_mut(index) {
                                *flag = !*flag;
                            }
                            favorites_for_detail.set(current);
                        };
                        let on_open_related =
                            move |target| route_for_toggle.set(Route::Detail(target));
                        DetailScreen(
                            index,
                            is_favorite,
                            ambient,
                            on_back,
                            on_toggle_favorite,
                            on_open_related,
                        );
                    });
                },
            );
        },
    );
}
