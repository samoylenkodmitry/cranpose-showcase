use cranpose::liquid::prelude::*;

/// The shared unblurred liquid material used by every Showcase surface.
pub fn showcase_glass(colors: LiquidColors, radius: f32) -> Glass {
    Glass::regular()
        .shape(LiquidShape::RoundedRect(radius))
        .blur_radius(0.0)
        .refraction_depth(0.58)
        .refraction_curve(0.62)
        .dispersion(1.0)
        .transmission_refraction(0.72)
        .highlight(0.72)
        .adaptive_frost(colors.label, 0.42)
}
