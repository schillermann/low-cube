use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::moisture::{ConvergenceCell, WeatherCurve, WeatherSample};

///Focused contract mapping a troposphere layer index to altitude and vertical span.
pub trait StratumBand: Copy {
    fn elevation_band(&self) -> (f32, f32);
    fn footprint_ratio(&self) -> f32;
}

/// Calibrated altitude stratum (`0` = Cumulus/Nimbostratus, `1` = Altostratus, `2` = Cirrus; `<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CloudStratum {
    layer: u8,
    half_size: f32,
}

impl CloudStratum {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(layer: u8, half_size: f32) -> Self {
        Self { layer, half_size }
    }

    fn metric_scale(self) -> f32 {
        (self.half_size / 64.0).clamp(0.32, 6.0)
    }
}

impl StratumBand for CloudStratum {
    fn elevation_band(&self) -> (f32, f32) {
        let s = self.metric_scale();
        match self.layer {
            0 => (13.5 * s, 4.8 * s),
            1 => (21.5 * s, 2.5 * s),
            _ => (27.2 * s, 0.9 * s),
        }
    }

    fn footprint_ratio(&self) -> f32 {
        match self.layer {
            0 => 0.44,
            1 => 0.52,
            _ => 0.62,
        }
    }
}

/// Sculpted 3D stepped-voxel cloud volume with Beer-Lambert storm darkening (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CloudVolume {
    planar: (f32, f32, f32, f32),
    heights: (f32, f32),
    sample: ConvergenceCell,
}

impl CloudVolume {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        planar: (f32, f32, f32, f32),
        heights: (f32, f32),
        sample: ConvergenceCell,
    ) -> Self {
        Self { planar, heights, sample }
    }

    /// Constructs a 3D `CloudVolume` centered at `(cu, cv)` within `stratum`.
    pub fn from_cell(
        cu: f32,
        cv: f32,
        cell_span: f32,
        stratum: CloudStratum,
        sample: ConvergenceCell,
    ) -> Self {
        let (base_h, max_thick) = stratum.elevation_band();
        let (mass, dark, _, _) = sample.transfer(|m, d, r, f| (m, d, r, f));
        let half_w = cell_span * stratum.footprint_ratio() * (0.58 + 0.52 * mass);
        let thick = max_thick * (0.35 + 0.65 * mass + 0.25 * dark);
        Self::new(
            (cu - half_w, cu + half_w, cv - half_w * 0.88, cv + half_w * 0.88),
            (base_h, base_h + thick),
            sample,
        )
    }

    /// Evaluates `(fog_density, darkness)` when an aircraft or walker enters this 3D cloud volume.
    pub fn immersion_at(self, u: f32, v: f32, elev: f32) -> (f32, f32) {
        if !self.sample.present() {
            return (0.0, 0.0);
        }
        let (u0, u1, v0, v1) = self.planar;
        let (h0, h1) = self.heights;
        let feather = 1.6;
        if u < u0 - feather || u > u1 + feather || v < v0 - feather || v > v1 + feather {
            return (0.0, 0.0);
        }
        if elev < h0 - feather || elev > h1 + feather {
            return (0.0, 0.0);
        }
        let wu = WeatherCurve::new(0.0, feather).sample((u - (u0 - feather)).min((u1 + feather) - u));
        let wv = WeatherCurve::new(0.0, feather).sample((v - (v0 - feather)).min((v1 + feather) - v));
        let wh = WeatherCurve::new(0.0, feather).sample((elev - (h0 - feather)).min((h1 + feather) - elev));
        let (mass, dark, _, _) = self.sample.transfer(|m, d, r, f| (m, d, r, f));
        let density = (wu * wv * wh * (0.45 + 0.50 * mass)).clamp(0.0, 0.88);
        (density, dark)
    }

    pub fn emit(self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        if !self.sample.present() {
            return;
        }
        let (mass, dark, _, _) = self.sample.transfer(|m, d, r, f| (m, d, r, f));
        let crown = Vec3::new(0.97, 0.98, 1.00).lerp(Vec3::new(0.36, 0.39, 0.46), dark);
        let flank = Vec3::new(0.90, 0.93, 0.98).lerp(Vec3::new(0.25, 0.28, 0.35), dark);
        let belly = Vec3::new(0.80, 0.85, 0.93).lerp(Vec3::new(0.16, 0.18, 0.24), dark);
        let alpha = (0.34 + 0.48 * mass + 0.12 * dark).clamp(0.28, 0.90);
        let glow = -4.0 - alpha;

        let (u0, u1, v0, v1) = self.planar;
        let (h0, h1) = self.heights;
        let split_h = if mass > 0.32 { h0 + (h1 - h0) * 0.58 } else { h1 };
        self.emit_translucent_slab(
            (u0, u1, v0, v1),
            (h0, split_h),
            (crown, flank, belly),
            glow,
            frame,
            target,
        );
        if mass > 0.32 {
            let inset_u = (u1 - u0) * 0.20;
            let inset_v = (v1 - v0) * 0.22;
            self.emit_translucent_slab(
                (u0 + inset_u, u1 - inset_u * 0.7, v0 + inset_v, v1 - inset_v * 0.8),
                (split_h, h1),
                (crown, crown.lerp(flank, 0.5), flank),
                glow,
                frame,
                target,
            );
        }
    }

    fn emit_translucent_slab(
        self,
        (u0, u1, v0, v1): (f32, f32, f32, f32),
        (h0, h1): (f32, f32),
        (top_c, wall_c, bot_c): (Vec3, Vec3, Vec3),
        glow: f32,
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    ) {
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        let [b0, b1, b2, b3] = [
            frame.point_at(u0, v0, h0),
            frame.point_at(u1, v0, h0),
            frame.point_at(u1, v1, h0),
            frame.point_at(u0, v1, h0),
        ];
        let [p0, p1, p2, p3] = [
            frame.point_at(u0, v0, h1),
            frame.point_at(u1, v0, h1),
            frame.point_at(u1, v1, h1),
            frame.point_at(u0, v1, h1),
        ];
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, top_c, glow));
        target.push_quad(Quad::glowing(b0, b3, b2, b1, -n, bot_c, glow));
        target.push_quad(Quad::glowing(b0, b1, p1, p0, -b, wall_c, glow));
        target.push_quad(Quad::glowing(p3, p2, b2, b3, b, wall_c, glow));
        target.push_quad(Quad::glowing(b3, b0, p0, p3, -t, wall_c, glow));
        target.push_quad(Quad::glowing(b1, b2, p2, p1, t, wall_c, glow));
    }
}

