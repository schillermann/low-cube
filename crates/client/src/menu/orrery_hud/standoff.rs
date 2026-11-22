use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::menu::widget::ScreenPoint;
use super::phosphor::{ClosureKinematics, PhosphorTone};
use super::scale_decorators::{ApproachZoneScale, ChevronReadoutScale, GlassPlateScale};
use super::spine::{GraduatedSpine, RangeScale, ScaleDomain};

const RETICLE_STANDOFF_X: f32 = 0.25;

/// `MeshTarget` decorator translating every emitted quad by a fixed HUD screen offset.
struct ShiftedTarget<'a> {
    origin: &'a mut dyn MeshTarget,
    offset: Vec3,
}

impl<'a> ShiftedTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(origin: &'a mut dyn MeshTarget, offset: Vec3) -> Self {
        Self { origin, offset }
    }
}

impl MeshTarget for ShiftedTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.origin.push_quad(quad.shifted(self.offset));
    }
}

/// `RangeScale` decorator anchoring an inner scale beside the boresight crosshair with a clear HUD standoff gap.
#[derive(Clone, Copy, Debug)]
pub struct ReticleStandoffScale<S> {
    origin: S,
    reticle: ScreenPoint,
    standoff_x: f32,
    inv_aspect: f32,
}

impl<S> ReticleStandoffScale<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, reticle: ScreenPoint, standoff_x: f32, inv_aspect: f32) -> Self {
        Self { origin, reticle, standoff_x, inv_aspect }
    }

    pub const fn beside(origin: S, reticle: ScreenPoint, inv_aspect: f32) -> Self {
        Self::new(origin, reticle, RETICLE_STANDOFF_X, inv_aspect)
    }

    fn shift_vector(&self) -> Vec3 {
        self.reticle.transfer(|rx, ry| Vec3::new(rx + self.standoff_x * self.inv_aspect, ry, 0.0))
    }
}

impl<S: RangeScale> RangeScale for ReticleStandoffScale<S> {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        let mut shifted = ShiftedTarget::new(target, self.shift_vector());
        self.origin.emit_scale(&mut shifted);
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        let mut shifted = ShiftedTarget::new(target, self.shift_vector());
        self.origin.project_band(low_meters, high_meters, tone, &mut shifted);
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        let mut shifted = ShiftedTarget::new(target, self.shift_vector());
        self.origin.project_chevron(meters, range_rate, tone, &mut shifted);
    }
}

/// Complete boresight-locked HUD range tape composing all five `RangeScale` layers without boilerplate duplication.
#[derive(Clone, Copy, Debug)]
pub struct BoresightTape {
    domain: ScaleDomain,
    reticle: ScreenPoint,
    kinematics: ClosureKinematics,
    inv_aspect: f32,
}

impl BoresightTape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        domain: ScaleDomain,
        reticle: ScreenPoint,
        kinematics: ClosureKinematics,
        inv_aspect: f32,
    ) -> Self {
        Self { domain, reticle, kinematics, inv_aspect }
    }

    pub const fn nose(reticle: ScreenPoint, meters: f32, range_rate: f32, inv_aspect: f32) -> Self {
        Self::new(ScaleDomain::CosmicTarget, reticle, ClosureKinematics::new(meters, range_rate), inv_aspect)
    }

    pub const fn belly(reticle: ScreenPoint, meters: f32, range_rate: f32, inv_aspect: f32) -> Self {
        Self::new(ScaleDomain::LunarPad, reticle, ClosureKinematics::new(meters, range_rate), inv_aspect)
    }
}

impl RangeScale for BoresightTape {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        ReticleStandoffScale::beside(
            ChevronReadoutScale::for_domain(
                ApproachZoneScale::for_domain(
                    GlassPlateScale::new(GraduatedSpine::new(self.domain, ux), ux),
                    self.domain,
                    self.kinematics,
                ),
                self.domain,
                self.kinematics,
            ),
            self.reticle,
            ux,
        )
        .emit_scale(target);
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        ReticleStandoffScale::beside(GraduatedSpine::new(self.domain, self.inv_aspect), self.reticle, self.inv_aspect)
            .project_band(low_meters, high_meters, tone, target);
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        ReticleStandoffScale::beside(GraduatedSpine::new(self.domain, self.inv_aspect), self.reticle, self.inv_aspect)
            .project_chevron(meters, range_rate, tone, target);
    }
}
