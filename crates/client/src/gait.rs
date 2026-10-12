use glam::Vec3;
use crate::lens::PerspectiveLens;
use crate::phase::{AirborneFlight, FlightContext, FlightProgress, GroundedFlight};
use crate::posture::AvatarPosture;
use crate::threshold::StepThreshold;

/// Locomotion gait determining pacing scale, posture shaping, and precipice crossing behavior.
pub trait LocomotionGait: Send + Sync {
    fn pace_distance(&self, distance: f32) -> f32;
    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture;
    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens;
    fn resolve_precipice(
        &self,
        ctx: &FlightContext,
        threshold: StepThreshold,
        horizontal: Vec3,
    ) -> FlightProgress;
}

/// Upright walking gait moving at standard pace and stepping off ledges into free flight.
pub struct WalkGait {
    elevation: f32,
    fov_radians: f32,
}

impl WalkGait {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(elevation: f32, fov_radians: f32) -> Self {
        Self {
            elevation,
            fov_radians,
        }
    }

    /// Secondary constructor for natural upright walking gait.
    pub fn upright() -> Self {
        Self::new(0.8, 65.0f32.to_radians())
    }
}

impl LocomotionGait for WalkGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        distance
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        posture.transition_to(self.elevation, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        lens.transition_to(self.fov_radians, dt)
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
pub struct SneakGait {
    pace_factor: f32,
    elevation: f32,
    fov_radians: f32,
}

impl SneakGait {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(pace_factor: f32, elevation: f32, fov_radians: f32) -> Self {
        Self {
            pace_factor,
            elevation,
            fov_radians,
        }
    }

    /// Secondary constructor for crouching sneak posture.
    pub fn crouching() -> Self {
        Self::new(0.35, 0.45, 65.0f32.to_radians())
    }
}

impl LocomotionGait for SneakGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        distance * self.pace_factor
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        posture.transition_to(self.elevation, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        lens.transition_to(self.fov_radians, dt)
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

/// High-speed sprinting gait moving at rapid pace, widening FOV, and stepping off ledges into free flight.
pub struct SprintGait {
    pace_factor: f32,
    elevation: f32,
    fov_radians: f32,
}

impl SprintGait {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(pace_factor: f32, elevation: f32, fov_radians: f32) -> Self {
        Self {
            pace_factor,
            elevation,
            fov_radians,
        }
    }

    /// Secondary constructor for swift high-speed sprint.
    pub fn swift() -> Self {
        Self::new(1.6, 0.8, 78.0f32.to_radians())
    }
}

impl LocomotionGait for SprintGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        distance * self.pace_factor
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        posture.transition_to(self.elevation, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        lens.transition_to(self.fov_radians, dt)
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
