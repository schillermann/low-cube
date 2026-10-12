use glam::Vec3;
use crate::phase::{AirborneFlight, FlightContext, FlightProgress, GroundedFlight};
use crate::posture::AvatarPosture;
use crate::threshold::StepThreshold;

/// Locomotion gait determining pacing scale, posture shaping, and precipice crossing behavior.
pub trait LocomotionGait: Send + Sync {
    fn pace_distance(&self, distance: f32) -> f32;
    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture;
    fn resolve_precipice(
        &self,
        ctx: &FlightContext,
        threshold: StepThreshold,
        horizontal: Vec3,
    ) -> FlightProgress;
}

/// Upright walking gait moving at standard pace and stepping off ledges into free flight.
pub struct WalkGait;

impl WalkGait {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

impl LocomotionGait for WalkGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        distance
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        posture.transition_to(0.8, dt)
    }

    fn resolve_precipice(
        &self,
        _ctx: &FlightContext,
        threshold: StepThreshold,
        horizontal: Vec3,
    ) -> FlightProgress {
        FlightProgress::new(Box::new(AirborneFlight::new(0.0, threshold)), horizontal)
    }
}

/// Careful sneaking gait moving at deliberate pace and guarding against drops from elevated blocks.
pub struct SneakGait;

impl SneakGait {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

impl LocomotionGait for SneakGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        distance * 0.35
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        posture.transition_to(0.45, dt)
    }

    fn resolve_precipice(
        &self,
        ctx: &FlightContext,
        threshold: StepThreshold,
        _horizontal: Vec3,
    ) -> FlightProgress {
        let next_pos = ctx.resolve_ground_stride(threshold);
        FlightProgress::new(Box::new(GroundedFlight::new(threshold)), next_pos)
    }
}
