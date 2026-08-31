use cranpose::liquid::prelude::*;

/// The shared unblurred liquid material used by every Showcase surface.
pub fn showcase_glass(colors: LiquidColors, radius: f32) -> Glass {
    Glass::regular()
        .shape(LiquidShape::RoundedRect(radius))
        .blur_radius(0.0)
        .refraction_depth(0.22)
        .transmission_refraction(0.18)
        .highlight(0.34)
        .adaptive_frost(colors.label, 0.42)
}
