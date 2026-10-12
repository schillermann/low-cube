use glam::Vec3;
use world::PlanetaryWorld;
use crate::gait::LocomotionGait;
use crate::stride::SurfaceStride;
use crate::threshold::{StepThreshold, SurfaceClearance};

/// Unified physical simulation context for a flight phase transition.
pub struct FlightContext<'a> {
    dt: f32,
    position: Vec3,
    stride: Vec3,
    surface_up: Vec3,
    eye_height: f32,
    gravity: f32,
    gait: &'a dyn LocomotionGait,
    world: &'a dyn PlanetaryWorld,
}

impl<'a> FlightContext<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        dt: f32,
        position: Vec3,
        stride: Vec3,
        surface_up: Vec3,
        eye_height: f32,
        gravity: f32,
        gait: &'a dyn LocomotionGait,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self {
            dt,
            position,
            stride,
            surface_up,
            eye_height,
            gravity,
            gait,
            world,
        }
    }

    pub fn has_stride(&self) -> bool {
        self.stride.length_squared() > 1e-6
    }

    pub fn candidate_horizontal(&self) -> Vec3 {
        if self.has_stride() {
            self.position + self.stride
        } else {
            self.position
        }
    }

    pub fn constrain_surface(&self, point: Vec3) -> Vec3 {
        self.world.constrain_to_surface(point, self.eye_height)
    }

    pub fn clearance(&self, point: Vec3, surface_point: Vec3) -> SurfaceClearance {
        SurfaceClearance::between(point, surface_point, self.surface_up)
    }

    pub fn gravity_velocity_delta(&self) -> f32 {
        self.gravity * self.dt
    }

    pub fn vertical_displacement(&self, speed: f32) -> Vec3 {
        self.surface_up * (speed * self.dt)
    }

    pub fn resolve_ground_stride(&self, threshold: StepThreshold) -> Vec3 {
        SurfaceStride::new(self.stride, self.surface_up)
            .resolve_ground(self.position, self.eye_height, threshold, self.world)
    }

    pub fn resolve_aerial_stride(&self, vertical_step: Vec3, threshold: StepThreshold) -> Vec3 {
        SurfaceStride::new(self.stride, self.surface_up)
            .resolve_aerial(self.position, vertical_step, self.eye_height, threshold, self.world)
    }

    pub fn resolve_precipice(&self, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.gait.resolve_precipice(self, threshold, horizontal)
    }
}

/// Outcome of a flight phase transition containing the subsequent phase and updated position.
pub struct FlightProgress {
    phase: Box<dyn FlightPhase>,
    position: Vec3,
}

impl FlightProgress {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(phase: Box<dyn FlightPhase>, position: Vec3) -> Self {
        Self { phase, position }
    }

    /// Mutates the caller's active phase reference and yields the updated spatial position.
    pub fn apply(self, flight: &mut Box<dyn FlightPhase>) -> Vec3 {
        *flight = self.phase;
        self.position
    }
}

/// Interface for polymorphic vertical flight phases under planetary gravity.
pub trait FlightPhase: Send + Sync {
    fn jump(&self, strength: f32) -> Box<dyn FlightPhase>;
    fn advance(&self, context: &FlightContext) -> FlightProgress;
}

/// Grounded state resting on the planetary surface floor.
pub struct GroundedFlight {
    threshold: StepThreshold,
}

impl GroundedFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(threshold: StepThreshold) -> Self {
        Self { threshold }
    }

    /// Secondary constructor with standard human stride calibration.
    pub fn human() -> Self {
        Self::new(StepThreshold::human_stride())
    }
}

impl FlightPhase for GroundedFlight {
    fn jump(&self, strength: f32) -> Box<dyn FlightPhase> {
        Box::new(AirborneFlight::new(strength, self.threshold))
    }

    fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        let horizontal = ctx.candidate_horizontal();
        let surface_pos = ctx.constrain_surface(horizontal);
        let clearance = ctx.clearance(horizontal, surface_pos);

        if clearance.is_precipice() {
            ctx.resolve_precipice(self.threshold, horizontal)
        } else {
            let next_pos = ctx.resolve_ground_stride(self.threshold);
            FlightProgress::new(Box::new(GroundedFlight::new(self.threshold)), next_pos)
        }
    }
}

/// Airborne state subjected to gravitational acceleration.
pub struct AirborneFlight {
    vertical_speed: f32,
    threshold: StepThreshold,
}

impl AirborneFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertical_speed: f32, threshold: StepThreshold) -> Self {
        Self {
            vertical_speed,
            threshold,
        }
    }
}

impl FlightPhase for AirborneFlight {
    fn jump(&self, _strength: f32) -> Box<dyn FlightPhase> {
        Box::new(AirborneFlight::new(self.vertical_speed, self.threshold))
    }

    fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        let next_speed = self.vertical_speed - ctx.gravity_velocity_delta();
        let vertical_step = ctx.vertical_displacement(next_speed);

        let rebound = ctx.resolve_aerial_stride(vertical_step, self.threshold);
        let supported = ctx.constrain_surface(rebound);
        let clearance = ctx.clearance(rebound, supported);

        if next_speed <= 0.0 && clearance.is_floor_contact(self.threshold) {
            FlightProgress::new(Box::new(GroundedFlight::new(self.threshold)), supported)
        } else {
            FlightProgress::new(Box::new(AirborneFlight::new(next_speed, self.threshold)), rebound)
        }
    }
}
