use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::relief::{
    FacetedReliefFace, HollowVeins, LivingVeins, NaturalCrown, ReliefElevation, SubterraneanVeins,
};
use crate::topology::ClimateZone;
use crate::{CubePlanet, Face, NullFace};

/// Deterministic impact-crater and mare-ridge elevation model for airless lunar cube faces.
pub struct LunarCraterRelief {
    frame: SurfaceFrame,
    half_size: f32,
    seed: u64,
}

impl LunarCraterRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(frame: SurfaceFrame, half_size: f32, seed: u64) -> Self {
        Self { frame, half_size, seed }
    }

    pub fn in_zone(zone: ClimateZone, half_size: f32) -> Self {
        Self::new(zone.surface_frame(half_size), half_size, (zone.index() as u64 + 1) * 0x9E37_79B9)
    }

    fn hash2(&self, gx: i32, gy: i32) -> (f32, f32, f32) {
        let mut h = self.seed
            ^ ((gx as i64 as u64).wrapping_mul(0x517C_C1B7_2722_0A95))
            ^ ((gy as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB));
        h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        let a = ((h & 0xFFFF) as f32) / 65535.0;
        let b = (((h >> 16) & 0xFFFF) as f32) / 65535.0;
        let c = (((h >> 32) & 0xFFFF) as f32) / 65535.0;
        (a, b, c)
    }

    fn crater_delta(&self, wu: f32, wv: f32) -> f32 {
        let spacing = 7.0;
        let (gx, gy) = ((wu / spacing).floor() as i32, (wv / spacing).floor() as i32);
        let mut sum = 0.0f32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (cx_i, cy_i) = (gx + dx, gy + dy);
                let ( ox, oy, r_seed) = self.hash2(cx_i, cy_i);
                let cx = ((cx_i as f32) + 0.2 + 0.6 * ox) * spacing;
                let cy = ((cy_i as f32) + 0.2 + 0.6 * oy) * spacing;
                let radius = 1.8 + 1.9 * r_seed;
                let d = (wu - cx).hypot(wv - cy) / radius;
                if d < 0.72 {
                    sum -= 2.1 * (1.0 - (d / 0.72).powi(2));
                } else if d < 1.12 {
                    let rim = 1.0 - ((d - 0.92).abs() / 0.20).clamp(0.0, 1.0);
                    sum += 1.25 * rim;
                }
            }
        }
        sum
    }
}

impl ReliefElevation for LunarCraterRelief {
    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        let wu = self.frame.step_elevation(u);
        let wv = self.frame.step_elevation(v);
        let edge_dist = (self.half_size - wu.abs()).min(self.half_size - wv.abs());
        let border = ((edge_dist - 1.5) / 3.5).clamp(0.0, 1.0);
        let smooth = border * border * (3.0 - 2.0 * border);
        if smooth <= 0.01 {
            return 0;
        }
        let ridge = ((wu * 0.22).sin() * (wv * 0.19).cos()) * 0.85;
        let raw = ((self.crater_delta(wu, wv) + ridge) * smooth).round() as i32;
        raw.clamp(-2, 2)
    }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        let cell = GridCell::at_planar(u, v);
        let (cu, cv) = cell.feed_coordinates(|gu, gv| (gu, gv));
        let (fu, fv) = cell.local_fraction(u, v);
        NaturalCrown::at(cu, cv, |gu, gv| self.elevation_at(gu, gv))
            .surface_height(self.elevation_at(cu, cv), fu, fv)
    }
}

/// Six-faced cratered lunar cube-world factory with bevel seams and rich subterranean ore veins.
pub struct LunarWorld;

impl LunarWorld {
    pub const BLOCKS: u32 = 32;
    pub const HALF_SIZE: f32 = 16.0;

    fn build(authoritative_veins: bool) -> CubePlanet {
        let divs = Self::BLOCKS * 2;
        let faces: Vec<Box<dyn Face>> = ClimateZone::all()
            .into_iter()
            .map(|zone| -> Box<dyn Face> {
                let craters = Box::new(LunarCraterRelief::in_zone(zone, Self::HALF_SIZE));
                let veins: Box<dyn SubterraneanVeins> = if authoritative_veins {
                    Box::new(LivingVeins::ambient(zone.index() as u64 + 77))
                } else {
                    Box::new(HollowVeins::untouched())
                };
                Box::new(FacetedReliefFace::lunar_with_veins(zone, Self::HALF_SIZE, divs, craters, veins))
            })
            .collect();
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), Self::HALF_SIZE, 1.0)
    }

    pub fn authoritative() -> CubePlanet {
        Self::build(true)
    }

    pub fn hollow() -> CubePlanet {
        Self::build(false)
    }
}
