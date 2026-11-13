use glam::Vec3;
use world::MeshTarget;
use super::glyph::PixelStamp;

/// Bottom-right release version watermark displayed whenever the session menu is open.
#[derive(Clone, Copy, Debug)]
pub struct VersionStamp {
    label: &'static str,
}

impl VersionStamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(label: &'static str) -> Self {
        Self { label }
    }

    /// Secondary constructor stamped with the crate's compile-time Semantic Version.
    pub const fn current() -> Self {
        Self::new(concat!("V", env!("CARGO_PKG_VERSION")))
    }

    /// Emits a crisp shadowed version label anchored in the bottom-right screen corner.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let shadow = Vec3::new(0.05, 0.07, 0.11);
        let ink = Vec3::new(0.84, 0.89, 0.96);
        PixelStamp::caption(shadow).emit_bottom_right(self.label, 0.983, -0.976, target);
        PixelStamp::caption(ink).emit_bottom_right(self.label, 0.980, -0.970, target);
    }
}
