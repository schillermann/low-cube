mod moisture;
mod precipitation;
mod strata;
#[cfg(test)]
mod tests;

pub use self::moisture::{
    BiomeMoisture, ClimateHumidity, ConvergenceCell, MoistureField, PlanarNoise, SmoothScalarField,
    TroposphereField, WeatherSample,
};
pub use self::precipitation::{CloudVeil, LocalDownpour};
pub use self::strata::{CloudStratum, CloudVolume, StratumBand};

use glam::Vec3;
use crate::{ClimateZone, MeshTarget, SightCone};
use self::moisture::WeatherCurve;

/// Focused contract for multi-layer planetary cloud decks, shower precipitation, and aircraft immersion.
pub trait PlanetaryWeather: Copy {
    fn emit_around(&self, cone: SightCone, target: &mut dyn MeshTarget);
    fn immersion_at(&self, point: Vec3) -> (f32, f32);
    fn precipitation_at(&self, point: Vec3) -> (f32, bool);
    fn overcast_at(&self, point: Vec3) -> f32;
}

/// Deterministic 3-layer troposphere deck (`Cumulus`, `Altocumulus`, `Cirrus`; `<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct TroposphereDeck {
    half_size: f32,
    epoch_secs: f32,
}

impl TroposphereDeck {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(half_size: f32, epoch_secs: f32) -> Self {
        Self { half_size, epoch_secs }
    }

    fn cell_jitter(self, zone: ClimateZone, layer: u8, iu: i32, iv: i32) -> (f32, f32) {
        let _ = self.half_size;
        let base = ((zone.index() as u32) + 1)
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(u32::from(layer).wrapping_mul(0x85EB_CA6B))
            .wrapping_add((iu as u32).wrapping_mul(0xC2B2_AE35))
            .wrapping_add((iv as u32).wrapping_mul(0x27D4_EB2F));
        let mut h1 = base ^ (base >> 15);
        h1 = h1.wrapping_mul(0x7FEB_352D);
        let mut h2 = (base.wrapping_add(0x1656_67B1)) ^ (h1 >> 13);
        h2 = h2.wrapping_mul(0x846C_A68B);
        let ju = ((h1 >> 16) as f32 / 65_535.0 - 0.5) * 0.46;
        let jv = ((h2 >> 16) as f32 / 65_535.0 - 0.5) * 0.46;
        (ju, jv)
    }
}

impl PlanetaryWeather for TroposphereDeck {
    fn emit_around(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        if self.half_size < 4.0 {
            return;
        }
        let viewer = cone.observe_from(|eye| eye);
        let radial = viewer.length();
        if radial < self.half_size * 0.85 || radial > self.half_size * 8.0 {
            return;
        }
        let max_reach = (self.half_size * 3.4).clamp(96.0, 280.0);
        let viewer_dir = viewer.normalize_or_zero();
        let orbital = radial > self.half_size * 1.55;
        let face_edge = self.half_size * 0.86;

        for zone in ClimateZone::all() {
            let align = viewer_dir.dot(zone.normal());
            let zone_fade = if orbital {
                WeatherCurve::new(0.10, 0.38).sample(align)
            } else {
                WeatherCurve::new(0.46, 0.70).sample(align)
            };
            if zone_fade <= 0.01 {
                continue;
            }
            let frame = zone.surface_frame(self.half_size);
            let field = MoistureField::new(zone, self.half_size, self.epoch_secs);
            let span = field.cell_span();
            let max_idx = (face_edge / span).floor() as i32;
            let cam_iu = (viewer.dot(frame.tangent()) / span).round() as i32;
            let cam_iv = (viewer.dot(frame.bitangent()) / span).round() as i32;
            let iu_lo = (cam_iu - 4).max(-max_idx);
            let iu_hi = (cam_iu + 4).min(max_idx);
            let iv_lo = (cam_iv - 4).max(-max_idx);
            let iv_hi = (cam_iv + 4).min(max_idx);

            for layer in 0..3_u8 {
                let stratum = CloudStratum::new(layer, self.half_size);
                let (base_h, _) = stratum.elevation_band();
                for iu in iu_lo..=iu_hi {
                    for iv in iv_lo..=iv_hi {
                        let (ju, jv) = self.cell_jitter(zone, layer, iu, iv);
                        let cu = ((iu as f32) + ju) * span;
                        let cv = ((iv as f32) + jv) * span;
                        let spot = frame.point_at(cu, cv, base_h);
                        if !cone.reaches(spot, max_reach) {
                            continue;
                        }
                        let sample = field.sample_layer(layer, cu, cv);
                        if !sample.present() {
                            continue;
                        }
                        let dist_fade = 1.0
                            - WeatherCurve::new(max_reach * 0.65, max_reach)
                                .sample((spot - viewer).length());
                        let rim_fade = 1.0
                            - WeatherCurve::new(0.76, 1.02)
                                .sample(cu.abs().max(cv.abs()) / face_edge.max(1.0));
                        let fade = zone_fade * dist_fade * rim_fade;
                        CloudVolume::from_cell(cu, cv, span, stratum, sample)
                            .emit(viewer, fade, &frame, target);
                    }
                }
            }
        }

        let dom = ClimateZone::dominant(viewer);
        let dom_frame = dom.surface_frame(self.half_size);
        let imm = self.immersion_at(viewer);
        CloudVeil::new(viewer, imm).emit(dom_frame.normal(), target);
        let (rain, frozen) = self.precipitation_at(viewer);
        LocalDownpour::new(
            viewer,
            (dom_frame.tangent(), dom_frame.bitangent(), dom_frame.normal()),
            (rain, self.epoch_secs, frozen),
        )
        .emit(target);
    }

