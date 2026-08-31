#![allow(non_snake_case)]

use std::f32::consts::TAU;
use std::sync::OnceLock;

use cranpose::prelude::*;
use cranpose_ui_graphics::TileMode;

const STAR_COUNT: usize = 160;
const TWINKLE_STEPS_PER_CYCLE: f32 = 20.0;
const PARALLAX_PITCH_PER_SCREEN: f32 = 0.11;
const PARALLAX_YAW_PER_SCREEN: f32 = 0.16;
const FAVORITES_FOR_FULL_STELLAR_RESPONSE: f32 = 6.0;

struct Star {
    x: f32,
    y: f32,
    z: f32,
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
                z: next_unit() * 2.0 - 1.0,
                radius: 0.5 + next_unit() * 1.5,
                base_alpha: 0.20 + next_unit() * 0.55,
                phase: (i as f32) * 0.618_034,
            })
            .collect()
    })
}

/// Collapses a continuously animating `0..1` value to a fixed number of
/// steps per cycle. Real atmospheric-scintillation twinkle is a discrete
/// flicker, not a silky 60fps interpolation, so stepping is truthful to the
/// phenomenon being drawn, not merely convenient: it also collapses the
/// backdrop's recorded primitives to a small, fixed set of distinct frames
/// per cycle instead of one distinct frame per render, so the renderer's
/// scene diff finds no change on most frames and skips re-encoding them.
fn quantize(value: f32, steps_per_cycle: f32) -> f32 {
    (value * steps_per_cycle).floor() / steps_per_cycle
}

fn parallax_rotation(scroll_offset: f32) -> (f32, f32) {
    let screens = scroll_offset / 720.0;
    (
        screens * PARALLAX_PITCH_PER_SCREEN,
        screens * PARALLAX_YAW_PER_SCREEN,
    )
}

fn favorite_stellar_response(favorite_count: usize) -> f32 {
    (favorite_count as f32 / FAVORITES_FOR_FULL_STELLAR_RESPONSE).clamp(0.0, 1.0)
}

fn star_alpha(base_alpha: f32, wave: f32, favorite_count: usize) -> f32 {
    let response = favorite_stellar_response(favorite_count);
    (base_alpha + wave * (0.22 + response * 0.12) + response * 0.24).clamp(0.04, 1.0)
}

/// A deep-space backdrop projected from a 3D field. The field makes one slow,
/// clockwise orbit while foreground scroll applies its pitch and yaw, so stars
/// move with perspective instead of translating as a flat texture.
#[composable]
pub fn Starfield(
    modifier: Modifier,
    scroll_offset: f32,
    drift: f32,
    twinkle: f32,
    favorite_count: usize,
) {
    let twinkle = quantize(twinkle, TWINKLE_STEPS_PER_CYCLE);
    let (pitch, yaw) = parallax_rotation(scroll_offset);
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

            let clockwise = drift * TAU * 0.025;
            let (sin_yaw, cos_yaw) = (yaw + clockwise).sin_cos();
            let (sin_pitch, cos_pitch) = pitch.sin_cos();
            for star in stars() {
                let x0 = star.x * 2.0 - 1.0;
                let y0 = star.y * 2.0 - 1.0;
                let z0 = star.z;
                let x1 = x0 * cos_yaw + z0 * sin_yaw;
                let z1 = z0 * cos_yaw - x0 * sin_yaw;
                let y1 = y0 * cos_pitch - z1 * sin_pitch;
                let depth = (z1 * sin_pitch + y0 * cos_pitch + 2.8).max(0.8);
                let perspective = 1.0 / depth;
                let x = size.width * (0.5 + x1 * perspective * 0.72);
                let y = size.height * (0.5 + y1 * perspective * 0.72);
                if !(0.0..=size.width).contains(&x) || !(0.0..=size.height).contains(&y) {
                    continue;
                }
                let wave = ((twinkle + star.phase) * TAU).sin();
                let alpha = star_alpha(star.base_alpha, wave, favorite_count);
                scope.draw_circle(
                    Brush::solid(Color::from_rgba_u8(255, 255, 255, (alpha * 255.0) as u8)),
                    Point { x, y },
                    star.radius * perspective * 2.2,
                );
            }
        }),
        BoxSpec::default(),
        || {},
    );
}

#[cfg(test)]
mod tests {
    use super::{parallax_rotation, star_alpha};

    #[test]
    fn parallax_moves_in_response_to_scrolling() {
        assert_eq!(parallax_rotation(0.0), (0.0, 0.0));
        assert_ne!(parallax_rotation(96.0), (0.0, 0.0));
        assert!(parallax_rotation(720.0).0 > parallax_rotation(96.0).0);
        assert!(parallax_rotation(720.0).1 > parallax_rotation(96.0).1);
    }

    #[test]
    fn favorites_brighten_the_starfield() {
        assert!(star_alpha(0.45, 0.0, 1) > star_alpha(0.45, 0.0, 0));
        assert!(star_alpha(0.45, 0.0, 6) > star_alpha(0.45, 0.0, 1));
    }
}
