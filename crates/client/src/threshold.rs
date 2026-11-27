use glam::Vec3;

/// Kinematic threshold defining permissible vertical variations for human surface traversal.
#[derive(Clone, Copy, Debug)]
pub struct StepThreshold {
    limit: f32,
}

impl StepThreshold {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(limit: f32) -> Self {
        Self { limit }
    }

    /// Secondary constructor calibrated for human rock shelf and 45° slope climbing.
    pub fn human_stride() -> Self {
        Self::new(0.35)
    }

    /// Verifies whether an elevation rise is within walkable climbing limits.
    pub fn permits_rise(&self, rise: f32) -> bool {
        rise <= self.limit
    }

    /// Checks whether an altitude difference corresponds to contact with the ground surface.
    pub fn permits_floor_contact(&self, altitude: f32) -> bool {
        altitude <= 2e-4 && altitude >= -self.limit
    }

    /// Determines whether an altitude penetration corresponds to a collision with a vertical cliff face.
    pub fn cliff_impact(&self, altitude: f32) -> bool {
        altitude < -self.limit
    }

    /// Verifies whether an elevation change is within continuous slope traversal limits.
    pub fn permits_slope(&self, elevation_change: f32) -> bool {
        elevation_change.abs() <= self.limit
    }
}

use world::PlanetaryWorld;

/// Polymorphic clearance between a 3D spatial position and the planetary surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurfaceClearance {
    Planar(f32),
    FoldedSeam(f32),
}

impl SurfaceClearance {
    /// Evaluates clearance between candidate motion and constrained surface across planetary topography.
    pub fn between(
        origin: Vec3,
        candidate: Vec3,
        surface_pos: Vec3,
        surface_up: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Self {
        let dest_up = world.surface_up_at(surface_pos);
        let cos_up = surface_up.dot(dest_up);
        if cos_up < 0.7 {
            let chord_distance = (surface_pos - origin).length();
            if chord_distance < 6.5 {
                return Self::FoldedSeam((candidate - surface_pos).dot(dest_up));
            }
        }
        let d = candidate - origin;
        let d_tan = d - surface_up * d.dot(surface_up);
        let curvature_lift = if cos_up > 0.5 {
            (d_tan.dot(dest_up - surface_up) / (1.0 + cos_up)).max(0.0)
        } else {
            0.0
        };
        Self::Planar((candidate - surface_pos).dot(dest_up) - curvature_lift)
    }

    /// Checks whether this surface clearance represents passable ground under a given seam traversal policy.
    pub fn passable_with(&self, seam_allowed: bool) -> bool {
        match self {
            Self::Planar(altitude) => *altitude <= 2e-4,
            Self::FoldedSeam(_) => seam_allowed,
        }
    }

    /// Checks whether the position makes valid contact with the walkable ground floor.
    pub fn floor_contact(&self, threshold: StepThreshold) -> bool {
        match self {
            Self::Planar(altitude) => threshold.permits_floor_contact(*altitude),
            Self::FoldedSeam(dest_altitude) => *dest_altitude <= 2e-4,
        }
    }

    /// Checks whether the position penetrates deeply into a vertical cliff wall.
    pub fn cliff_impact(&self, threshold: StepThreshold) -> bool {
        match self {
            Self::Planar(altitude) => threshold.cliff_impact(*altitude),
            Self::FoldedSeam(_) => false,
        }
    }
}

/// Encapsulated step translation across a planetary surface.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceStep {
    origin: Vec3,
    target: Vec3,
    surface_up: Vec3,
}

impl SurfaceStep {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Vec3, target: Vec3, surface_up: Vec3) -> Self {
        Self {
            origin,
            target,
            surface_up,
        }
    }

    /// Determines the elevation rise or drop along this step.
    pub fn elevation_gain(&self) -> f32 {
        (self.target - self.origin).dot(self.surface_up)
    }

    /// Determines whether this step can be climbed smoothly without jumping.
    pub fn walkable_with(&self, threshold: StepThreshold) -> bool {
        threshold.permits_rise(self.elevation_gain())
    }

    /// Computes the normalized slope gradient clamped to a unit slope ratio.
    pub fn gradient(&self) -> f32 {
        let rise = self.elevation_gain();
        let horizontal = self.target - self.origin - self.surface_up * rise;
        let run = horizontal.length();
        if run > 1e-4 {
            (rise / run).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }
}
