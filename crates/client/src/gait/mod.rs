use glam::Vec3;
use crate::lens::PerspectiveLens;
use crate::phase::{AirborneFlight, FlightContext, FlightPhase, FlightProgress, GroundedFlight};
use crate::posture::AvatarPosture;
use crate::threshold::StepThreshold;

/// Locomotion gait determining pacing scale, posture shaping, and precipice crossing behavior.
pub trait LocomotionGait: Send + Sync {
    fn pace_distance(&self, distance: f32) -> f32;
    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture;
    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens;
    fn permits_seam_fold(&self) -> bool;
    fn resolve_precipice(
        &self,
        ctx: &FlightContext,
        threshold: StepThreshold,
        horizontal: Vec3,
    ) -> FlightProgress;
}

/// Upright walking gait moving at standard pace and stepping off ledges into free flight.
#[derive(Clone, Copy, Debug)]
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

    fn permits_seam_fold(&self) -> bool {
        true
    }

    fn resolve_precipice(
        &self,
        _ctx: &FlightContext,
        threshold: StepThreshold,
        horizontal: Vec3,
    ) -> FlightProgress {
        FlightProgress::new(FlightPhase::Airborne(AirborneFlight::new(0.0, threshold)), horizontal)
    }
}

/// Careful sneaking gait decorating an underlying locomotion gait with crouch posture and cliff arrest.
#[derive(Clone, Copy, Debug)]
pub struct SneakGait<G: LocomotionGait = WalkGait> {
    origin: G,
    pace_factor: f32,
    elevation: f32,
}

impl<G: LocomotionGait> SneakGait<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: G, pace_factor: f32, elevation: f32) -> Self {
        Self {
            origin,
            pace_factor,
            elevation,
        }
    }
}

impl SneakGait<WalkGait> {
    /// Secondary constructor wrapping standard upright gait in crouching posture.
    pub fn crouching() -> Self {
        Self::new(WalkGait::upright(), 0.35, 0.45)
    }
}

impl<G: LocomotionGait> LocomotionGait for SneakGait<G> {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.origin.pace_distance(distance) * self.pace_factor
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        posture.transition_to(self.elevation, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.origin.shape_lens(lens, dt)
    }

    fn permits_seam_fold(&self) -> bool {
        false
    }

    fn resolve_precipice(
        &self,
        ctx: &FlightContext,
        threshold: StepThreshold,
        _horizontal: Vec3,
    ) -> FlightProgress {
        let arrested = ctx.resolve_ground_stride(threshold);
        FlightProgress::new(FlightPhase::Grounded(GroundedFlight::new(threshold)), arrested)
    }
}

/// High-speed sprinting gait decorating an underlying locomotion gait with dynamic lens expansion and swift pace.
#[derive(Clone, Copy, Debug)]
pub struct SprintGait<G: LocomotionGait = WalkGait> {
    origin: G,
    pace_factor: f32,
    fov_radians: f32,
}

impl<G: LocomotionGait> SprintGait<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: G, pace_factor: f32, fov_radians: f32) -> Self {
        Self {
            origin,
            pace_factor,
            fov_radians,
        }
    }
}

impl SprintGait<WalkGait> {
    /// Secondary constructor wrapping standard upright gait in swift sprint pacing.
    pub fn swift() -> Self {
        Self::new(WalkGait::upright(), 1.6, 78.0f32.to_radians())
    }
}

impl<G: LocomotionGait> LocomotionGait for SprintGait<G> {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.origin.pace_distance(distance) * self.pace_factor
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.origin.shape_posture(posture, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        lens.transition_to(self.fov_radians, dt)
    }

    fn permits_seam_fold(&self) -> bool {
        self.origin.permits_seam_fold()
    }

    fn resolve_precipice(
        &self,
        ctx: &FlightContext,
        threshold: StepThreshold,
        horizontal: Vec3,
    ) -> FlightProgress {
        self.origin.resolve_precipice(ctx, threshold, horizontal)
    }
}
