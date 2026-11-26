use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::moisture::{ConvergenceCell, WeatherCurve, WeatherSample};

/// Focused contract mapping a troposphere layer index to altitude and vertical span.
pub trait StratumBand: Copy {
    fn elevation_band(&self) -> (f32, f32);
    fn footprint_ratio(&self) -> f32;
}

/// Calibrated high-sky stratum (`0` = Cumulus, `1` = Altocumulus, `2` = Cirrus; `<= 4` fields).
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

    pub(super) const fn layer_index(self) -> u8 {
        self.layer
    }

    pub(super) fn metric_scale(self) -> f32 {
        (self.half_size / 64.0).clamp(0.45, 6.0)
    }
}

impl StratumBand for CloudStratum {
    fn elevation_band(&self) -> (f32, f32) {
        let s = self.metric_scale();
        match self.layer {
            0 => (34.0 * s, 6.0 * s),
            1 => (48.0 * s, 3.2 * s),
            _ => (60.0 * s, 1.6 * s),
        }
    }

    fn footprint_ratio(&self) -> f32 {
        match self.layer {
            0 => 0.36,
            1 => 0.42,
            _ => 0.48,
        }
    }
}

/// Sculpted 3D multi-puff cloud cluster with Beer-Lambert storm shading (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CloudVolume {
    planar: (f32, f32, f32, f32),
    heights: (f32, f32),
    sample: ConvergenceCell,
    layer: u8,
}

