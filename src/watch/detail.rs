//! One body's page on the watch: the body, its numbers, a fact and its
//! description, in a list the crown scrolls.

use std::rc::Rc;

use cranpose::prelude::*;
use cranpose_coroflow::{viewModel, Handle, StateFlowCollect};
use cranpose_ui::round_scaling_list::CentreAnchor;
use cranpose_ui::text::paragraph::TextAlign;
use cranpose_ui::widgets::wear::{
    rememberWearScalingListState, ScreenScaffold, WearButton, WearButtonSpec,
    WearScalingLazyColumn, WearTextStyle,
};

use super::planet::PlanetDisc;
use super::theme;
use crate::data::AppServices;
use crate::model::BODIES;
use crate::presentation::{
    body_card_view_model::BodyCardViewModel, screen_messages::ScreenMessages,
};

/// The side of the body at the top of the page.
const HERO_SIDE: f32 = 96.0;
/// How far below the centre line the page starts.
const HERO_DROP: f32 = 20.0;
/// Space between a line of text and the edges of the list.
const TEXT_SIDE_PADDING: f32 = 8.0;

/// Body `index`'s page. `on_back` returns to the list.
///
/// The page resolves the same `BodyCardViewModel` a card on the phone does,
/// from the store `NavHost` gives this page: saving and the fact come from
/// the view model, and the page only draws them.
#[composable]
pub fn BodyPage(services: Handle<AppServices>, index: usize, on_back: impl Fn() + 'static) {
    let on_back: Rc<dyn Fn()> = Rc::new(on_back);
    let messages = viewModel((), |_| ScreenMessages::default());
    let page = viewModel(index, move |scope| {
        let services = services.get();
        BodyCardViewModel::new(
            scope,
            index,
            services.favorites.clone(),
            services.facts.clone(),
            messages.get(),
        )
    });
    let saved = page.get().saved().collectAsStateWithLifecycle().get();
    let fact = page.get().fact().collectAsStateWithLifecycle().get();
    let body = &BODIES[index];
    let colors = theme::colors();

    // The name on the centre line and the body above it, both a little
    // lower, so the body clears the clock of an Apple Watch and the narrow
    // top of a round screen.
    let list = rememberWearScalingListState(CentreAnchor {
        index: 1,
        offset: -HERO_DROP,
    });
    ScreenScaffold(
        Modifier::empty().fill_max_size(),
        list,
        theme::scaffold_spec(),
        move || {
            let fact = fact.clone();
            let on_back = on_back.clone();
            WearScalingLazyColumn(
                Modifier::empty().fill_max_size(),
                list,
                theme::list_spec(),
                move |scope| {
                    scope.item(move || {
                        Box(
                            Modifier::empty().fill_max_width(),
                            BoxSpec::default().content_alignment(Alignment::CENTER),
                            move || {
                                PlanetDisc(
                                    Modifier::empty().size(Size {
                                        width: HERO_SIDE,
                                        height: HERO_SIDE,
                                    }),
                                    index,
                                    true,
                                );
                            },
                        );
                    });
                    scope.item(move || {
                        Column(
                            Modifier::empty().fill_max_width(),
                            ColumnSpec::default()
                                .horizontal_alignment(HorizontalAlignment::CenterHorizontally),
                            move || {
                                CentredText(
                                    body.name,
                                    WearTextStyle::TITLE_MEDIUM
                                        .at_size(20.0)
                                        .with_line_height(24.0),
                                    colors.on_surface,
                                );
                                CentredText(
                                    body.kind.label(),
                                    WearTextStyle::LABEL_SMALL,
                                    theme::kind_color(body.kind),
                                );
                            },
                        );
                    });
                    scope.item(move || {
                        CentredText(
                            body.tagline,
                            WearTextStyle::BODY_LARGE.at_size(14.0),
                            colors.on_surface_variant,
                        );
                    });
                    scope.item(move || {
                        let page = page;
                        WearButton(
                            Modifier::empty().fill_max_width(),
                            WearButtonSpec::default().colors(colors),
                            if saved { "Saved" } else { "Save" }.to_owned(),
                            Some(
                                if saved {
                                    "Tap to remove"
                                } else {
                                    "Keep it in your list"
                                }
                                .to_owned(),
                            ),
                            move || page.get().on_toggle_saved(),
                        );
                    });
                    for (label, value) in [
                        ("Distance", body.distance),
                        ("Diameter", body.diameter),
                        ("Day", body.day_length),
                        ("Year", body.year_length),
                        ("Moons", body.moons),
                        ("Gravity", body.gravity_label),
                    ] {
                        scope.item(move || Stat(label, value));
                    }
                    let fact = fact.clone();
                    scope.item(move || FactCard(fact.clone()));
                    // One item per sentence: a short item fits the width a
                    // round screen has above and below its middle, and the
                    // list scales each one as it nears the edge.
                    for sentence in sentences(body.description) {
                        scope.item(move || {
                            CentredText(
                                sentence,
                                WearTextStyle::BODY_LARGE
                                    .at_size(14.0)
                                    .with_line_height(18.0),
                                colors.on_surface,
                            );
                        });
                    }
                    let on_back = on_back.clone();
                    scope.item(move || {
                        let on_back = on_back.clone();
                        WearButton(
                            Modifier::empty().fill_max_width(),
                            WearButtonSpec::default().colors(theme::quiet_button_colors()),
                            "All bodies".to_owned(),
                            None,
                            move || on_back(),
                        );
                    });
                },
            );
        },
    );
}

