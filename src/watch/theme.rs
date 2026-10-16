//! Colours and spacing of the watch app.

use cranpose::prelude::*;
use cranpose_ui::widgets::wear::{
    ScreenScaffoldSpec, ScrollIndicatorSpec, WearColors, WearScalingLazyColumnSpec,
};

use crate::model::BodyKind;

/// A watch shows its apps on black, which also saves power on an OLED screen.
pub const BACKGROUND: Color = Color::BLACK;

/// The lavender of the phone app's accent, lighter so it reads on black.
pub const ACCENT: Color = Color::from_rgb_u8(190, 178, 255);

/// Space between the list and the left and right edges of the screen. A row at
/// the centre line of a round screen has the whole width; rows above and below
/// it shrink as the list scales them.
const LIST_SIDE_PADDING: f32 = 10.0;
/// Space above the first row and below the last one.
const LIST_END_PADDING: f32 = 24.0;

/// The roles the Wear widgets read, in the app's colours.
pub fn colors() -> WearColors {
    WearColors {
        primary: ACCENT,
        primary_container: Color::from_rgb_u8(62, 52, 128),
        on_primary: Color::from_rgb_u8(28, 20, 66),
        on_primary_container: Color::from_rgb_u8(230, 226, 255),
        surface_container: Color::from_rgb_u8(30, 29, 42),
        on_surface: Color::from_rgb_u8(238, 236, 246),
        on_surface_variant: Color::from_rgb_u8(178, 174, 198),
        outline: Color::from_rgb_u8(120, 116, 140),
        background: BACKGROUND,
        on_background: Color::from_rgb_u8(238, 236, 246),
        content: Color::WHITE,
        indicator_thumb: Color::WHITE,
        indicator_track: Color::WHITE,
    }
    .with_wear_scroll_indicator()
}

/// The colours of a button that is not the main action of its screen.
pub fn quiet_button_colors() -> WearColors {
    let colors = colors();
    WearColors {
        primary: colors.surface_container,
        on_primary: colors.on_surface,
        ..colors
    }
}

/// The colour that marks each kind of body, as on the phone.
pub fn kind_color(kind: BodyKind) -> Color {
    match kind {
        BodyKind::Star => Color::from_rgb_u8(255, 189, 89),
        BodyKind::Planet => ACCENT,
        BodyKind::DwarfPlanet => Color::from_rgb_u8(196, 186, 232),
        BodyKind::Moon => Color::from_rgb_u8(170, 190, 210),
        BodyKind::Exoplanet => Color::from_rgb_u8(236, 132, 132),
    }
}

/// How every list of the watch app is laid out.
pub fn list_spec() -> WearScalingLazyColumnSpec {
    WearScalingLazyColumnSpec::default().content_padding(PaddingValues::symmetric(
        LIST_SIDE_PADDING,
        LIST_END_PADDING,
    ))
}

/// The screen around a list: its scroll indicator stays on screen, so a still
/// screen shows where the list is.
pub fn scaffold_spec() -> ScreenScaffoldSpec {
    ScreenScaffoldSpec::default()
        .indicator(ScrollIndicatorSpec::default().colors(colors()).alpha(1.0))
}