impl CloudVolume {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        planar: (f32, f32, f32, f32),
        heights: (f32, f32),
        sample: ConvergenceCell,
        layer: u8,
    ) -> Self {
        Self { planar, heights, sample, layer }
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
        let (mass, dark, rain, _) = sample.transfer(|m, d, r, f| (m, d, r, f));
        let storm = if stratum.layer_index() == 0 {
            WeatherCurve::new(0.01, 0.28)
                .sample(rain)
                .max(WeatherCurve::new(0.36, 0.68).sample(dark))
        } else {
            0.0
        };
        let swell = 1.0 + 1.48 * storm;
        let half_w = cell_span * stratum.footprint_ratio() * (0.56 + 0.48 * mass) * swell;
        let low_h = base_h - 6.0 * stratum.metric_scale() * storm;
        let thick = max_thick * (0.38 + 0.62 * mass + 0.35 * storm);
        Self::new(
            (cu - half_w, cu + half_w, cv - half_w * 0.90, cv + half_w * 0.90),
            (low_h, low_h + thick),
            sample,
            stratum.layer_index(),
        )
    }

    /// Evaluates `(fog_density, darkness)` when an aircraft or climber enters this 3D cloud volume.
    pub fn immersion_at(self, u: f32, v: f32, elev: f32) -> (f32, f32) {
        if !self.sample.present() {
            return (0.0, 0.0);
        }
        let (u0, u1, v0, v1) = self.planar;
        let (h0, h1) = self.heights;
        let feather = 1.8;
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
        let density = (wu * wv * wh * (0.42 + 0.48 * mass)).clamp(0.0, 0.82);
        (density, dark)
    }

    fn puff_hash(self, slot: u32) -> (f32, f32) {
        let (u0, u1, v0, v1) = self.planar;
        let ku = (((u0 + u1) * 2.5) as i32) as u32;
        let kv = (((v0 + v1) * 2.5) as i32) as u32;
        let mut h = ku
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(kv.wrapping_mul(0x85EB_CA6B))
            .wrapping_add(u32::from(self.layer).wrapping_mul(0xC2B2_AE35))
            .wrapping_add(slot.wrapping_mul(0x27D4_EB2F));
        h ^= h >> 16;
        h = h.wrapping_mul(0x7FEB_352D);
        let a = ((h >> 16) as f32) / 65_535.0;
        let b = ((h & 0xFFFF) as f32) / 65_535.0;
        (a, b)
    }

    pub fn emit(self, viewer: Vec3, fade: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        if !self.sample.present() || fade <= 0.02 {
            return;
        }
        let (mass, dark, rain, _) = self.sample.transfer(|m, d, r, f| (m, d, r, f));
        let storm = WeatherCurve::new(0.01, 0.28)
            .sample(rain)
            .max(WeatherCurve::new(0.36, 0.68).sample(dark));
        let crown = Vec3::new(0.99, 0.99, 1.00).lerp(Vec3::new(0.34, 0.39, 0.48), dark);
        let flank = Vec3::new(0.93, 0.96, 1.00).lerp(Vec3::new(0.20, 0.24, 0.33), dark);
        let belly = Vec3::new(0.85, 0.90, 0.98).lerp(Vec3::new(0.12, 0.15, 0.22), dark);
        let fair_a = 0.28 + 0.24 * mass;
        let storm_a = 0.80 + 0.10 * storm;
        let base_a = match self.layer {
            0 => (fair_a + (storm_a - fair_a) * storm) * fade,
            1 => (0.16 + 0.18 * mass) * fade,
            _ => (0.10 + 0.14 * mass) * fade,
        };

        let (u0, u1, v0, v1) = self.planar;
        let (h0, h1) = self.heights;
        let (cu, cv) = ((u0 + u1) * 0.5, (v0 + v1) * 0.5);
        let (ru, rv, dh) = ((u1 - u0) * 0.5, (v1 - v0) * 0.5, h1 - h0);
        let up = frame.normal();
        let (t_ax, b_ax) = (frame.tangent(), frame.bitangent());

        if self.layer == 0 {
            let (j1, k1) = self.puff_hash(1);
            let off_u = (j1 - 0.5) * ru * 0.72;
            let off_v = (k1 - 0.5) * rv * 0.72;
            if storm > 0.18 {
                self.emit_puff(viewer, (cu, cv, h0 + dh * 0.18), (ru * 1.08, rv * 1.02), (-up, belly, base_a, 6), frame, target);
            }
            self.emit_puff(viewer, (cu, cv, h0 + dh * 0.30), (ru * 0.96, rv * 0.88), (-up * 0.55 + t_ax * 0.2, belly, base_a, 0), frame, target);
            self.emit_puff(viewer, (cu + off_u, cv + off_v, h0 + dh * 0.46), (ru * 0.80, rv * 0.76), (up * 0.35 + t_ax * 0.5, flank, base_a * 0.94, 1), frame, target);
            self.emit_puff(viewer, (cu - off_u * 0.85, cv - off_v * 0.80, h0 + dh * 0.50), (ru * 0.76, rv * 0.72), (up * 0.45 - b_ax * 0.4, flank.lerp(crown, 0.40), base_a * 0.92, 2), frame, target);
            if mass > 0.22 {
                self.emit_puff(viewer, (cu - off_v * 0.35, cv + off_u * 0.35, h0 + dh * 0.78), (ru * 0.74, rv * 0.72), (up, crown, base_a * 0.95, 3), frame, target);
            }
            return;
        }
        let stretch = if self.layer == 1 { 1.12 } else { 1.38 };
        let slim = if self.layer == 1 { 0.78 } else { 0.54 };
        let (j2, k2) = self.puff_hash(4);
        let du = (j2 - 0.5) * ru * 0.65;
        let dv = (k2 - 0.5) * rv * 0.55;
        self.emit_puff(viewer, (cu - du * 0.5, cv - dv * 0.5, h0 + dh * 0.42), (ru * stretch, rv * slim), (up * 0.8 + t_ax * 0.2, flank.lerp(crown, 0.5), base_a, 4), frame, target);
        self.emit_puff(viewer, (cu + du * 0.7, cv + dv * 0.7, h0 + dh * 0.66), (ru * stretch * 0.85, rv * slim * 0.88), (up, crown, base_a * 0.86, 5), frame, target);
    }

    fn emit_puff(
        self,
        viewer: Vec3,
        (pu, pv, ph): (f32, f32, f32),
        (ru, rv): (f32, f32),
        (shade_dir, col, alpha, slot): (Vec3, Vec3, f32, u32),
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    ) {
        let pos = frame.point_at(pu, pv, ph);
        let to_eye = viewer - pos;
        let dist = to_eye.length();
        let r_max = ru.max(rv).max(0.5);
        let near_fade = WeatherCurve::new(r_max * 0.32, r_max * 1.25).sample(dist);
        let a = (alpha * near_fade).clamp(0.0, 0.90);
        if a < 0.015 || dist < 0.05 {
            return;
        }
        let view_n = to_eye / dist;
        let anchor = if view_n.dot(-frame.normal()) > -0.95 {
            -frame.normal()
        } else {
            frame.normal()
        };
        let align_rot = Quat::from_rotation_arc(anchor, view_n);
        let right = (align_rot * frame.tangent()).normalize_or_zero();
        let up_ax = view_n.cross(right).normalize_or_zero();
        let (roll_u, seed_u) = self.puff_hash(slot + 8);
        let roll = roll_u * std::f32::consts::TAU;
        let (cr, sr) = (roll.cos(), roll.sin());
        let ax_u = (right * cr + up_ax * sr) * ru;
        let ax_v = (-right * sr + up_ax * cr) * rv;
        let encoded_n = shade_dir.normalize_or_zero() * (1.05 + 0.90 * seed_u);
        target.push_quad(Quad::glowing(
            pos - ax_u - ax_v,
            pos + ax_u - ax_v,
            pos + ax_u + ax_v,
            pos - ax_u + ax_v,
            encoded_n,
            col,
            -4.0 - a,
        ));
    }
}
