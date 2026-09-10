#![allow(non_snake_case)]

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;
use cranpose::BackHandler;
use cranpose_animation::prelude::{tween, Easing};

use crate::model::BODIES;
use crate::motion::rememberAmbientMotion;
use crate::screens::detail_screen::DetailScreen;
use crate::screens::list_screen::{ListScreen, Tab};

/// Builds the launcher used by every target except iOS, whose window is
/// owned and sized by UIKit instead.
#[cfg(not(target_os = "ios"))]
pub fn create_app() -> AppLauncher {
    AppLauncher::new()
        .with_title("Showcase Cranpose")
        .with_size(412, 915)
        .with_android_gpu_backend(cranpose::AndroidGpuBackend::OpenGlEs)
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
pub fn ShowcaseApp() {
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
    let tab = rememberMutableStateOf(|| Tab::Explore);
    let route = rememberMutableStateOf(|| Route::List);
    let favorites = rememberMutableStateOf(|| vec![false; BODIES.len()]);
    let system_bars = local_safe_area_insets().current();

    let destination = route.get();
    let showing_detail = matches!(destination, Route::Detail(_));
    let favorite_count = favorites.get().iter().filter(|&&favorite| favorite).count();

    let ambient = rememberAmbientMotion(showing_detail);

    let route_for_system_back = route;
    BackHandler(showing_detail, move || {
        route_for_system_back.set(Route::List)
    });

    Box(
        Modifier::empty().fill_max_size(),
        BoxSpec::default(),
        move || {
            let route_for_content = route;
            let favorites_for_content = favorites;
            Crossfade(
                destination,
                tween(260, Easing::EaseOut),
                move |destination| match destination {
                    Route::List => {
                        let current_tab = tab.get();
                        let on_open = move |index| route_for_content.set(Route::Detail(index));
                        ListScreen(
                            current_tab,
                            favorites_for_content,
                            favorite_count,
                            ambient,
                            on_open,
                        );
                        Box(
                            Modifier::empty()
                                .fill_max_width()
                                .padding_each(
                                    system_bars.left,
                                    0.0,
                                    system_bars.right,
                                    system_bars.bottom + 12.0,
                                )
                                .align(Alignment::new(
                                    HorizontalAlignment::CenterHorizontally,
                                    VerticalAlignment::Bottom,
                                )),
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
                    }
                    Route::Detail(index) => {
                        let is_favorite = favorites_for_content
                            .get()
                            .get(index)
                            .copied()
                            .unwrap_or(false);
                        let on_back = move || route_for_content.set(Route::List);
                        let on_toggle_favorite = move || {
                            let mut current = favorites_for_content.get();
                            if let Some(flag) = current.get_mut(index) {
                                *flag = !*flag;
                            }
                            favorites_for_content.set(current);
                        };
                        let on_open_related =
                            move |target| route_for_content.set(Route::Detail(target));
                        DetailScreen(
                            index,
                            is_favorite,
                            favorite_count,
                            ambient,
                            on_back,
                            on_toggle_favorite,
                            on_open_related,
                        );
                    }
                },
            );
        },
    );
}
