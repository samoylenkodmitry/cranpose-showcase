#![allow(non_snake_case)]

use cranpose::prelude::*;
use cranpose_animation::prelude::*;

pub(crate) const PLANET_CYCLE_MS: u64 = 48_000;
pub(crate) const STAR_ORBIT_CYCLE_MS: u64 = 240_000;
pub(crate) const SHEEN_CYCLE_MS: u64 = 5_200;
pub(crate) const STAR_TWINKLE_CYCLE_MS: u64 = 3_600;
pub(crate) const SHEEN_STEPS: u32 = 104;
pub(crate) const STAR_ORBIT_STEPS: u32 = 2_400;
pub(crate) const STAR_TWINKLE_STEPS: u32 = 20;
pub(crate) const THUMBNAIL_DRIFT_STEPS: u32 = 960;

const _: () = assert!(STAR_ORBIT_CYCLE_MS >= PLANET_CYCLE_MS * 4);

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct AmbientMotion {
    sheen: State<f32>,
    planet_rotation: State<f32>,
    star_orbit: State<f32>,
    twinkle: State<f32>,
}

impl AmbientMotion {
    pub(crate) fn sheen(self) -> f32 {
        self.sheen.value()
    }

    pub(crate) fn planet_rotation(self) -> f32 {
        self.planet_rotation.value()
    }

    pub(crate) fn star_orbit(self) -> f32 {
        self.star_orbit.value()
    }

    pub(crate) fn twinkle(self) -> f32 {
        self.twinkle.value()
    }
}

fn repeating(spec: AnimationSpec) -> InfiniteRepeatableSpec<f32> {
    infiniteRepeatable(spec, RepeatMode::Restart, StartOffset::default())
}

#[composable]
pub(crate) fn rememberAmbientMotion(showing_detail: bool) -> AmbientMotion {
    let infinite = rememberInfiniteTransition("showcase-ambient");
    let sheen = infinite.animateFloat(
        0.0,
        1.0,
        infiniteRepeatable(
            AnimationSpec::stepped(SHEEN_CYCLE_MS, SHEEN_STEPS),
            RepeatMode::Reverse,
            StartOffset::default(),
        ),
        "sheen",
    );
    let planet_spec = if showing_detail {
        AnimationSpec::linear(PLANET_CYCLE_MS)
    } else {
        AnimationSpec::stepped(PLANET_CYCLE_MS, THUMBNAIL_DRIFT_STEPS)
    };
    let planet_rotation =
        infinite.animateFloat(0.0, 1.0, repeating(planet_spec), "planet-rotation");
    let star_orbit = infinite.animateFloat(
        0.0,
        1.0,
        repeating(AnimationSpec::stepped(
            STAR_ORBIT_CYCLE_MS,
            STAR_ORBIT_STEPS,
        )),
        "star-orbit",
    );
    let twinkle = infinite.animateFloat(
        0.0,
        1.0,
        repeating(AnimationSpec::stepped(
            STAR_TWINKLE_CYCLE_MS,
            STAR_TWINKLE_STEPS,
        )),
        "twinkle",
    );
    AmbientMotion {
        sheen,
        planet_rotation,
        star_orbit,
        twinkle,
    }
}
