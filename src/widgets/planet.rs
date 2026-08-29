#![allow(non_snake_case)]

use std::f32::consts::PI;

use cranpose::prelude::*;
use cranpose_ui_graphics::{Stroke, StrokeCap, TileMode};

use crate::model::CelestialBody;

fn color(c: (u8, u8, u8)) -> Color {
    Color::from_rgb_u8(c.0, c.1, c.2)
}

fn color_alpha(c: (u8, u8, u8), alpha: u8) -> Color {
    Color::from_rgba_u8(c.0, c.1, c.2, alpha)
}

#[composable]
fn Ring(body: &'static CelestialBody, tilt: f32, front: bool) {
    Box(
        Modifier::empty()
            .fill_max_size()
            .graphics_layer_block(move |layer| {
                layer.scale_y = 0.36;
                layer.rotation_z = tilt;
            })
            .draw_behind(move |scope| {
                let size = scope.size();
                let center = Point {
                    x: size.width * 0.5,
                    y: size.height * 0.5,
                };
                let outer = size.width * 0.62;
                let inner = size.width * 0.5;
                let band = color_alpha(body.glow_color, 190);
                let edge = color_alpha(body.glow_color, 90);
                if front {
                    scope.draw_arc(
                        Brush::solid(band),
                        center,
                        (outer + inner) * 0.5,
                        PI * 0.06,
                        PI * 0.88,
                        Stroke::new(outer - inner).with_cap(StrokeCap::Round),
                    );
                } else {
                    scope.draw_circle_stroked(
                        Brush::solid(edge),
                        center,
                        outer,
                        Stroke::new((outer - inner) * 0.4),
                    );
                    scope.draw_circle_stroked(
                        Brush::solid(band),
                        center,
                        (outer + inner) * 0.5,
                        Stroke::new(outer - inner),
                    );
                }
            }),
        BoxSpec::default(),
        || {},
    );
}

#[composable]
fn Sphere(body: &'static CelestialBody, drift: f32) {
    Box(
        Modifier::empty().fill_max_size().draw_behind(move |scope| {
            let size = scope.size();
            let r = size.width.min(size.height) * 0.5;
            let center = Point {
                x: size.width * 0.5,
                y: size.height * 0.5,
            };

            scope.draw_circle(
                Brush::radial_gradient_stops(
                    vec![
                        (0.0, color_alpha(body.glow_color, 70)),
                        (1.0, color_alpha(body.glow_color, 0)),
                    ],
                    center,
                    r * 1.5,
                    TileMode::Clamp,
                ),
                center,
                r * 1.5,
            );

            let light_center = Point {
                x: center.x - r * (0.32 + drift),
                y: center.y - r * 0.34,
            };
            scope.draw_circle(
                Brush::radial_gradient_stops(
                    vec![
                        (0.0, color(body.top_color)),
                        (0.6, color(body.top_color)),
                        (1.0, color(body.bottom_color)),
                    ],
                    light_center,
                    r * 1.7,
                    TileMode::Clamp,
                ),
                center,
                r,
            );

            let highlight_center = Point {
                x: center.x - r * (0.38 + drift),
                y: center.y - r * 0.4,
            };
            scope.draw_circle(
                Brush::radial_gradient_stops(
                    vec![
                        (0.0, Color::from_rgba_u8(255, 255, 255, 130)),
                        (1.0, Color::from_rgba_u8(255, 255, 255, 0)),
                    ],
                    highlight_center,
                    r * 0.42,
                    TileMode::Clamp,
                ),
                highlight_center,
                r * 0.42,
            );
        }),
        BoxSpec::default(),
        || {},
    );
}

/// A procedurally lit sphere for one celestial body: a radial gradient body
/// shaded as if lit from the upper left, a soft atmosphere glow, a specular
/// highlight, and — for ringed worlds — a squashed elliptical ring split
/// behind/in front of the sphere. `sheen` (0..1, looping) drifts the
/// highlight slightly so a resting card still reads as alive.
#[composable]
pub fn PlanetSphere(modifier: Modifier, body: &'static CelestialBody, sheen: f32) {
    let drift = (sheen - 0.5) * 0.18;
    Box(
        modifier,
        BoxSpec::default().content_alignment(Alignment::CENTER),
        move || {
            if body.has_ring {
                Ring(body, -14.0, false);
            }
            Sphere(body, drift);
            if body.has_ring {
                Ring(body, -14.0, true);
            }
        },
    );
}