    fn immersion_at(&self, point: Vec3) -> (f32, f32) {
        if self.half_size < 4.0 {
            return (0.0, 0.0);
        }
        let zone = ClimateZone::dominant(point);
        let frame = zone.surface_frame(self.half_size);
        let (u, v) = frame.decompose(point);
        let elev = frame.elevation_of(point);
        let field = MoistureField::new(zone, self.half_size, self.epoch_secs);
        let span = field.cell_span();
        let max_idx = ((self.half_size * 0.86) / span).floor() as i32;
        let ciu = (u / span).round() as i32;
        let civ = (v / span).round() as i32;

        let (mut best_d, mut best_dark) = (0.0_f32, 0.0_f32);
        for layer in 0..3_u8 {
            let stratum = CloudStratum::new(layer, self.half_size);
            let (base_h, thick) = stratum.elevation_band();
            if elev < base_h - 2.0 || elev > base_h + thick * 1.4 + 2.0 {
                continue;
            }
            for iu in (ciu - 1).max(-max_idx)..=(ciu + 1).min(max_idx) {
                for iv in (civ - 1).max(-max_idx)..=(civ + 1).min(max_idx) {
                    let (ju, jv) = self.cell_jitter(zone, layer, iu, iv);
                    let (cu, cv) = (((iu as f32) + ju) * span, ((iv as f32) + jv) * span);
                    let sample = field.sample_layer(layer, cu, cv);
                    let (d, dark) =
                        CloudVolume::from_cell(cu, cv, span, stratum, sample).immersion_at(u, v, elev);
                    if d > best_d {
                        best_d = d;
                        best_dark = dark;
                    }
                }
            }
        }
        (best_d, best_dark)
    }

    fn precipitation_at(&self, point: Vec3) -> (f32, bool) {
        if self.half_size < 4.0 {
            return (0.0, false);
        }
        let zone = ClimateZone::dominant(point);
        let frame = zone.surface_frame(self.half_size);
        let elev = frame.elevation_of(point);
        let (low_base, low_thick) = CloudStratum::new(0, self.half_size).elevation_band();
        if elev < -1.0 || elev > low_base + low_thick {
            return (0.0, false);
        }
        let (u, v) = frame.decompose(point);
        let sample = MoistureField::new(zone, self.half_size, self.epoch_secs).sample_layer(0, u, v);
        sample.transfer(|_, _, rate, frozen| (rate, frozen))
    }

    fn overcast_at(&self, point: Vec3) -> f32 {
        if self.half_size < 4.0 {
            return 0.0;
        }
        let zone = ClimateZone::dominant(point);
        let frame = zone.surface_frame(self.half_size);
        let elev = frame.elevation_of(point);
        let (low_base, low_thick) = CloudStratum::new(0, self.half_size).elevation_band();
        if elev < -4.0 || elev > low_base + low_thick * 1.4 {
            return 0.0;
        }
        let below = 1.0 - WeatherCurve::new(low_base * 0.82, low_base + low_thick * 1.3).sample(elev);
        let (u, v) = frame.decompose(point);
        let field = MoistureField::new(zone, self.half_size, self.epoch_secs);
        let step = field.cell_span() * 0.60;
        let probe = |du: f32, dv: f32| {
            field.sample_layer(0, u + du, v + dv).transfer(|_, dark, rain, _| {
                let d_cov = WeatherCurve::new(0.34, 0.68).sample(dark) * 0.62;
                let r_cov = WeatherCurve::new(0.02, 0.30).sample(rain) * 0.72;
                (d_cov + r_cov).clamp(0.0, 1.0)
            })
        };
        let mean = probe(0.0, 0.0) * 0.44
            + (probe(step, 0.0) + probe(-step, 0.0) + probe(0.0, step) + probe(0.0, -step)) * 0.14;
        (mean * below).clamp(0.0, 1.0)
    }
}
