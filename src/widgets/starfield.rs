#![allow(non_snake_case)]

use std::f32::consts::TAU;
use std::rc::Rc;

use cranpose::prelude::*;
use cranpose_foundation::lazy::LazyListState;
use cranpose_ui_graphics::TileMode;

use crate::motion::{AmbientMotion, STAR_TWINKLE_STEPS};

const STAR_COUNT: usize = 160;
const STARFIELD_SPAN: f32 = 5.0;
const STARFIELD_DEPTH: f32 = 5.0;
const CAMERA_DISTANCE: f32 = 4.2;
const PROJECTION_SCALE: f32 = 0.92;
const PARALLAX_PITCH_PER_SCREEN: f32 = 0.11;
const PARALLAX_YAW_PER_SCREEN: f32 = 0.16;
const PARALLAX_SHIFT_X_PER_SCREEN: f32 = 0.06;
const PARALLAX_SHIFT_Y_PER_SCREEN: f32 = 0.18;
const MAX_PARALLAX_SCREENS: f32 = 2.5;
const FAVORITES_FOR_FULL_STELLAR_RESPONSE: f32 = 6.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ParallaxCamera {
    pitch: f32,
    yaw: f32,
    shift_x: f32,
    shift_y: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StarfieldScroll {
    LazyList(LazyListState),
    Scroll(ScrollState),
}

impl StarfieldScroll {
    fn offset(self) -> f32 {
        match self {
            Self::LazyList(state) => {
                state.first_visible_item_index() as f32 * 112.0
                    + state.first_visible_item_scroll_offset()
            }
            Self::Scroll(state) => state.value(),
        }
    }
}

struct Star {
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
    base_alpha: f32,
    wave_by_step: [f32; STAR_TWINKLE_STEPS as usize],
}

/// A small deterministic generator (xorshift32) so the field looks random
/// without pulling in a `rand` dependency for decoration.
fn build_stars() -> Rc<[Star]> {
    let mut seed: u32 = 0x9E37_79B9;
    let mut next_unit = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed as f64 / u32::MAX as f64) as f32
    };
    (0..STAR_COUNT)
        .map(|index| {
            let phase = index as f32 * 0.618_034;
            Star {
                x: (next_unit() - 0.5) * STARFIELD_SPAN,
                y: (next_unit() - 0.5) * STARFIELD_SPAN,
                z: (next_unit() - 0.5) * STARFIELD_DEPTH,
                radius: 0.5 + next_unit() * 1.5,
                base_alpha: 0.20 + next_unit() * 0.55,
                wave_by_step: std::array::from_fn(|step| {
                    let twinkle = step as f32 / STAR_TWINKLE_STEPS as f32;
                    ((twinkle + phase) * TAU).sin()
                }),
            }
        })
        .collect()
}

fn parallax_camera(scroll_offset: f32) -> ParallaxCamera {
    let screens = (scroll_offset / 720.0).clamp(0.0, MAX_PARALLAX_SCREENS);
    ParallaxCamera {
        pitch: screens * PARALLAX_PITCH_PER_SCREEN,
        yaw: screens * PARALLAX_YAW_PER_SCREEN,
        shift_x: screens * PARALLAX_SHIFT_X_PER_SCREEN,
        shift_y: screens * PARALLAX_SHIFT_Y_PER_SCREEN,
    }
}

fn orbital_yaw(drift: f32) -> f32 {
    -drift * TAU
}

fn rotate_around_vertical_axis(x: f32, z: f32, sin_yaw: f32, cos_yaw: f32) -> (f32, f32) {
    (x * cos_yaw + z * sin_yaw, z * cos_yaw - x * sin_yaw)
}

fn depth_scale(perspective: f32) -> f32 {
    (4.0 * perspective).clamp(0.5, 2.6)
}

fn depth_brightness(depth: f32) -> f32 {
    (1.25 - depth * 0.11).clamp(0.55, 1.0)
}

fn projected_axis(world: f32, camera_shift: f32, perspective: f32) -> f32 {
    (world - camera_shift) * perspective * PROJECTION_SCALE
}

fn favorite_stellar_response(favorite_count: usize) -> f32 {
    (favorite_count as f32 / FAVORITES_FOR_FULL_STELLAR_RESPONSE).clamp(0.0, 1.0)
}

fn star_alpha(base_alpha: f32, wave: f32, response: f32) -> f32 {
    (base_alpha + wave * (0.22 + response * 0.12) + response * 0.24).clamp(0.04, 1.0)
}

