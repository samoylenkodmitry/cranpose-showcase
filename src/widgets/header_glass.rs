#![allow(non_snake_case)]

use cranpose::prelude::*;
use cranpose_ui_graphics::GradientBlurDirection;

const TAPER_HEIGHT: f32 = 88.0;
const TAPER_START_BLUR: f32 = 16.0;

/// The fixed blurred-gradient crown over the starfield.
#[composable]
pub fn HeaderBlurGradient() {
    Box(
        Modifier::empty()
            .fill_max_width()
            .height(TAPER_HEIGHT)
            .backdrop_gradient_blur(
                Dp(TAPER_START_BLUR),
                Dp(0.0),
                GradientBlurDirection::TopToBottom,
            ),
        BoxSpec::default(),
        || {},
    );
}
