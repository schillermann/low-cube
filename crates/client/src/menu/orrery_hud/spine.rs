use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::menu::glyph::PixelStamp;
use crate::menu::widget::ScreenBounds;
use super::phosphor::{ClosureKinematics, PhosphorTone};

pub(super) const SPINE_BOT: f32 = -0.20;
pub(super) const SPINE_TOP: f32 = 0.20;

/// Focused vertical laser-scale contract supporting Yegor-style decorator composition without getters.
pub trait RangeScale {
    fn emit_scale(&self, target: &mut dyn MeshTarget);
    fn project_band(&self, low_meters: f32, high_meters: f32, tone: PhosphorTone, target: &mut dyn MeshTarget);
    fn project_chevron(&self, meters: f32, range_rate: f32, tone: PhosphorTone, target: &mut dyn MeshTarget);
}

/// Polymorphic domain calibration between Nose-Camera cosmic target range and Belly-Camera lunar pad clearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleDomain {
    CosmicTarget,
    LunarPad,
}

impl ScaleDomain {
    pub const fn span_meters(self) -> f32 {
        match self {
            Self::CosmicTarget => 1_000.0,
            Self::LunarPad => 100.0,
        }
    }

    pub const fn zone_ceilings(self) -> (f32, f32) {
        match self {
            Self::CosmicTarget => (100.0, 250.0),
            Self::LunarPad => (10.0, 25.0),
        }
    }

    pub const fn danger_sink(self) -> f32 {
        match self {
            Self::CosmicTarget => 34.0,
            Self::LunarPad => 12.5,
        }
    }

    const fn caption(self) -> &'static str {
        match self {
            Self::CosmicTarget => "TGT",
            Self::LunarPad => "ALT",
        }
    }

    const fn stations(self) -> [(f32, &'static str); 5] {
        match self {
            Self::CosmicTarget => [
                (0.0, "0M"),
                (100.0, "100M"),
                (250.0, "250M"),
                (500.0, "500M"),
                (1_000.0, "1000M"),
            ],
            Self::LunarPad => [
                (0.0, "0M"),
                (10.0, "10M"),
                (25.0, "25M"),
                (50.0, "50M"),
                (100.0, "100M"),
            ],
        }
    }

    fn map_y(self, meters: f32) -> f32 {
        let u = (meters.max(0.0) / self.span_meters()).clamp(0.0, 1.0).sqrt();
        SPINE_BOT + (SPINE_TOP - SPINE_BOT) * u
    }
}

/// Self-drawing HUD pointer triangle, dark-phosphor readout well, and two-line metric stamp.
#[derive(Clone, Copy, Debug)]
struct ChevronBadge {
    y: f32,
    kinematics: ClosureKinematics,
    tone: PhosphorTone,
    inv_aspect: f32,
}

impl ChevronBadge {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(
        y: f32,
        kinematics: ClosureKinematics,
        tone: PhosphorTone,
        inv_aspect: f32,
    ) -> Self {
        Self { y, kinematics, tone, inv_aspect }
    }

    fn emit(self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let y_cur = self.y;
        let accent = self.tone.color();
        if y_cur > SPINE_BOT + 1e-4 {
            ScreenBounds::new(-0.0024 * ux, SPINE_BOT, 0.0024 * ux, y_cur).emit_fill(accent, target);
        }
        target.push_quad(Quad::new(
            Vec3::new(0.005 * ux, y_cur, 0.0),
            Vec3::new(0.023 * ux, y_cur - 0.013, 0.0),
            Vec3::new(0.023 * ux, y_cur + 0.013, 0.0),
            Vec3::new(0.023 * ux, y_cur + 0.013, 0.0),
            Vec3::Z,
            accent,
        ));
        let badge = ScreenBounds::new(0.023 * ux, y_cur - 0.029, 0.205 * ux, y_cur + 0.029);
        badge.emit_fill(PhosphorTone::BadgeWell.color(), target);
        badge.emit_outline(accent, 0.0032 * ux, 0.0032, target);

        let cx = badge.center_x();
        PixelStamp::new(0.0048 * ux, 0.0068, self.tone.readout_core())
            .emit_centered(&self.kinematics.distance_label(), cx, y_cur + 0.010, target);
        PixelStamp::new(0.0035 * ux, 0.0048, accent)
            .emit_centered(&self.kinematics.rate_label(), cx, y_cur - 0.012, target);
    }
}

/// Origin-local graduated vertical laser spine (`x = 0`) decorated by glass, zone, chevron, and standoff layers.
#[derive(Clone, Copy, Debug)]
pub struct GraduatedSpine {
    domain: ScaleDomain,
    inv_aspect: f32,
}

impl GraduatedSpine {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(domain: ScaleDomain, inv_aspect: f32) -> Self {
        Self { domain, inv_aspect }
    }
}

impl RangeScale for GraduatedSpine {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let spine_col = PhosphorTone::SpineRule.color();
        let minor_col = PhosphorTone::MinorTick.color();
        let label_col = PhosphorTone::StationLabel.color();
        ScreenBounds::new(-0.0022 * ux, SPINE_BOT, 0.0022 * ux, SPINE_TOP)
            .emit_fill(spine_col, target);
        PixelStamp::new(0.0044 * ux, 0.0062, PhosphorTone::NominalBeam.color())
            .emit_centered(self.domain.caption(), 0.036 * ux, SPINE_TOP + 0.030, target);

        for step in 1..16 {
            let y = SPINE_BOT + (SPINE_TOP - SPINE_BOT) * ((step as f32) / 16.0);
            ScreenBounds::new(0.0022 * ux, y - 0.0012, 0.011 * ux, y + 0.0012)
                .emit_fill(minor_col, target);
        }
        let stamp = PixelStamp::new(0.0038 * ux, 0.0052, label_col);
        for (station_m, text) in self.domain.stations() {
            let y = self.domain.map_y(station_m);
            ScreenBounds::new(0.0022 * ux, y - 0.0020, 0.022 * ux, y + 0.0020)
                .emit_fill(spine_col, target);
            let half_w = (text.len() as f32) * (0.0038 * ux * 4.0) * 0.5;
            stamp.emit_centered(text, 0.030 * ux + half_w, y, target);
        }
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let y0 = self.domain.map_y(low_meters);
        let y1 = self.domain.map_y(high_meters);
        if y1 > y0 + 1e-4 {
            ScreenBounds::new(-0.013 * ux, y0, -0.0042 * ux, y1).emit_fill(tone.color(), target);
        }
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, tone: PhosphorTone, target: &mut dyn MeshTarget) {
        ChevronBadge::new(
            self.domain.map_y(meters),
            ClosureKinematics::new(meters, range_rate),
            tone,
            self.inv_aspect,
        )
        .emit(target);
    }
}
