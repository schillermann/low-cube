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
        altitude <= 1e-4 && altitude >= -self.limit
    }

    /// Determines whether an altitude penetration corresponds to a collision with a vertical cliff face.
    pub fn is_cliff_impact(&self, altitude: f32) -> bool {
        altitude < -self.limit
    }
}

/// Clearance altitude between a 3D spatial position and the planetary surface.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceClearance {
    altitude: f32,
}

impl SurfaceClearance {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(altitude: f32) -> Self {
        Self { altitude }
    }

    /// Secondary constructor measuring clearance between a point and the constrained surface.
    pub fn between(point: Vec3, surface_point: Vec3, surface_up: Vec3) -> Self {
        Self::new((point - surface_point).dot(surface_up))
    }

    /// Determines whether the position is suspended above open air over a precipice.
    pub fn is_precipice(&self) -> bool {
        self.altitude > 1e-4
    }

    /// Checks whether the position makes valid contact with the walkable ground floor.
    pub fn is_floor_contact(&self, threshold: StepThreshold) -> bool {
        threshold.permits_floor_contact(self.altitude)
    }

    /// Checks whether the position penetrates deeply into a vertical cliff wall.
    pub fn is_cliff_impact(&self, threshold: StepThreshold) -> bool {
        threshold.is_cliff_impact(self.altitude)
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

    /// Determines whether this step can be climbed smoothly without jumping.
    pub fn is_walkable_with(&self, threshold: StepThreshold) -> bool {
        let elevation_gain = (self.target - self.origin).dot(self.surface_up);
        threshold.permits_rise(elevation_gain)
    }
}
