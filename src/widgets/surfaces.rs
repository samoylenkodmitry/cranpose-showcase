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

#[cfg(test)]
mod tests {
    use super::showcase_glass;
    use cranpose::liquid::prelude::*;
    use cranpose::prelude::Color;

    #[test]
    fn showcase_surfaces_keep_visible_spectral_refraction() {
        let glass = showcase_glass(LiquidColors::dark(Color::WHITE), 20.0);

        assert!(glass.refraction_depth > 0.0);
        assert!(glass.dispersion > 0.0);
        assert!(glass.transmission_refraction > 0.0);
    }
}
