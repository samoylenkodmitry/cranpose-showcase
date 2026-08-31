#![allow(non_snake_case)]

use std::rc::Rc;

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;
use cranpose_core::rememberKeyed;
use cranpose_ui::text::{FontWeight, TextUnit};

use crate::model::{CelestialBody, BODIES};
use crate::motion::AmbientMotion;
use crate::screens::list_screen::FavoriteButton;
use crate::widgets::header_glass::HeaderBlurGradient;
use crate::widgets::planet::PlanetSphere;
use crate::widgets::starfield::Starfield;
use crate::widgets::surfaces::showcase_glass;

fn secondary_style(colors: LiquidColors, base: TextStyle) -> TextStyle {
    base.merge(&TextStyle {
        span_style: SpanStyle {
            color: Some(colors.secondary_label),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn primary_style(colors: LiquidColors, base: TextStyle) -> TextStyle {
    base.merge(&TextStyle {
        span_style: SpanStyle {
            color: Some(colors.label),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[composable]
fn Hero(body: &'static CelestialBody, ambient: AmbientMotion) {
    let colors = liquid_colors();
    Column(
        Modifier::empty().fill_max_width(),
        ColumnSpec::default()
            .horizontal_alignment(HorizontalAlignment::CenterHorizontally)
            .vertical_arrangement(LinearArrangement::spaced_by(14.0)),
        move || {
            PlanetSphere(
                Modifier::empty().size(Size {
                    width: 208.0,
                    height: 208.0,
                }),
                body,
                ambient,
            );
            Text(
                body.tagline,
                Modifier::empty().padding_each(32.0, 0.0, 32.0, 0.0),
                secondary_style(colors, liquid_typography().title3),
            );
        },
    );
}

#[composable]
fn StatTile(label: &'static str, value: &'static str) {
    let colors = liquid_colors();
    GlassSurface(
        Modifier::empty().weight(1.0).padding(14.0),
        showcase_glass(colors, 16.0),
        move || {
            Column(
                Modifier::empty().fill_max_width(),
                ColumnSpec::default().vertical_arrangement(LinearArrangement::spaced_by(5.0)),
                move || {
                    Text(
                        label,
                        Modifier::empty(),
                        TextStyle {
                            span_style: SpanStyle {
                                font_size: TextUnit::Sp(11.0),
                                font_weight: Some(FontWeight::SEMI_BOLD),
                                letter_spacing: TextUnit::Sp(0.6),
                                color: Some(colors.tertiary_label),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    );
                    Text(
                        value,
                        Modifier::empty(),
                        primary_style(colors, liquid_typography().headline),
                    );
                },
            );
        },
    );
}

#[composable]
fn StatsGrid(body: &'static CelestialBody) {
    Column(
        Modifier::empty()
            .fill_max_width()
            .padding_each(20.0, 0.0, 20.0, 0.0),
        ColumnSpec::default().vertical_arrangement(LinearArrangement::spaced_by(10.0)),
        move || {
            Row(
                Modifier::empty().fill_max_width(),
                RowSpec::default().horizontal_arrangement(LinearArrangement::spaced_by(10.0)),
                move || {
                    StatTile("DISTANCE", body.distance);
                    StatTile("DIAMETER", body.diameter);
                },
            );
            Row(
                Modifier::empty().fill_max_width(),
                RowSpec::default().horizontal_arrangement(LinearArrangement::spaced_by(10.0)),
                move || {
                    StatTile("DAY LENGTH", body.day_length);
                    StatTile("YEAR LENGTH", body.year_length);
                },
            );
            Row(
                Modifier::empty().fill_max_width(),
                RowSpec::default().horizontal_arrangement(LinearArrangement::spaced_by(10.0)),
                move || {
                    StatTile("MOONS", body.moons);
                    StatTile("GRAVITY", body.gravity_label);
                },
            );
        },
    );
}

#[composable]
fn Description(body: &'static CelestialBody) {
    let colors = liquid_colors();
    Text(
        body.description,
        Modifier::empty()
            .fill_max_width()
            .padding_each(20.0, 4.0, 20.0, 0.0),
        secondary_style(colors, liquid_typography().body),
    );
}

#[composable]
fn GravityPlayground(body: &'static CelestialBody) {
    let colors = liquid_colors();
    let earth_weight = rememberKeyed(body.name, |_| mutableStateOf(70.0f32));
    Box(
        Modifier::empty()
            .fill_max_width()
            .padding_each(20.0, 0.0, 20.0, 0.0),
        BoxSpec::default(),
        move || {
            let weight = earth_weight.get();
            let here = weight * body.gravity_g;
            GlassSurface(
                Modifier::empty().fill_max_width().padding(16.0),
                showcase_glass(colors, 18.0),
                move || {
                    Column(
                        Modifier::empty().fill_max_width(),
                        ColumnSpec::default()
                            .vertical_arrangement(LinearArrangement::spaced_by(10.0)),
                        move || {
                            Text(
                                "Feel the gravity",
                                Modifier::empty(),
                                primary_style(colors, liquid_typography().headline),
                            );
                            Text(
                                format!(
                                    "At {weight:.0} kg on Earth, you would weigh {here:.0} kg on {}.",
                                    body.name
                                ),
                                Modifier::empty(),
                                secondary_style(colors, liquid_typography().subheadline),
                            );
                            LiquidSlider(
                                Modifier::empty().fill_max_width(),
                                ((weight - 30.0) / 150.0).clamp(0.0, 1.0),
                                move |fraction| earth_weight.set(30.0 + fraction * 150.0),
                            );
                        },
                    );
                },
            );
        },
    );
}

#[composable]
fn RelatedTile(body: &'static CelestialBody, ambient: AmbientMotion, on_open: impl Fn() + 'static) {
    let colors = liquid_colors();
    Column(
        Modifier::empty()
            .width(92.0)
            .clickable(move |_point| on_open()),
        ColumnSpec::default()
            .horizontal_alignment(HorizontalAlignment::CenterHorizontally)
            .vertical_arrangement(LinearArrangement::spaced_by(8.0)),
        move || {
            PlanetSphere(
                Modifier::empty().size(Size {
                    width: 68.0,
                    height: 68.0,
                }),
                body,
                ambient,
            );
            Text(
                body.name,
                Modifier::empty(),
                secondary_style(colors, liquid_typography().footnote),
            );
        },
    );
}

#[composable]
fn RelatedRow(
    body: &'static CelestialBody,
    ambient: AmbientMotion,
    on_open: impl Fn(usize) + 'static,
) {
    if body.related.is_empty() {
        return;
    }
    let on_open: Rc<dyn Fn(usize)> = Rc::new(on_open);
    let colors = liquid_colors();
    let scroll = rememberKeyed(body.name, |_| ScrollState::new(0.0));
    let related = body.related;
    Column(
        Modifier::empty().fill_max_width(),
        ColumnSpec::default().vertical_arrangement(LinearArrangement::spaced_by(10.0)),
        move || {
            let on_open = on_open.clone();
            Text(
                "RELATED WORLDS",
                Modifier::empty().padding_each(20.0, 0.0, 20.0, 0.0),
                TextStyle {
                    span_style: SpanStyle {
                        font_size: TextUnit::Sp(12.0),
                        font_weight: Some(FontWeight::SEMI_BOLD),
                        letter_spacing: TextUnit::Sp(0.6),
                        color: Some(colors.tertiary_label),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            Row(
                Modifier::empty()
                    .fill_max_width()
                    .horizontal_scroll(scroll, false)
                    .padding_each(20.0, 0.0, 20.0, 0.0),
                RowSpec::default().horizontal_arrangement(LinearArrangement::spaced_by(14.0)),
                move || {
                    for &target in related {
                        let related_body = &BODIES[target];
                        let on_open = on_open.clone();
                        RelatedTile(related_body, ambient, move || on_open(target));
                    }
                },
            );
        },
    );
}

/// The full-screen detail page for one celestial body: a hero sphere, a
/// stat grid, its description, an interactive gravity toy, and a horizontal
/// strip of related worlds. `ambient` is the shared ambient animation state
/// so the hero's highlight and the backdrop drift in sync with the list
/// behind it.
#[composable]
pub fn DetailScreen(
    body_index: usize,
    favorite: bool,
    favorite_count: usize,
    ambient: AmbientMotion,
    on_back: impl Fn() + 'static,
    on_toggle_favorite: impl Fn() + 'static,
    on_open_related: impl Fn(usize) + 'static,
) {
    let on_back: Rc<dyn Fn()> = Rc::new(on_back);
    let on_toggle_favorite: Rc<dyn Fn()> = Rc::new(on_toggle_favorite);
    let on_open_related: Rc<dyn Fn(usize)> = Rc::new(on_open_related);
    let body = &BODIES[body_index];
    let scroll = rememberKeyed(body_index, |_| ScrollState::new(0.0));

    Box(
        Modifier::empty().fill_max_size(),
        BoxSpec::default(),
        move || {
            Starfield(
                Modifier::empty().fill_max_size(),
                scroll.value(),
                ambient.twinkle,
                favorite_count,
            );
            let on_toggle_favorite = on_toggle_favorite.clone();
            let on_open_related = on_open_related.clone();
            Column(
                Modifier::empty()
                    .fill_max_size()
                    .vertical_scroll(scroll, false)
                    .padding_each(0.0, 88.0, 0.0, 48.0),
                ColumnSpec::default().vertical_arrangement(LinearArrangement::spaced_by(22.0)),
                move || {
                    let on_open_related = on_open_related.clone();
                    Hero(body, ambient);
                    StatsGrid(body);
                    Description(body);
                    GravityPlayground(body);
                    RelatedRow(body, ambient, move |target| on_open_related(target));
                },
            );
            HeaderBlurGradient();

            Row(
                Modifier::empty()
                    .fill_max_width()
                    .padding_each(16.0, 18.0, 16.0, 0.0),
                RowSpec::default()
                    .horizontal_arrangement(LinearArrangement::SpaceBetween)
                    .vertical_alignment(VerticalAlignment::CenterVertically),
                {
                    let on_back = on_back.clone();
                    let on_toggle_favorite = on_toggle_favorite.clone();
                    move || {
                        let back = on_back.clone();
                        GlassIconButton(
                            Modifier::empty(),
                            GlassButtonSpec::glass(),
                            40.0,
                            move || back(),
                            icons::CHEVRON_LEFT,
                        );
                        let toggle = on_toggle_favorite.clone();
                        FavoriteButton(favorite, move || toggle());
                    }
                },
            );
        },
    );
}
