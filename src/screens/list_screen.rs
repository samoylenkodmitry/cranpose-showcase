#![allow(non_snake_case)]

use std::rc::Rc;
use std::time::Duration;

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;
use cranpose_animation::prelude::*;
use cranpose_core::rememberKeyed;
use cranpose_foundation::text::TextFieldState;
use cranpose_ui::text::{FontWeight, TextUnit};
use cranpose_ui_graphics::Stroke;

use crate::model::{BodyKind, CelestialBody, BODIES};
use crate::motion::AmbientMotion;
use crate::widgets::planet::PlanetSphere;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Explore,
    Saved,
}

const CATEGORIES: &[(&str, Option<BodyKind>)] = &[
    ("All", None),
    ("Planets", Some(BodyKind::Planet)),
    ("Moons", Some(BodyKind::Moon)),
    ("Dwarf", Some(BodyKind::DwarfPlanet)),
    ("Star", Some(BodyKind::Star)),
    ("Exoplanet", Some(BodyKind::Exoplanet)),
];

fn kind_color(colors: LiquidColors, kind: BodyKind) -> Color {
    match kind {
        BodyKind::Star => Color::from_rgb_u8(255, 189, 89),
        BodyKind::Planet => colors.accent,
        BodyKind::DwarfPlanet => Color::from_rgb_u8(178, 168, 214),
        BodyKind::Moon => Color::from_rgb_u8(158, 176, 196),
        BodyKind::Exoplanet => Color::from_rgb_u8(224, 118, 118),
    }
}

