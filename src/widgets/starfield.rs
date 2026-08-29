#![allow(non_snake_case)]

use std::f32::consts::TAU;
use std::sync::OnceLock;

use cranpose::prelude::*;
use cranpose_ui_graphics::TileMode;

const STAR_COUNT: usize = 160;

struct Star {
    x: f32,
    y: f32,
    radius: f32,
    base_alpha: f32,
    phase: f32,
}

/// A small deterministic generator (xorshift32) so the field looks random
/// without pulling in a `rand` dependency for decoration.
fn stars() -> &'static [Star] {
    static STARS: OnceLock<Vec<Star>> = OnceLock::new();
    STARS.get_or_init(|| {
        let mut seed: u32 = 0x9E37_79B9;
        let mut next_unit = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed as f64 / u32::MAX as f64) as f32
        };
        (0..STAR_COUNT)
            .map(|i| Star {
                x: next_unit(),
                y: next_unit(),
                radius: 0.5 + next_unit() * 1.5,
                base_alpha: 0.20 + next_unit() * 0.55,
                phase: (i as f32) * 0.618_034,
            })
            .collect()
    })
}

/// A deep-space backdrop: a soft vignette wash plus a field of twinkling
/// stars that drift slightly with `parallax` (driven by the list's scroll
/// offset) so the background reads as sitting behind the content rather than
/// printed on it.
#[composable]
pub fn Starfield(modifier: Modifier, parallax: f32, twinkle: f32) {
    Box(
        modifier.draw_behind(move |scope| {
            let size = scope.size();
            let vignette_center = Point {
                x: size.width * 0.5,
                y: size.height * 0.1,
            };
            scope.draw_rect(Brush::radial_gradient_stops(
                vec![
                    (0.0, Color::from_rgb_u8(24, 20, 46)),
                    (0.55, Color::from_rgb_u8(11, 10, 26)),
                    (1.0, Color::from_rgb_u8(4, 4, 10)),
                ],
                vignette_center,
                size.width.max(size.height) * 0.95,
                TileMode::Clamp,
            ));

            for star in stars() {
                let drift = (parallax * 0.06) % 1.0;
                let y = ((star.y + drift).rem_euclid(1.0)) * size.height;
                let x = star.x * size.width;
                let wave = ((twinkle + star.phase) * TAU).sin();
                let alpha = (star.base_alpha + wave * 0.22).clamp(0.04, 1.0);
                scope.draw_circle(
                    Brush::solid(Color::from_rgba_u8(255, 255, 255, (alpha * 255.0) as u8)),
                    Point { x, y },
                    star.radius,
                );
            }
        }),
        BoxSpec::default(),
        || {},
    );
}
