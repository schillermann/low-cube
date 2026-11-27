use glam::Vec3;
use crate::GravityField;

/// Inverse-square gravitational well decorator tapering smoothly to `0.0g` vacuum beyond atmosphere reach.
pub struct RadialWell<G> {
    origin: G,
    center: Vec3,
    surface_accel: f32,
    crust_half: f32,
}

impl<G> RadialWell<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: G, center: Vec3, surface_accel: f32, crust_half: f32) -> Self {
        Self { origin, center, surface_accel, crust_half }
    }

    pub const fn homeworld(origin: G, crust_half: f32) -> Self {
        Self::new(origin, Vec3::ZERO, 9.81, crust_half)
    }

    pub const fn lunar(origin: G, center: Vec3, crust_half: f32) -> Self {
        Self::new(origin, center, 2.16, crust_half)
    }

    fn halo_span(&self) -> (f32, f32) {
        let knee = if self.crust_half >= 32.0 { 12.0 } else { 8.0 };
        let cutoff = if self.crust_half >= 32.0 { 96.0 } else { 44.0 };
        (self.crust_half + knee, self.crust_half + cutoff)
    }

    fn local_accel(&self, rel: Vec3) -> f32 {
        let dist = rel.length();
        let (r0, r_max) = self.halo_span();
        if dist <= r0 {
            self.surface_accel
        } else if dist >= r_max {
            0.0
        } else {
            let inv_sq = (r0 / dist).powi(2);
            let fade = ((r_max - dist) / (r_max - r0)).clamp(0.0, 1.0);
            self.surface_accel * inv_sq * fade
        }
    }
}

impl<G: GravityField> GravityField for RadialWell<G> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.origin.pull_direction_at(position - self.center)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.origin.surface_up_at(position - self.center)
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        let rel = position - self.center;
        let accel = self.local_accel(rel);
        if accel <= 1e-6 {
            Vec3::ZERO
        } else {
            self.origin.pull_direction_at(rel) * (accel * dt)
        }
    }
}

/// Composite two-body gravitational field summing vector pulls across Homeworld, Vacuum, and Moon without zone branching.
pub struct BinaryGravity<P, S> {
    primary: P,
    secondary: S,
}

impl<P, S> BinaryGravity<P, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(primary: P, secondary: S) -> Self {
        Self { primary, secondary }
    }
}

impl<P: GravityField, S: GravityField> BinaryGravity<P, S> {
    fn secondary_dominates(&self, position: Vec3) -> bool {
        let p_sq = self.primary.apply_pull(position, 1.0).length_squared();
        let s_sq = self.secondary.apply_pull(position, 1.0).length_squared();
        s_sq > p_sq && s_sq > 1e-6
    }
}

impl<P: GravityField, S: GravityField> GravityField for BinaryGravity<P, S> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        if self.secondary_dominates(position) {
            self.secondary.pull_direction_at(position)
        } else {
            self.primary.pull_direction_at(position)
        }
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        if self.secondary_dominates(position) {
            self.secondary.surface_up_at(position)
        } else {
            self.primary.surface_up_at(position)
        }
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        self.primary.apply_pull(position, dt) + self.secondary.apply_pull(position, dt)
    }
}
