use glam::Vec3;
use world::MeshTarget;
use super::spine::RangeScale;

/// `RangeScale` decorator projecting braking and touchdown safety bands beside the laser spine.
#[derive(Clone, Copy, Debug)]
pub struct ApproachZoneScale<S> {
    origin: S,
    caution_ceil: f32,
    safe_ceil: f32,
    range_rate: f32,
}

impl<S> ApproachZoneScale<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, caution_ceil: f32, safe_ceil: f32, range_rate: f32) -> Self {
        Self { origin, caution_ceil, safe_ceil, range_rate }
    }

    pub const fn nose_braking(origin: S, range_rate: f32) -> Self {
        Self::new(origin, 250.0, 100.0, range_rate)
    }

    pub const fn belly_touchdown(origin: S, range_rate: f32) -> Self {
        Self::new(origin, 25.0, 10.0, range_rate)
    }
}

impl<S: RangeScale> RangeScale for ApproachZoneScale<S> {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_scale(target);
        let (low_col, mid_col) = if self.range_rate < -12.5 && self.safe_ceil <= 20.0 {
            (Vec3::new(0.98, 0.28, 0.22), Vec3::new(1.00, 0.56, 0.20))
        } else {
            (Vec3::new(0.22, 0.94, 0.64), Vec3::new(0.98, 0.72, 0.24))
        };
        self.origin.project_band(0.0, self.safe_ceil, low_col, target);
        self.origin.project_band(self.safe_ceil, self.caution_ceil, mid_col, target);
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, color: Vec3, target: &mut dyn MeshTarget) {
        self.origin.project_band(low_meters, high_meters, color, target);
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, accent: Vec3, target: &mut dyn MeshTarget) {
        self.origin.project_chevron(meters, range_rate, accent, target);
    }
}

/// `RangeScale` decorator projecting the live sliding chevron badge and dynamic sink-rate color alert.
#[derive(Clone, Copy, Debug)]
pub struct ChevronReadoutScale<S> {
    origin: S,
    meters: f32,
    range_rate: f32,
    danger_sink: f32,
}

impl<S> ChevronReadoutScale<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, meters: f32, range_rate: f32, danger_sink: f32) -> Self {
        Self { origin, meters, range_rate, danger_sink }
    }

    pub const fn nose_lock(origin: S, meters: f32, range_rate: f32) -> Self {
        Self::new(origin, meters, range_rate, 34.0)
    }

    pub const fn belly_radar(origin: S, meters: f32, range_rate: f32) -> Self {
        Self::new(origin, meters, range_rate, 12.5)
    }
}

impl<S: RangeScale> RangeScale for ChevronReadoutScale<S> {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_scale(target);
        let steep_sink = self.range_rate < -self.danger_sink;
        let caution_sink = self.range_rate < -(self.danger_sink * 0.45);
        let accent = if steep_sink && self.meters < self.danger_sink * 4.5 {
            Vec3::new(0.98, 0.30, 0.24)
        } else if caution_sink && self.meters < self.danger_sink * 3.0 {
            Vec3::new(1.00, 0.74, 0.26)
        } else if self.meters <= 15.0 && self.range_rate <= 0.2 {
            Vec3::new(0.28, 0.96, 0.70)
        } else {
            Vec3::new(0.36, 0.94, 1.00)
        };
        self.origin.project_chevron(self.meters, self.range_rate, accent, target);
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, color: Vec3, target: &mut dyn MeshTarget) {
        self.origin.project_band(low_meters, high_meters, color, target);
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, accent: Vec3, target: &mut dyn MeshTarget) {
        self.origin.project_chevron(meters, range_rate, accent, target);
    }
}
