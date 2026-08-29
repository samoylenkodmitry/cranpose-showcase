#![allow(non_snake_case)]

use cranpose::prelude::*;
use cranpose_ui_graphics::GradientBlurDirection;

const TAPER_HEIGHT: f32 = 34.0;
const TAPER_START_BLUR: f32 = 16.0;

/// A short spatial-blur taper starting at the inline bar's bottom edge
/// (`collapse_range`, the same value driving the paired
/// [`LiquidNavBar`]'s collapse) and fading to zero further down.
///
/// `LiquidNavBar`'s glass band blurs at one fixed radius and clips there, so
/// once the large title has faded past it, scrolling content meets that
/// radius head-on at a hard seam. Compositing this taper right below the
/// band continues the falloff instead of cutting it, so the transition reads
/// as a ramp. Place it as a sibling immediately before `LiquidNavBar` in the
/// same `Box`, so the bar's own title/band still draw on top of it.
#[composable]
pub fn HeaderBlurRamp(collapse_range: f32) {
    Box(
        Modifier::empty()
            .fill_max_width()
            .offset(0.0, collapse_range)
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
