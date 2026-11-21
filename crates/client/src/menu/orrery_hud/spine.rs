use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::menu::glyph::PixelStamp;
use crate::menu::widget::ScreenBounds;

const SPINE_X: f32 = 0.115;
const SPINE_BOT: f32 = -0.17;
const SPINE_TOP: f32 = 0.23;

/// Focused vertical laser-scale contract supporting Yegor-style decorator composition without getters.
pub trait RangeScale {
    fn emit_scale(&self, target: &mut dyn MeshTarget);
    fn project_band(&self, low_meters: f32, high_meters: f32, color: Vec3, target: &mut dyn MeshTarget);
    fn project_chevron(&self, meters: f32, range_rate: f32, accent: Vec3, target: &mut dyn MeshTarget);
}

/// Polymorphic domain calibration between Nose-Camera cosmic target range and Belly-Camera lunar pad clearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleDomain {
    CosmicTarget,
    LunarPad,
}

impl ScaleDomain {
    const fn span_meters(self) -> f32 {
        match self {
            Self::CosmicTarget => 1_000.0,
            Self::LunarPad => 100.0,
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
}

/// Immutable metric distance and signed range-rate formatter avoiding procedural helper functions.
#[derive(Clone, Copy, Debug)]
struct MetricReadout {
    meters: f32,
    range_rate: f32,
}

impl MetricReadout {
    const fn new(meters: f32, range_rate: f32) -> Self {
        Self { meters, range_rate }
    }

    fn distance_label(self) -> String {
        let d = self.meters.max(0.0);
        if d >= 1_000.0 {
            format!("{:.2} KM", d * 0.001)
        } else if d < 20.0 {
            format!("{:.1} M", d)
        } else {
            format!("{:.0} M", d)
        }
    }

    fn rate_label(self) -> String {
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
}

/// Core graduated vertical laser spine right of the reticle crosshair implementing `RangeScale`.
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

    pub const fn nose_target(inv_aspect: f32) -> Self {
        Self::new(ScaleDomain::CosmicTarget, inv_aspect)
    }

    pub const fn belly_altitude(inv_aspect: f32) -> Self {
        Self::new(ScaleDomain::LunarPad, inv_aspect)
    }

    fn map_y(self, meters: f32) -> f32 {
        let u = (meters.max(0.0) / self.domain.span_meters()).clamp(0.0, 1.0).sqrt();
        SPINE_BOT + (SPINE_TOP - SPINE_BOT) * u
    }
}

impl RangeScale for GraduatedSpine {
    fn emit_scale(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let sx = SPINE_X * ux;
        let (bx0, bx1) = (sx - 0.020 * ux, sx + 0.128 * ux);
        let (by0, by1) = (SPINE_BOT - 0.020, SPINE_TOP + 0.052);
        target.push_quad(Quad::glowing(
            Vec3::new(bx0, by0, 0.0),
            Vec3::new(bx1, by0, 0.0),
            Vec3::new(bx1, by1, 0.0),
            Vec3::new(bx0, by1, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(0.02, 0.06, 0.11),
            -0.44,
        ));
        let spine_col = Vec3::new(0.30, 0.66, 0.82);
        let minor_col = Vec3::new(0.22, 0.48, 0.62);
        let label_col = Vec3::new(0.58, 0.82, 0.92);
        ScreenBounds::new(sx - 0.0022 * ux, SPINE_BOT, sx + 0.0022 * ux, SPINE_TOP)
            .emit_fill(spine_col, target);
        PixelStamp::new(0.0044 * ux, 0.0062, Vec3::new(0.44, 0.94, 0.98))
            .emit_centered(self.domain.caption(), sx + 0.036 * ux, SPINE_TOP + 0.030, target);

        for step in 1..16 {
            let y = SPINE_BOT + (SPINE_TOP - SPINE_BOT) * ((step as f32) / 16.0);
            ScreenBounds::new(sx + 0.0022 * ux, y - 0.0012, sx + 0.011 * ux, y + 0.0012)
                .emit_fill(minor_col, target);
        }
        let stamp = PixelStamp::new(0.0038 * ux, 0.0052, label_col);
        for (station_m, text) in self.domain.stations() {
            let y = self.map_y(station_m);
            ScreenBounds::new(sx + 0.0022 * ux, y - 0.0020, sx + 0.022 * ux, y + 0.0020)
                .emit_fill(spine_col, target);
            let half_w = (text.len() as f32) * (0.0038 * ux * 4.0) * 0.5;
            stamp.emit_centered(text, sx + 0.030 * ux + half_w, y, target);
        }
    }

    fn project_band(&self, low_meters: f32, high_meters: f32, color: Vec3, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let sx = SPINE_X * ux;
        let y0 = self.map_y(low_meters);
        let y1 = self.map_y(high_meters);
        if y1 > y0 + 1e-4 {
            ScreenBounds::new(sx - 0.013 * ux, y0, sx - 0.0042 * ux, y1).emit_fill(color, target);
        }
    }

    fn project_chevron(&self, meters: f32, range_rate: f32, accent: Vec3, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let sx = SPINE_X * ux;
        let y_cur = self.map_y(meters);
        if y_cur > SPINE_BOT + 1e-4 {
            ScreenBounds::new(sx - 0.0024 * ux, SPINE_BOT, sx + 0.0024 * ux, y_cur)
                .emit_fill(accent, target);
        }
        target.push_quad(Quad::new(
            Vec3::new(sx + 0.005 * ux, y_cur, 0.0),
            Vec3::new(sx + 0.023 * ux, y_cur - 0.013, 0.0),
            Vec3::new(sx + 0.023 * ux, y_cur + 0.013, 0.0),
            Vec3::new(sx + 0.023 * ux, y_cur + 0.013, 0.0),
            Vec3::Z,
            accent,
        ));
        let (x0, x1) = (sx + 0.023 * ux, sx + 0.205 * ux);
        let (y0, y1) = (y_cur - 0.029, y_cur + 0.029);
        let badge = ScreenBounds::new(x0, y0, x1, y1);
        badge.emit_fill(Vec3::new(0.04, 0.10, 0.16), target);
        badge.emit_outline(accent, 0.0032 * ux, 0.0032, target);

        let metric = MetricReadout::new(meters, range_rate);
        let cx = badge.center_x();
        let bright = (accent * 0.45 + Vec3::splat(0.58)).clamp(Vec3::ZERO, Vec3::ONE);
        PixelStamp::new(0.0048 * ux, 0.0068, bright)
            .emit_centered(&metric.distance_label(), cx, y_cur + 0.010, target);
        PixelStamp::new(0.0035 * ux, 0.0048, accent)
            .emit_centered(&metric.rate_label(), cx, y_cur - 0.012, target);
    }
}
