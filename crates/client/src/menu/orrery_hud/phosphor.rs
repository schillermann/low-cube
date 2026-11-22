use glam::Vec3;
use world::MeshTarget;
use super::spine::RangeScale;

/// Monochromatic avionics Head-Up Display phosphor-green luminance tiers sharing a constant 129° hue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhosphorTone {
    GlassTint,
    BadgeWell,
    FarBand,
    MinorTick,
    NearBand,
    SpineRule,
    StationLabel,
    NominalBeam,
    CautionBeam,
    PeakPhosphor,
}

impl PhosphorTone {
    const fn pair(floor: f32, green: f32) -> Vec3 {
        Vec3::new(floor, green, floor + 0.15 * (green - floor))
    }

    pub const fn color(self) -> Vec3 {
        match self {
            Self::GlassTint => Self::pair(0.012, 0.080),
            Self::BadgeWell => Self::pair(0.020, 0.130),
            Self::FarBand => Self::pair(0.048, 0.320),
            Self::MinorTick => Self::pair(0.066, 0.440),
            Self::NearBand => Self::pair(0.084, 0.560),
            Self::SpineRule => Self::pair(0.099, 0.660),
            Self::StationLabel => Self::pair(0.117, 0.780),
            Self::NominalBeam => Self::pair(0.132, 0.880),
            Self::CautionBeam => Self::pair(0.220, 0.950),
            Self::PeakPhosphor => Self::pair(0.440, 1.000),
        }
    }

    pub fn readout_core(self) -> Vec3 {
        (self.color() * 0.45 + Self::PeakPhosphor.color() * 0.55).clamp(Vec3::ZERO, Vec3::ONE)
    }
}

/// Immutable closure distance and signed range-rate formatting and phosphor urgency evaluation.
#[derive(Clone, Copy, Debug)]
pub struct ClosureKinematics {
    meters: f32,
    range_rate: f32,
}

impl ClosureKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(meters: f32, range_rate: f32) -> Self {
        Self { meters, range_rate }
    }

    pub fn distance_label(self) -> String {
        let d = self.meters.max(0.0);
        if d >= 1_000.0 {
            format!("{:.2} KM", d * 0.001)
        } else if d < 20.0 {
            format!("{:.1} M", d)
        } else {
            format!("{:.0} M", d)
        }
    }

    pub fn rate_label(self) -> String {
        if self.range_rate.abs() < 0.15 {
            "0.0 M/S".to_string()
        } else if self.range_rate < 0.0 {
            if self.range_rate > -10.0 {
                format!("{:.1} M/S", self.range_rate)
            } else {
                format!("{:.0} M/S", self.range_rate)
            }
        } else if self.range_rate < 10.0 {
            format!("+{:.1} M/S", self.range_rate)
        } else {
            format!("+{:.0} M/S", self.range_rate)
        }
    }

    pub fn zone_tones(self, danger_sink: f32) -> (PhosphorTone, PhosphorTone) {
        if self.range_rate < -danger_sink {
            (PhosphorTone::PeakPhosphor, PhosphorTone::CautionBeam)
        } else {
            (PhosphorTone::NearBand, PhosphorTone::FarBand)
        }
    }

    pub fn chevron_tone(self, danger_sink: f32) -> PhosphorTone {
        if self.range_rate < -danger_sink && self.meters < danger_sink * 4.5 {
            PhosphorTone::PeakPhosphor
        } else if self.range_rate < -(danger_sink * 0.45) && self.meters < danger_sink * 3.0 {
            PhosphorTone::CautionBeam
        } else if self.meters <= 15.0 && self.range_rate <= 0.2 {
            PhosphorTone::StationLabel
        } else {
            PhosphorTone::NominalBeam
        }
    }

    pub fn project_onto<S: RangeScale>(self, scale: &S, danger_sink: f32, target: &mut dyn MeshTarget) {
        scale.project_chevron(self.meters, self.range_rate, self.chevron_tone(danger_sink), target);
    }
}
