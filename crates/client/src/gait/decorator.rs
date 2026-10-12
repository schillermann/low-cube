use glam::Vec3;
use crate::lens::PerspectiveLens;
use crate::phase::{FlightContext, FlightProgress, GroundedFlight};
use crate::posture::AvatarPosture;
use crate::threshold::StepThreshold;
use super::LocomotionGait;

/// Decorator scaling locomotion stride by a pacing multiplier.
pub struct PacedGait {
    origin: Box<dyn LocomotionGait>,
    pace_factor: f32,
}

impl PacedGait {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn LocomotionGait>, pace_factor: f32) -> Self {
        Self { origin, pace_factor }
    }
}

impl LocomotionGait for PacedGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.origin.pace_distance(distance) * self.pace_factor
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.origin.shape_posture(posture, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.origin.shape_lens(lens, dt)
    }

    fn resolve_precipice(&self, ctx: &FlightContext, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.origin.resolve_precipice(ctx, threshold, horizontal)
    }
}

/// Decorator dynamically modifying field of view during high-speed locomotion.
pub struct DynamicLensGait {
    origin: Box<dyn LocomotionGait>,
    fov_radians: f32,
}

impl DynamicLensGait {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn LocomotionGait>, fov_radians: f32) -> Self {
        Self { origin, fov_radians }
    }
}

impl LocomotionGait for DynamicLensGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.origin.pace_distance(distance)
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.origin.shape_posture(posture, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        lens.transition_to(self.fov_radians, dt)
    }

    fn resolve_precipice(&self, ctx: &FlightContext, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.origin.resolve_precipice(ctx, threshold, horizontal)
    }
}

/// Decorator modifying eye elevation posture for sneaking or crouching.
pub struct CrouchingGait {
    origin: Box<dyn LocomotionGait>,
    elevation: f32,
}

impl CrouchingGait {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn LocomotionGait>, elevation: f32) -> Self {
        Self { origin, elevation }
    }
}

impl LocomotionGait for CrouchingGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.origin.pace_distance(distance)
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        posture.transition_to(self.elevation, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.origin.shape_lens(lens, dt)
    }

    fn resolve_precipice(&self, ctx: &FlightContext, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.origin.resolve_precipice(ctx, threshold, horizontal)
    }
}

/// Decorator guarding against falling off edges by sliding along ledges.
pub struct CliffGuardedGait {
    origin: Box<dyn LocomotionGait>,
}

impl CliffGuardedGait {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn LocomotionGait>) -> Self {
        Self { origin }
    }
}

impl LocomotionGait for CliffGuardedGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.origin.pace_distance(distance)
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.origin.shape_posture(posture, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.origin.shape_lens(lens, dt)
    }

    fn resolve_precipice(&self, ctx: &FlightContext, threshold: StepThreshold, _horizontal: Vec3) -> FlightProgress {
        let next_pos = ctx.resolve_ground_stride(threshold);
        FlightProgress::new(Box::new(GroundedFlight::new(threshold)), next_pos)
    }
}
