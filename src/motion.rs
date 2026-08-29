/// The shared ambient animation values driven by one infinite transition in
/// [`crate::app::RootShell`] and threaded through every screen: the starfield
/// drift and twinkle, and the slow highlight sheen every planet sphere reads.
/// Bundled together so a screen that needs "the ambient motion" takes one
/// argument instead of three unrelated floats.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AmbientMotion {
    pub sheen: f32,
    pub drift: f32,
    pub twinkle: f32,
}