/// Distant 3D vertical Virga rain/snow curtains beneath saturated dark Nimbostratus clouds (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct RainShaft {
    center: (f32, f32),
    span_and_top: (f32, f32),
    sample: ConvergenceCell,
}

impl RainShaft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(center: (f32, f32), span_and_top: (f32, f32), sample: ConvergenceCell) -> Self {
        Self { center, span_and_top, sample }
    }

    pub fn emit(self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        if !self.sample.raining() {
            return;
        }
        let (_, _, rate, frozen) = self.sample.transfer(|m, d, r, f| (m, d, r, f));
        let (cu, cv) = self.center;
        let (half_w, top_h) = self.span_and_top;
        let bot_h = (top_h * 0.08).max(0.8);
        let col = if frozen {
            Vec3::new(0.82, 0.90, 0.98)
        } else {
            Vec3::new(0.46, 0.56, 0.70)
        };
        let glow = -6.0 - (0.12 + 0.30 * rate).clamp(0.10, 0.44);
        let (t, b) = (frame.tangent(), frame.bitangent());

        let [a0, a1, a2, a3] = [
            frame.point_at(cu - half_w, cv, bot_h),
            frame.point_at(cu + half_w, cv, bot_h),
            frame.point_at(cu + half_w * 0.92, cv, top_h),
            frame.point_at(cu - half_w * 0.92, cv, top_h),
        ];
        let [c0, c1, c2, c3] = [
            frame.point_at(cu, cv - half_w, bot_h),
            frame.point_at(cu, cv + half_w, bot_h),
            frame.point_at(cu, cv + half_w * 0.92, top_h),
            frame.point_at(cu, cv - half_w * 0.92, top_h),
        ];
        target.push_quad(Quad::glowing(a0, a1, a2, a3, b, col, glow));
        target.push_quad(Quad::glowing(c0, c1, c2, c3, t, col, glow));
    }
}
