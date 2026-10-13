//! The first screen of the watch app: every body, in a list the crown scrolls.

use std::rc::Rc;

use cranpose::prelude::*;
use cranpose_coroflow::{Handle, StateFlowCollect};
use cranpose_ui::widgets::wear::{
    ListHeader, ListHeaderSpec, ScreenScaffold, WearScalingLazyColumn, WearScalingListState,
    WearTextStyle,
};

use super::planet::PlanetDisc;
use super::theme;
use crate::data::AppServices;
use crate::model::BODIES;

/// The height of a row, Wear's height for a button.
const ROW_HEIGHT: f32 = 52.0;
/// The side of the body drawn at the start of a row.
const ROW_DISC: f32 = 34.0;

/// The list of bodies, scrolled to where `list` says. `on_open` gets the
/// index of the body tapped.
#[composable]
pub fn BodyList(
    services: Handle<AppServices>,
    list: WearScalingListState,
    on_open: impl Fn(usize) + 'static,
) {
    let on_open: Rc<dyn Fn(usize)> = Rc::new(on_open);
    let saved = services.get().favorites.saved().collectAsState().get();
    ScreenScaffold(
        Modifier::empty().fill_max_size(),
        list,
        theme::scaffold_spec(),
        move || {
            let saved = saved.clone();
            let on_open = on_open.clone();
            WearScalingLazyColumn(
                Modifier::empty().fill_max_size(),
                list,
                theme::list_spec(),
                move |scope| {
                    scope.item(|| {
                        ListHeader(
                            Modifier::empty().fill_max_width(),
                            ListHeaderSpec::default().colors(theme::colors()),
                            "Star chart",
                        );
                    });
                    scope.items(BODIES.len(), move |index| {
                        let on_open = on_open.clone();
                        BodyRow(
                            index,
                            saved.get(index).copied().unwrap_or(false),
                            move || on_open(index),
                        );
                    });
                },
            );
        },
    );
}

/// One body: a capsule with the body, its name and its kind. A saved body has
/// a dot at the end.
#[composable]
fn BodyRow(index: usize, saved: bool, on_open: impl Fn() + 'static) {
    let body = &BODIES[index];
    let colors = theme::colors();
    let container = colors.surface_container;
    Row(
        Modifier::empty()
            .fill_max_width()
            .height_in(ROW_HEIGHT, f32::INFINITY)
            .draw_behind(move |scope| {
                let height = scope.size().height;
                scope.draw_round_rect(Brush::Solid(container), CornerRadii::uniform(height / 2.0));
            })
            .clickable(move |_point| on_open())
            .padding_each(9.0, 6.0, 16.0, 6.0),
        RowSpec::default()
            .vertical_alignment(VerticalAlignment::CenterVertically)
            .horizontal_arrangement(LinearArrangement::spaced_by(10.0)),
        move || {
            PlanetDisc(
                Modifier::empty().size(Size {
                    width: ROW_DISC,
                    height: ROW_DISC,
                }),
                index,
                false,
            );
            Column(
                Modifier::empty().weight(1.0),
                ColumnSpec::default(),
                move || {
                    TextWithOptions(
                        body.name,
                        Modifier::empty(),
                        WearTextStyle::LABEL_MEDIUM.resolve(colors.on_surface),
                        TextOptions {
                            overflow: TextOverflow::Ellipsis,
                            max_lines: Some(1),
                            ..TextOptions::default()
                        },
                    );
                    Text(
                        body.kind.label(),
                        Modifier::empty(),
                        WearTextStyle::LABEL_SMALL.resolve(theme::kind_color(body.kind)),
                    );
                },
            );
            if saved {
                Box(
                    Modifier::empty()
                        .size(Size {
                            width: 8.0,
                            height: 8.0,
                        })
                        .draw_behind(|scope| {
                            let size = scope.size();
                            scope.draw_circle(
                                Brush::Solid(theme::ACCENT),
                                Point::new(size.width / 2.0, size.height / 2.0),
                                size.width / 2.0,
                            );
                        }),
                    BoxSpec::default(),
                    || {},
                );
            }
        },
    );
}
