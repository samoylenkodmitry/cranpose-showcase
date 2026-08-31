#![allow(non_snake_case)]

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;
use cranpose_ui_graphics::GradientBlurDirection;

const TAPER_HEIGHT: f32 = 88.0;
const TAPER_START_BLUR: f32 = 18.0;
const TAPER_END_BLUR: f32 = 0.35;

/// The fixed blurred-gradient crown over the starfield.
#[composable]
pub fn HeaderBlurGradient() {
    let colors = liquid_colors();
    let top = colors.background.with_alpha(0.82);
    let middle = colors.background.with_alpha(0.40);
    Box(
        Modifier::empty()
            .fill_max_width()
            .height(TAPER_HEIGHT)
            .alignInBox(Alignment::new(
                HorizontalAlignment::CenterHorizontally,
                VerticalAlignment::Top,
            ))
            .backdrop_gradient_blur(
                Dp(TAPER_START_BLUR),
                Dp(TAPER_END_BLUR),
                GradientBlurDirection::TopToBottom,
            )
            .draw_behind(move |scope| {
                scope.draw_rect(Brush::vertical_gradient(
                    vec![top, middle, top.with_alpha(0.0)],
                    0.0,
                    scope.size().height,
                ));
            }),
        BoxSpec::default(),
        || {},
    );
}
