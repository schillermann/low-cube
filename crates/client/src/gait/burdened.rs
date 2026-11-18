use glam::Vec3;
use crate::attire::SatchelBurden;
use crate::lens::PerspectiveLens;
use crate::phase::{FlightContext, FlightProgress};
use crate::posture::AvatarPosture;
use crate::threshold::StepThreshold;
use super::{LocomotionGait, WalkGait};

/// Encumbrance gait decorator scaling horizontal stride distance and vertical leap velocity by carried burden.
#[derive(Clone, Copy, Debug)]
pub struct BurdenedGait<G: LocomotionGait = WalkGait> {
    origin: G,
    burden: SatchelBurden,
}

impl<G: LocomotionGait> BurdenedGait<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: G, burden: SatchelBurden) -> Self {
        Self { origin, burden }
    }

    pub fn with_origin<H: LocomotionGait>(self, origin: H) -> BurdenedGait<H> {
        BurdenedGait::new(origin, self.burden)
    }

    pub fn with_burden(self, burden: SatchelBurden) -> Self {
        Self::new(self.origin, burden)
    }
}

impl<G: LocomotionGait> LocomotionGait for BurdenedGait<G> {
    fn pace_distance(&self, distance: f32) -> f32 {
        self.burden.scale_pace(self.origin.pace_distance(distance))
    }

    fn pace_leap(&self, leap_speed: f32) -> f32 {
        self.burden.scale_leap(self.origin.pace_leap(leap_speed))
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.origin.shape_posture(posture, dt)
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.origin.shape_lens(lens, dt)
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