/// A full-screen deep-space backdrop projected from a rotating 3D field. The
/// field orbits counter-clockwise around its vertical axis while scroll shifts
/// and tilts the camera.
#[composable]
pub fn Starfield(
    modifier: Modifier,
    scroll: StarfieldScroll,
    ambient: AmbientMotion,
    favorite_count: usize,
) {
    let stars = remember(build_stars).with(Rc::clone);
    let stellar_response = favorite_stellar_response(favorite_count);
    Box(
        modifier.draw_behind(move |scope| {
            let camera = parallax_camera(scroll.offset());
            let twinkle_step = (ambient.twinkle() * STAR_TWINKLE_STEPS as f32) as usize
                % STAR_TWINKLE_STEPS as usize;
            let orbit = ambient.star_orbit();
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

            let (sin_yaw, cos_yaw) = (camera.yaw + orbital_yaw(orbit)).sin_cos();
            let (sin_pitch, cos_pitch) = camera.pitch.sin_cos();
            let center_x = size.width * 0.5;
            let center_y = size.height * 0.5;
            for star in stars.iter() {
                let (x1, z1) = rotate_around_vertical_axis(star.x, star.z, sin_yaw, cos_yaw);
                let y1 = star.y * cos_pitch - z1 * sin_pitch;
                let z2 = z1 * cos_pitch + star.y * sin_pitch;
                let depth = (z2 + CAMERA_DISTANCE).max(0.9);
                let perspective = 1.0 / depth;
                let x = center_x + size.width * projected_axis(x1, camera.shift_x, perspective);
                let y = center_y + size.height * projected_axis(y1, camera.shift_y, perspective);
                if !(0.0..=size.width).contains(&x) || !(0.0..=size.height).contains(&y) {
                    continue;
                }
                let wave = star.wave_by_step[twinkle_step];
                let alpha =
                    star_alpha(star.base_alpha, wave, stellar_response) * depth_brightness(depth);
                scope.draw_circle(
                    Brush::solid(Color::from_rgba_u8(255, 255, 255, (alpha * 255.0) as u8)),
                    Point { x, y },
                    star.radius * depth_scale(perspective),
                );
            }
        }),
        BoxSpec::default(),
        || {},
    );
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::{
        build_stars, depth_scale, favorite_stellar_response, orbital_yaw, parallax_camera,
        projected_axis, rotate_around_vertical_axis, star_alpha, CAMERA_DISTANCE,
    };

    #[test]
    fn parallax_moves_in_response_to_scrolling() {
        assert_eq!(parallax_camera(0.0), Default::default());
        assert_ne!(parallax_camera(96.0), Default::default());
        assert!(parallax_camera(720.0).shift_x > parallax_camera(96.0).shift_x);
        assert!(parallax_camera(720.0).shift_y > parallax_camera(96.0).shift_y);
    }

    #[test]
    fn ambient_orbit_is_counter_clockwise_around_the_vertical_axis() {
        assert!(orbital_yaw(0.25) < 0.0);
        assert!(orbital_yaw(1.0).sin().abs() < f32::EPSILON * 2.0);
    }

    #[test]
    fn vertical_axis_orbit_moves_horizontal_position_into_depth() {
        let (sin_yaw, cos_yaw) = (-TAU * 0.25).sin_cos();
        let (x, z) = rotate_around_vertical_axis(1.0, 0.0, sin_yaw, cos_yaw);
        assert!(x.abs() < f32::EPSILON * 2.0);
        assert!(z > 0.99);
    }

    #[test]
    fn rotating_volume_keeps_stars_at_every_screen_edge() {
        let stars = build_stars();
        for quarter_turn in 0..4 {
            let (sin_yaw, cos_yaw) = (-(quarter_turn as f32) * TAU * 0.25).sin_cos();
            let mut covered = [false; 4];
            for star in stars.iter() {
                let (x, z) = rotate_around_vertical_axis(star.x, star.z, sin_yaw, cos_yaw);
                let perspective = 1.0 / (z + CAMERA_DISTANCE).max(0.9);
                let projected_x = projected_axis(x, 0.0, perspective);
                let projected_y = projected_axis(star.y, 0.0, perspective);
                if projected_x.abs() > 0.5 || projected_y.abs() > 0.5 {
                    continue;
                }
                covered[0] |= projected_x < -0.42;
                covered[1] |= projected_x > 0.42;
                covered[2] |= projected_y < -0.42;
                covered[3] |= projected_y > 0.42;
            }
            assert!(covered.into_iter().all(|edge| edge));
        }
    }

    #[test]
    fn nearby_stars_are_visibly_larger_than_distant_stars() {
        assert!(depth_scale(1.0 / 1.8) > depth_scale(1.0 / 4.2));
    }

    #[test]
    fn camera_translation_has_stronger_parallax_on_nearby_stars() {
        let near = projected_axis(0.0, 0.2, 1.0 / 1.8).abs();
        let far = projected_axis(0.0, 0.2, 1.0 / 4.2).abs();
        assert!(near > far);
    }

    #[test]
    fn favorites_brighten_the_starfield() {
        let none = favorite_stellar_response(0);
        let one = favorite_stellar_response(1);
        let six = favorite_stellar_response(6);
        assert!(star_alpha(0.45, 0.0, one) > star_alpha(0.45, 0.0, none));
        assert!(star_alpha(0.45, 0.0, six) > star_alpha(0.45, 0.0, one));
    }
}