#[composable]
fn KindTag(kind: BodyKind) {
    let colors = liquid_colors();
    let tint = kind_color(colors, kind);
    Box(
        Modifier::empty()
            .background(Color::from_rgba_u8(
                (tint.0 * 255.0) as u8,
                (tint.1 * 255.0) as u8,
                (tint.2 * 255.0) as u8,
                46,
            ))
            .rounded_corners(7.0)
            .padding_each(7.0, 2.0, 7.0, 2.0),
        BoxSpec::default(),
        move || {
            Text(
                kind.label(),
                Modifier::empty(),
                TextStyle {
                    span_style: SpanStyle {
                        font_size: TextUnit::Sp(11.0),
                        font_weight: Some(FontWeight::SEMI_BOLD),
                        color: Some(tint),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        },
    );
}

#[composable]
pub(crate) fn FavoriteButton(favorite: bool, on_toggle: impl Fn() + 'static) {
    let colors = liquid_colors();
    let previous = rememberMutableStateOf(|| favorite);
    let pulse = rememberMutableStateOf(|| 1.0f32);
    let scope = rememberCoroutineScope();

    if previous.get() != favorite {
        previous.set(favorite);
        pulse.set(1.45);
        scope.launch(async move {
            delay(Duration::from_millis(90)).await;
            pulse.set(1.0);
        });
    }

    let scale = animateFloatAsState(
        pulse.get(),
        spring(Spring::DampingRatioHighBouncy, Spring::StiffnessMedium),
        "favorite-scale",
    )
    .value();
    let tint = animateColorAsState(
        if favorite {
            Color::from_rgb_u8(255, 196, 60)
        } else {
            colors.tertiary_label
        },
        tween(220, Easing::EaseOut),
        "favorite-color",
    )
    .value();

    let handler = on_toggle;
    GlassIconButton(
        Modifier::empty().graphics_layer_block(move |layer| {
            layer.scale = scale;
        }),
        GlassButtonSpec::glass().with_content_color(tint),
        40.0,
        move || handler(),
        icons::STAR,
    );
}

#[composable]
fn BodyCard(
    index: usize,
    body: &'static CelestialBody,
    favorite: bool,
    ambient: AmbientMotion,
    on_toggle_favorite: impl Fn() + 'static,
    on_open: impl Fn() + 'static,
) {
    let on_toggle_favorite: Rc<dyn Fn()> = Rc::new(on_toggle_favorite);
    let scope = rememberCoroutineScope();
    let appeared = rememberMutableStateOf(|| false);
    remember(|| {
        scope.launch(async move {
            delay(Duration::from_millis(35 * (index.min(14)) as u64)).await;
            appeared.set(true);
        });
    });

    let progress = animateFloatAsState(
        if appeared.get() { 1.0 } else { 0.0 },
        spring(Spring::DampingRatioLowBouncy, Spring::StiffnessLow),
        "card-appear",
    )
    .value();

    let opener = on_open;
    LiquidCard(
        Modifier::empty()
            .fill_max_width()
            .graphics_layer_block(move |layer| {
                layer.alpha = progress;
                layer.translation_y = (1.0 - progress) * 28.0;
            })
            .clickable(move |_point| opener()),
        move || {
            let colors = liquid_colors();
            let on_toggle_favorite = on_toggle_favorite.clone();
            Row(
                Modifier::empty().fill_max_width().padding(14.0),
                RowSpec::default().vertical_alignment(VerticalAlignment::CenterVertically),
                move || {
                    PlanetSphere(
                        Modifier::empty().size(Size {
                            width: 60.0,
                            height: 60.0,
                        }),
                        body,
                        ambient,
                    );
                    Box(Modifier::empty().width(14.0), BoxSpec::default(), || {});
                    Column(
                        Modifier::empty().weight(1.0),
                        ColumnSpec::default()
                            .vertical_arrangement(LinearArrangement::spaced_by(5.0)),
                        move || {
                            Row(
                                Modifier::empty().fill_max_width(),
                                RowSpec::default()
                                    .vertical_alignment(VerticalAlignment::CenterVertically),
                                move || {
                                    Text(
                                        body.name,
                                        Modifier::empty(),
                                        liquid_typography().headline.merge(&TextStyle {
                                            span_style: SpanStyle {
                                                color: Some(colors.label),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        }),
                                    );
                                    Box(Modifier::empty().width(8.0), BoxSpec::default(), || {});
                                    KindTag(body.kind);
                                },
                            );
                            Text(
                                body.tagline,
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
                    Box(Modifier::empty().width(6.0), BoxSpec::default(), || {});
                    let toggle = on_toggle_favorite.clone();
                    FavoriteButton(favorite, move || toggle());
                },
            );
        },
    );
}

#[composable]
fn EmptySavedState() {
    let colors = liquid_colors();
    let infinite = rememberInfiniteTransition("empty-bob");
    let bob = infinite
        .animateFloat(
            0.0,
            1.0,
            infiniteRepeatable(
                AnimationSpec::tween(1800, Easing::EaseInOut),
                RepeatMode::Reverse,
                StartOffset::default(),
            ),
            "bob",
        )
        .value();

    Column(
        Modifier::empty()
            .fill_max_width()
            .padding_each(32.0, 96.0, 32.0, 32.0),
        ColumnSpec::default()
            .horizontal_alignment(HorizontalAlignment::CenterHorizontally)
            .vertical_arrangement(LinearArrangement::spaced_by(14.0)),
        move || {
            Box(
                Modifier::empty()
                    .size(Size {
                        width: 96.0,
                        height: 96.0,
                    })
                    .graphics_layer_block(move |layer| {
                        layer.translation_y = (bob - 0.5) * 10.0;
                    })
                    .draw_behind(move |scope| {
                        let size = scope.size();
                        let center = Point {
                            x: size.width * 0.5,
                            y: size.height * 0.5,
                        };
                        scope.draw_circle_stroked(
                            Brush::solid(Color::from_rgba_u8(255, 255, 255, 60)),
                            center,
                            size.width * 0.42,
                            Stroke::new(2.0),
                        );
                    }),
                BoxSpec::default().content_alignment(Alignment::CENTER),
                || {
                    icons::Icon(icons::STAR, 36.0, Color::from_rgba_u8(255, 255, 255, 90));
                },
            );
            Text(
                "No saved worlds yet",
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
                "Tap the star on any world in Explore to keep it here.",
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
}

/// The main scrolling screen: search, category chips, and the list of
/// bodies, topped by a large-title nav bar that collapses as the content
/// scrolls beneath it.
#[composable]
pub fn ListScreen(
    tab: Tab,
    favorites: MutableState<Vec<bool>>,
    ambient: AmbientMotion,
    on_open: impl Fn(usize) + 'static,
) -> f32 {
    let on_open: Rc<dyn Fn(usize)> = Rc::new(on_open);
    let scroll = rememberKeyed(tab, |_| ScrollState::new(0.0));
    let category: MutableState<usize> = rememberKeyed(tab, |_| mutableStateOf(0usize));
    let search = remember(|| TextFieldState::new("")).with(|state| *state);
    let chip_scroll = remember(|| ScrollState::new(0.0)).with(|state| *state);

    let title = match tab {
        Tab::Explore => "Explore",
        Tab::Saved => "Saved",
    };

    Box(
        Modifier::empty().fill_max_size(),
        BoxSpec::default(),
        move || {
            let colors = liquid_colors();
            let on_open = on_open.clone();
            Column(
                Modifier::empty()
                    .fill_max_size()
                    .vertical_scroll(scroll, false)
                    .padding_each(20.0, liquid_nav_bar_expanded_height() + 10.0, 20.0, 128.0),
                ColumnSpec::default().vertical_arrangement(LinearArrangement::spaced_by(14.0)),
                move || {
                    let favorite_flags = favorites.get();
                    let query = search.text().trim().to_lowercase();
                    let selected_category = category.get().min(CATEGORIES.len() - 1);
                    let visible: Vec<usize> = BODIES
                        .iter()
                        .enumerate()
                        .filter(|(index, entry)| {
                            let tab_ok = match tab {
                                Tab::Explore => true,
                                Tab::Saved => favorite_flags.get(*index).copied().unwrap_or(false),
                            };
                            let category_ok = match CATEGORIES[selected_category].1 {
                                None => true,
                                Some(kind) => entry.kind == kind,
                            };
                            let query_ok =
                                query.is_empty() || entry.name.to_lowercase().contains(&query);
                            tab_ok && category_ok && query_ok
                        })
                        .map(|(index, _)| index)
                        .collect();

                    SearchField(Modifier::empty().fill_max_width(), search, "Search the sky");

                    Row(
                        Modifier::empty()
                            .fill_max_width()
                            .horizontal_scroll(chip_scroll, false),
                        RowSpec::default()
                            .horizontal_arrangement(LinearArrangement::spaced_by(8.0)),
                        move || {
                            for (index, (label, _)) in CATEGORIES.iter().enumerate() {
                                let category = category;
                                LiquidChip(
                                    Modifier::empty(),
                                    selected_category == index,
                                    move || category.set(index),
                                    *label,
                                );
                            }
                        },
                    );

                    if visible.is_empty() {
                        if matches!(tab, Tab::Saved) && query.is_empty() && selected_category == 0 {
                            EmptySavedState();
                        } else {
                            Text(
                                "No worlds match that search.",
                                Modifier::empty().padding_each(4.0, 24.0, 4.0, 0.0),
                                liquid_typography().body.merge(&TextStyle {
                                    span_style: SpanStyle {
                                        color: Some(colors.secondary_label),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            );
                        }
                    } else {
                        for (order, body_index) in visible.iter().copied().enumerate() {
                            let body = &BODIES[body_index];
                            let is_favorite =
                                favorite_flags.get(body_index).copied().unwrap_or(false);
                            let toggle = move || {
                                let mut current = favorites.get();
                                if let Some(flag) = current.get_mut(body_index) {
                                    *flag = !*flag;
                                }
                                favorites.set(current);
                            };
                            let on_open = on_open.clone();
                            let opener = move || on_open(body_index);
                            BodyCard(order, body, is_favorite, ambient, toggle, opener);
                        }
                    }
                },
            );

            LiquidNavBar(
                Modifier::empty().fill_max_width(),
                LiquidNavBarSpec::new(title),
                scroll,
                || {},
                || {},
            );
        },
    );

    scroll.value()
}