/// Text centred across the list's width, kept off the curved edges.
#[composable]
fn CentredText(text: &'static str, style: WearTextStyle, color: Color) {
    Text(
        text,
        Modifier::empty()
            .fill_max_width()
            .padding_symmetric(TEXT_SIDE_PADDING, 0.0),
        style.aligned(TextAlign::Center).resolve(color),
    );
}

/// The sentences of `text`.
fn sentences(text: &'static str) -> impl Iterator<Item = &'static str> {
    text.split_inclusive(". ").map(str::trim_end)
}

/// One number about the body, under its name.
#[composable]
fn Stat(label: &'static str, value: &'static str) {
    let colors = theme::colors();
    Column(
        Modifier::empty()
            .fill_max_width()
            .padding_symmetric(0.0, 2.0),
        ColumnSpec::default().horizontal_alignment(HorizontalAlignment::CenterHorizontally),
        move || {
            CentredText(label, WearTextStyle::LABEL_SMALL, colors.on_surface_variant);
            CentredText(value, WearTextStyle::LABEL_MEDIUM, colors.on_surface);
        },
    );
}

/// The fact the view model looked up, or a line saying it is on its way.
#[composable]
fn FactCard(fact: Option<String>) {
    let colors = theme::colors();
    let container = colors.surface_container;
    Column(
        Modifier::empty()
            .fill_max_width()
            .draw_behind(move |scope| {
                scope.draw_round_rect(Brush::Solid(container), CornerRadii::uniform(18.0));
            })
            .padding_symmetric(14.0, 10.0),
        ColumnSpec::default()
            .horizontal_alignment(HorizontalAlignment::CenterHorizontally)
            .vertical_arrangement(LinearArrangement::spaced_by(2.0)),
        move || {
            CentredText("Fact", WearTextStyle::LABEL_SMALL, theme::ACCENT);
            let waiting = fact.is_none();
            Text(
                fact.clone().unwrap_or_else(|| "Looking one up".to_owned()),
                Modifier::empty().fill_max_width(),
                WearTextStyle::BODY_LARGE
                    .at_size(14.0)
                    .aligned(TextAlign::Center)
                    .resolve(if waiting {
                        colors.on_surface_variant
                    } else {
                        colors.on_surface
                    }),
            );
        },
    );
}
