use glam::Vec3;
use crate::lens::PerspectiveLens;
use crate::phase::{AirborneFlight, FlightContext, FlightProgress};
use crate::posture::AvatarPosture;
use crate::threshold::StepThreshold;

mod decorator;

pub use decorator::{CliffGuardedGait, CrouchingGait, DynamicLensGait, PacedGait};

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
    pub fn new(elevation: f32, fov_radians: f32) -> Self {
        Self { elevation, fov_radians }
    }

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

/// Careful sneaking gait composed of cliff-guard, crouched posture, and reduced pacing decorators.
pub struct SneakGait {
    decorated: CliffGuardedGait,
}

impl SneakGait {
    pub fn new(decorated: CliffGuardedGait) -> Self {
        Self { decorated }
    }

    pub fn crouching() -> Self {
        Self::new(CliffGuardedGait::new(Box::new(CrouchingGait::new(
            Box::new(PacedGait::new(Box::new(WalkGait::upright()), 0.35)),
            0.45,
        ))))
    }
}

impl LocomotionGait for SneakGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.decorated.pace_distance(distance)
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.decorated.shape_posture(posture, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.decorated.shape_lens(lens, dt)
    }

    fn resolve_precipice(&self, ctx: &FlightContext, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.decorated.resolve_precipice(ctx, threshold, horizontal)
    }
}

/// High-speed sprinting gait composed of dynamic lens expansion and accelerated pacing decorators.
pub struct SprintGait {
    decorated: PacedGait,
}

impl SprintGait {
    pub fn new(decorated: PacedGait) -> Self {
        Self { decorated }
    }

    pub fn swift() -> Self {
        Self::new(PacedGait::new(
            Box::new(DynamicLensGait::new(
                Box::new(WalkGait::upright()),
                78.0f32.to_radians(),
            )),
            1.6,
        ))
    }
}

impl LocomotionGait for SprintGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.decorated.pace_distance(distance)
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.decorated.shape_posture(posture, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.decorated.shape_lens(lens, dt)
    }

    fn resolve_precipice(&self, ctx: &FlightContext, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.decorated.resolve_precipice(ctx, threshold, horizontal)
    }
}
