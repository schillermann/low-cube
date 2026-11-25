use glam::Vec3;

/// Camera sight cone culling distant or rear-hemisphere wildlife meshes with elastic horizon scaling.
#[derive(Clone, Copy, Debug)]
pub struct SightCone {
    eye: Vec3,
    forward: Vec3,
    aperture: f32,
    horizon_scale: f32,
}

impl SightCone {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eye: Vec3, forward: Vec3, aperture: f32, horizon_scale: f32) -> Self {
        Self { eye, forward, aperture, horizon_scale }
    }

    /// Secondary constructor culling beyond nominal range and behind the active camera gaze.
    pub fn from_view(eye: Vec3, forward: Vec3) -> Self {
        Self::new(eye, forward.normalize_or_zero(), -0.28, 1.0)
    }

    /// Secondary constructor accepting all active wildlife within planetary range.
    pub const fn omnidirectional(eye: Vec3) -> Self {
        Self::new(eye, Vec3::NEG_Z, -1.01, 1.0)
    }

    /// Returns a cone whose far-horizon reach dilates or contracts with hardware frame headroom.
    pub const fn scaled(self, factor: f32) -> Self {
        Self::new(self.eye, self.forward, self.aperture, (self.horizon_scale * factor).clamp(0.65, 1.45))
    }

    /// Evaluates whether an animal at `spot` lies within elastic horizon range and aperture cosine.
    pub fn reaches(&self, spot: Vec3, max_dist: f32) -> bool {
        let reach = max_dist * self.horizon_scale;
        let delta = spot - self.eye;
        let dist_sq = delta.length_squared();
        if dist_sq > reach * reach {
            return false;
        }
        if dist_sq <= 4.2 * 4.2 {
            return true;
        }
        delta.dot(self.forward) >= self.aperture * dist_sq.sqrt()
    }

    /// Evaluates whether unit direction `dir` lies inside the active view hemisphere.
    pub fn faces_direction(&self, dir: Vec3) -> bool {
        dir.dot(self.forward) >= self.aperture
    }

    /// Transfers the observer `eye` to a spatial-hash collaborator without exposing internal fields.
    pub fn observe_from<R>(self, f: impl FnOnce(Vec3) -> R) -> R {
        f(self.eye)
    }
}
