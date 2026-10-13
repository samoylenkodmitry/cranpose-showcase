//! A body drawn from circles and gradients, and the stars behind the lists.
//!
//! The phone app draws each body with a WGSL shader. The Apple Watch host has
//! no shaders of the app's own, so the watch draws the same colours with plain
//! shapes: a lit sphere, a shadow on its far side, a halo and, for Saturn and
//! Uranus, a ring.

use std::f32::consts::PI;

use cranpose::prelude::*;
use cranpose_ui_graphics::{DrawStyle, Path, TileMode};

use crate::model::{BodyClass, CelestialBody, BODIES};

/// How far the halo reaches past the sphere, as a multiple of its radius.
const HALO_REACH: f32 = 1.45;
/// Segments of half a ring outline.
const RING_SEGMENTS: usize = 32;
/// The tilt of every ring on screen, in radians.
const RING_ROLL: f32 = -0.21;
/// How far each half of a ring reaches past its half turn, in radians.
const RING_OVERLAP: f32 = 0.03;

/// Body `index` drawn into a square of the size `modifier` gives it. `halo`
/// adds the glow of its atmosphere around it.
#[composable]
pub fn PlanetDisc(modifier: Modifier, index: usize, halo: bool) {
    let body = &BODIES[index];
    Box(
        modifier.draw_behind(move |scope| draw_body(scope, body, halo)),
        BoxSpec::default(),
        || {},
    );
}

fn rgb(color: (u8, u8, u8)) -> Color {
    Color::from_rgb_u8(color.0, color.1, color.2)
}

fn draw_body(scope: &mut dyn DrawScope, body: &CelestialBody, halo: bool) {
    let size = scope.size();
    let half = size.width.min(size.height) / 2.0;
    let centre = Point::new(size.width / 2.0, size.height / 2.0);
    let ring_reach = if body.has_ring {
        body.ring_outer_frac
    } else {
        1.0
    };
    let halo_reach = if halo { HALO_REACH } else { 1.0 };
    let radius = half / ring_reach.max(halo_reach);

    if halo {
        let reach = radius * HALO_REACH;
        let glow = rgb(body.glow_color);
        // A gradient's centre is measured from the corner of the shape it
        // fills, here the square around the circle.
        scope.draw_circle(
            Brush::radial_gradient_stops(
                vec![
                    (radius / reach, glow.with_alpha(0.45)),
                    (1.0, glow.with_alpha(0.0)),
                ],
                Point::new(reach, reach),
                reach,
                TileMode::Clamp,
            ),
            centre,
            reach,
        );
    }
    if body.has_ring {
        draw_ring_half(scope, body, centre, radius, PI);
    }

    // The sphere, lit from the upper left. A star shines by itself, so it has
    // no dark side.
    let light = Point::new(radius * 0.62, radius * 0.55);
    scope.draw_circle(
        Brush::radial_gradient(
            vec![rgb(body.top_color), rgb(body.bottom_color)],
            light,
            radius * 1.55,
        ),
        centre,
        radius,
    );
    if body.body_class != BodyClass::Star {
        scope.draw_circle(
            Brush::radial_gradient_stops(
                vec![
                    (0.55, Color::from_rgba_u8(0, 0, 0, 0)),
                    (1.0, Color::from_rgba_u8(0, 0, 0, 190)),
                ],
                light,
                radius * 1.6,
                TileMode::Clamp,
            ),
            centre,
            radius,
        );
    }

    if body.has_ring {
        draw_ring_half(scope, body, centre, radius, 0.0);
    }
}

/// Half of a ring: the far half (`from` = PI) goes behind the sphere, the near
/// half (`from` = 0) in front of it.
fn draw_ring_half(
    scope: &mut dyn DrawScope,
    body: &CelestialBody,
    centre: Point,
    radius: f32,
    from: f32,
) {
    // The ring is a flat disc seen at an angle: its height on screen is its
    // width times the sine of the tilt.
    let squash = body.ring_tilt_deg.to_radians().sin().max(0.2);
    let (roll_sin, roll_cos) = RING_ROLL.sin_cos();
    let point = |reach: f32, angle: f32| {
        let x = reach * radius * angle.cos();
        let y = reach * radius * squash * angle.sin();
        Point::new(
            centre.x + x * roll_cos - y * roll_sin,
            centre.y + x * roll_sin + y * roll_cos,
        )
    };
    // The halves overlap a little, so no seam shows where they meet.
    let start = from - RING_OVERLAP;
    let step = (PI + 2.0 * RING_OVERLAP) / RING_SEGMENTS as f32;
    let mut path = Path::new();
    path.move_to(point(body.ring_outer_frac, start));
    for segment in 1..=RING_SEGMENTS {
        path.line_to(point(body.ring_outer_frac, start + step * segment as f32));
    }
    for segment in (0..=RING_SEGMENTS).rev() {
        path.line_to(point(body.ring_inner_frac, start + step * segment as f32));
    }
    path.close();
    scope.draw_path(&path, Brush::Solid(rgb(body.ring_color)), DrawStyle::Fill);
}

/// Faint stars behind a screen. They stay still while the list moves over
/// them.
#[composable]
pub fn StarBackdrop(modifier: Modifier) {
    Box(modifier.draw_behind(draw_stars), BoxSpec::default(), || {});
}

fn draw_stars(scope: &mut dyn DrawScope) {
    let size = scope.size();
    // A fixed sequence, so every frame and every launch draws the same sky.
    let mut seed: u32 = 0x9E37_79B9;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed % 10_000) as f32 / 10_000.0
    };
    for _ in 0..48 {
        let x = next() * size.width;
        let y = next() * size.height;
        let brightness = 0.18 + next() * 0.5;
        let star_radius = 0.6 + next() * 0.9;
        scope.draw_circle(
            Brush::Solid(Color::rgba(0.85, 0.86, 1.0, brightness)),
            Point::new(x, y),
            star_radius,
        );
    }
    // A wash of the accent colour from the top, as the phone app's sky has.
    let reach = size.width * 0.75;
    scope.draw_circle(
        Brush::radial_gradient(
            vec![
                Color::from_rgba_u8(110, 90, 220, 70),
                Color::from_rgba_u8(110, 90, 220, 0),
            ],
            Point::new(reach, reach),
            reach,
        ),
        Point::new(size.width / 2.0, -size.height * 0.15),
        reach,
    );
}
