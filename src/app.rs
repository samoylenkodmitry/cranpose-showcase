#![allow(non_snake_case)]

use std::rc::Rc;

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;
use cranpose::BackHandler;
use cranpose_animation::prelude::{tween, Easing};

use crate::model::BODIES;
use crate::motion::{rememberAmbientMotion, AmbientMotion};
use crate::screens::detail_screen::DetailScreen;
use crate::screens::list_screen::{ListScreen, Tab};
use crate::widgets::starfield::{Starfield, StarfieldScroll};
use crate::widgets::surfaces::showcase_glass;

/// Where the single-column phone layout gives way to the list/detail split.
const SPLIT_LAYOUT_MIN_WIDTH: f32 = 880.0;
/// The list column's width once the layout splits. A row never stretches past
/// this, however wide the window gets.
const LIST_PANE_WIDTH: f32 = 420.0;

/// Builds the launcher used by every target except iOS, whose window is
/// owned and sized by UIKit instead.
#[cfg(not(target_os = "ios"))]
pub fn create_app() -> AppLauncher {
    AppLauncher::new()
        .with_title("Showcase Cranpose")
        .with_size(412, 915)
        .with_fps_counter(false)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Route {
    List,
    Detail(usize),
}

/// Everything both layouts need: the navigation state, the favourites, and
/// the ambient animation the starfield and every planet read.
#[derive(Clone, Copy, PartialEq)]
struct ShellState {
    tab: MutableState<Tab>,
    route: MutableState<Route>,
    favorites: MutableState<Vec<bool>>,
    destination: Route,
    favorite_count: usize,
    ambient: AmbientMotion,
}

impl ShellState {
    fn open(self, index: usize) {
        self.route.set(Route::Detail(index));
    }

    fn toggle_favorite(self, index: usize) {
        let mut current = self.favorites.get();
        if let Some(flag) = current.get_mut(index) {
            *flag = !*flag;
        }
        self.favorites.set(current);
    }

    fn is_favorite(self, index: usize) -> bool {
        self.favorites.get().get(index).copied().unwrap_or(false)
    }
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

    let destination = route.get();
    let showing_detail = matches!(destination, Route::Detail(_));
    let favorite_count = favorites.get().iter().filter(|&&favorite| favorite).count();
    let ambient = rememberAmbientMotion(showing_detail);
    let state = ShellState {
        tab,
        route,
        favorites,
        destination,
        favorite_count,
        ambient,
    };

    BackHandler(showing_detail, move || route.set(Route::List));

    // The shell reports its own size rather than subcomposing under a
    // `BoxWithConstraints`: everything below here animates, and an animated
    // read inside a measure-phase subcomposition has no invalidation path.
    let shell_size = rememberMutableStateOf(|| Size {
        width: 0.0,
        height: 0.0,
    });
    let split = shell_size.get().width >= SPLIT_LAYOUT_MIN_WIDTH;
    Box(
        Modifier::empty()
            .fill_max_size()
            .report_size_state(shell_size),
        BoxSpec::default(),
        move || {
            if split {
                SplitShell(state);
            } else {
                StackedShell(state);
            }
        },
    );
}

/// Phone and portrait tablet: one screen at a time, crossfading between the
/// list and a body's detail.
#[composable]
fn StackedShell(state: ShellState) {
    Crossfade(
        state.destination,
        tween(260, Easing::EaseOut),
        move |destination| match destination {
            Route::List => ListPane(state),
            Route::Detail(index) => BodyDetail(state, index, true),
        },
    );
}

/// Desktop and landscape tablet: the list keeps its phone width in a column
/// of its own and the selected body fills the rest of the window, so nothing
/// is stretched across a wide display.
#[composable]
fn SplitShell(state: ShellState) {
    Row(
        Modifier::empty().fill_max_size(),
        RowSpec::default(),
        move || {
            Box(
                Modifier::empty().width(LIST_PANE_WIDTH).fill_max_height(),
                BoxSpec::default(),
                move || ListPane(state),
            );
            Box(
                Modifier::empty().weight(1.0).fill_max_height(),
                BoxSpec::default(),
                move || {
                    Crossfade(
                        state.destination,
                        tween(260, Easing::EaseOut),
                        move |destination| match destination {
                            Route::List => NoSelectionPane(state),
                            Route::Detail(index) => BodyDetail(state, index, false),
                        },
                    );
                },
            );
        },
    );
}

/// The body list with the Explore/Saved tab bar docked over its bottom.
#[composable]
fn ListPane(state: ShellState) {
    let system_bars = local_safe_area_insets().current();
    let tab = state.tab;
    // Read here and not in the shell above: the list's own state is keyed by
    // the tab, and a tab read further up re-keys it from outside its scope.
    let current_tab = tab.get();
    ListScreen(
        current_tab,
        state.favorites,
        state.favorite_count,
        state.ambient,
        move |index| state.open(index),
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

/// One body's page. `with_back` is false where the list stays beside it and
/// there is nowhere to go back to.
#[composable]
fn BodyDetail(state: ShellState, index: usize, with_back: bool) {
    let on_back = with_back.then(|| {
        let handler: Rc<dyn Fn()> = Rc::new(move || state.route.set(Route::List));
        handler
    });
    DetailScreen(
        index,
        state.is_favorite(index),
        state.favorite_count,
        state.ambient,
        on_back,
        move || state.toggle_favorite(index),
        move |target| state.open(target),
    );
}

/// What the detail half shows before a body is picked.
#[composable]
fn NoSelectionPane(state: ShellState) {
    let colors = liquid_colors();
    let scroll = remember(|| ScrollState::new(0.0)).with(|value| *value);
    Box(
        Modifier::empty().fill_max_size(),
        BoxSpec::default().content_alignment(Alignment::CENTER),
        move || {
            Starfield(
                Modifier::empty().fill_max_size(),
                StarfieldScroll::Scroll(scroll),
                state.ambient,
                state.favorite_count,
            );
            GlassSurface(
                Modifier::empty().padding(28.0),
                showcase_glass(colors, 24.0),
                move || {
                    Column(
                        Modifier::empty(),
                        ColumnSpec::default()
                            .horizontal_alignment(HorizontalAlignment::CenterHorizontally)
                            .vertical_arrangement(LinearArrangement::spaced_by(10.0)),
                        move || {
                            icons::Icon(
                                icons::ROCKET,
                                34.0,
                                Color::from_rgba_u8(255, 255, 255, 120),
                            );
                            Text(
                                "Pick a world",
                                Modifier::empty(),
                                liquid_typography().title3.merge(&TextStyle {
                                    span_style: SpanStyle {
                                        color: Some(colors.label),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            );
                            Text(
                                "Choose anything in the list to open it here.",
                                Modifier::empty(),
                                liquid_typography().subheadline.merge(&TextStyle {
                                    span_style: SpanStyle {
                                        color: Some(colors.secondary_label),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            );
                        },
                    );
                },
            );
        },
    );
}
