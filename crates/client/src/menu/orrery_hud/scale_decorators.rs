use glam::Vec3;
use world::{MeshTarget, Quad};
use super::phosphor::{ClosureKinematics, PhosphorTone};
use super::spine::{RangeScale, ScaleDomain, SPINE_BOT, SPINE_TOP};

/// `RangeScale` decorator projecting a translucent phosphor-green HUD glass plate behind an inner scale.
#[derive(Clone, Copy, Debug)]
pub struct GlassPlateScale<S> {
    origin: S,
    inv_aspect: f32,
}

impl<S> GlassPlateScale<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, inv_aspect: f32) -> Self {
        Self { origin, inv_aspect }
    }
}

impl<S: RangeScale> RangeScale for GlassPlateScale<S> {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let (bx0, bx1) = (-0.020 * ux, 0.128 * ux);
        let (by0, by1) = (SPINE_BOT - 0.020, SPINE_TOP + 0.052);
        target.push_quad(Quad::glowing(
            Vec3::new(bx0, by0, 0.0),
            Vec3::new(bx1, by0, 0.0),
            Vec3::new(bx1, by1, 0.0),
            Vec3::new(bx0, by1, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            PhosphorTone::GlassTint.color(),
            -0.36,
        ));
        self.origin.emit_scale(target);
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        self.origin.project_band(low_meters, high_meters, tone, target);
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        self.origin.project_chevron(meters, range_rate, tone, target);
    }
}

/// `RangeScale` decorator projecting braking and touchdown safety bands beside the laser spine.
#[derive(Clone, Copy, Debug)]
pub struct ApproachZoneScale<S> {
    origin: S,
    safe_ceil: f32,
    caution_ceil: f32,
    tones: (PhosphorTone, PhosphorTone),
}

impl<S> ApproachZoneScale<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        safe_ceil: f32,
        caution_ceil: f32,
        tones: (PhosphorTone, PhosphorTone),
    ) -> Self {
        Self { origin, safe_ceil, caution_ceil, tones }
    }

    pub fn for_domain(origin: S, domain: ScaleDomain, kinematics: ClosureKinematics) -> Self {
        let (safe_ceil, caution_ceil) = domain.zone_ceilings();
        Self::new(origin, safe_ceil, caution_ceil, kinematics.zone_tones(domain.danger_sink()))
    }

    #[cfg(test)]
    pub fn nose_braking(origin: S, range_rate: f32) -> Self {
        Self::for_domain(origin, ScaleDomain::CosmicTarget, ClosureKinematics::new(0.0, range_rate))
    }

    #[cfg(test)]
    pub fn belly_touchdown(origin: S, range_rate: f32) -> Self {
        Self::for_domain(origin, ScaleDomain::LunarPad, ClosureKinematics::new(0.0, range_rate))
    }
}

impl<S: RangeScale> RangeScale for ApproachZoneScale<S> {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_scale(target);
        let (low_tone, mid_tone) = self.tones;
        self.origin.project_band(0.0, self.safe_ceil, low_tone, target);
        self.origin.project_band(self.safe_ceil, self.caution_ceil, mid_tone, target);
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        self.origin.project_band(low_meters, high_meters, tone, target);
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        self.origin.project_chevron(meters, range_rate, tone, target);
    }
}

/// `RangeScale` decorator projecting the live sliding chevron badge and dynamic sink-rate phosphor intensity.
#[derive(Clone, Copy, Debug)]
pub struct ChevronReadoutScale<S> {
    origin: S,
    kinematics: ClosureKinematics,
    danger_sink: f32,
}

impl<S> ChevronReadoutScale<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, kinematics: ClosureKinematics, danger_sink: f32) -> Self {
        Self { origin, kinematics, danger_sink }
    }

    pub const fn for_domain(origin: S, domain: ScaleDomain, kinematics: ClosureKinematics) -> Self {
        Self::new(origin, kinematics, domain.danger_sink())
    }

    #[cfg(test)]
    pub const fn nose_lock(origin: S, meters: f32, range_rate: f32) -> Self {
        Self::for_domain(origin, ScaleDomain::CosmicTarget, ClosureKinematics::new(meters, range_rate))
    }

    #[cfg(test)]
    pub const fn belly_radar(origin: S, meters: f32, range_rate: f32) -> Self {
        Self::for_domain(origin, ScaleDomain::LunarPad, ClosureKinematics::new(meters, range_rate))
    }
}

impl<S: RangeScale> RangeScale for ChevronReadoutScale<S> {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_scale(target);
        self.kinematics.project_onto(&self.origin, self.danger_sink, target);
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        self.origin.project_band(low_meters, high_meters, tone, target);
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        self.origin.project_chevron(meters, range_rate, tone, target);
    }
}
