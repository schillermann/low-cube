use glam::Vec3;

/// Immutable two-tick position span enabling sub-tick visual interpolation with zero causal drift.
#[derive(Clone, Copy, Debug)]
pub struct BodySpan {
    prior: Vec3,
    current: Vec3,
    blend: f32,
}

impl BodySpan {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(prior: Vec3, current: Vec3, blend: f32) -> Self {
        Self { prior, current, blend }
    }

    /// Secondary constructor anchoring both endpoints at `point` without interpolation offset.
    pub const fn anchored(point: Vec3) -> Self {
        Self::new(point, point, 1.0)
    }

    pub const fn current(self) -> Vec3 {
        self.current
    }

    pub const fn settled(self) -> Self {
        Self::anchored(self.current)
    }

    pub const fn advanced(self, next: Vec3) -> Self {
        Self::new(self.current, next, 1.0)
    }

    pub const fn blended(self, alpha: f32) -> Self {
        Self::new(self.prior, self.current, alpha.clamp(0.0, 1.0))
    }

    pub fn visual(self) -> Vec3 {
        self.prior.lerp(self.current, self.blend)
    }
}
