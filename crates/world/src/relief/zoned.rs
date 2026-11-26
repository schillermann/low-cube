use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::noise::{CoherentNoise, DomainWarp, FractalHarmonics, SeededFrequency, SpatialFrequency};
use crate::topology::ClimateZone;
use crate::WorldSeed;
use super::corner::NaturalCrown;
use super::elevation::ReliefElevation;

/// Zoned planetary relief evaluating continuous spatial frequency into stepped voxel strata.
pub struct ZonedRelief {
    frequency: Box<dyn SpatialFrequency>,
    frame: SurfaceFrame,
    half_size: f32,
    bounds: (i32, i32),
}

impl ZonedRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        frequency: Box<dyn SpatialFrequency>,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        half_size: f32,
        min_elevation: i32,
        max_elevation: i32,
    ) -> Self {
        Self {
            frequency,
            frame: SurfaceFrame::new(tangent, bitangent, normal, half_size),
            half_size,
            bounds: (min_elevation, max_elevation),
        }
    }

    /// Secondary constructor calibrating coherent noise harmonics and domain warp for a climate zone.
    pub fn in_climate(zone: ClimateZone, half_size: f32) -> Self {
        Self::in_climate_seeded(zone, half_size, WorldSeed::origin())
    }

    /// Secondary constructor calibrating coherent noise harmonics for a climate zone and `WorldSeed`.
    pub fn in_climate_seeded(zone: ClimateZone, half_size: f32, seed: WorldSeed) -> Self {
        let frame = zone.surface_frame(half_size);
        let (seed_offset, max_large, max_med) = match zone {
            ClimateZone::Volcanic => (202, 16, 12),
            ClimateZone::Arctic => (404, 14, 10),
            ClimateZone::Steppe => (606, 10, 8),
            ClimateZone::Temperate => (101, 9, 7),
            ClimateZone::Jungle => (505, 8, 6),
            ClimateZone::Desert => (303, 7, 5),
        };
        let min_elev = if matches!(zone, ClimateZone::Jungle) { -1 } else { 0 };
        let freq = if half_size >= 48.0 { 0.025 } else { 0.04 };
        let harmonics = Box::new(FractalHarmonics::new(Box::new(CoherentNoise::new(seed_offset, freq)), 3, 0.5, 2.0));
        let warped = DomainWarp::new(
            harmonics,
            Box::new(CoherentNoise::new(seed_offset + 10, freq * 0.5)),
            Box::new(CoherentNoise::new(seed_offset + 20, freq * 0.5)),
            2.5,
        );
        let decorated = Box::new(SeededFrequency::seeded(warped, seed));
        let max_elev = if half_size >= 48.0 {
            max_large
        } else if half_size >= 24.0 {
            max_med
        } else if matches!(zone, ClimateZone::Volcanic | ClimateZone::Arctic) {
            3
        } else {
            2
        };
        Self::new(decorated, frame.normal(), frame.tangent(), frame.bitangent(), half_size, min_elev, max_elev)
    }

    fn sample_at(&self, u: f32, v: f32) -> f32 {
        self.frequency.sample(self.frame.point_at(u, v, 0.0))
    }
}

impl ReliefElevation for ZonedRelief {
    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        let wu = self.frame.step_elevation(u);
        let wv = self.frame.step_elevation(v);
        let r = wu.hypot(wv);
        let spawn_range = if self.half_size >= 48.0 {
            16.0
        } else if self.half_size >= 24.0 {
            10.0
        } else {
            4.0
        };
        let spawn_weight = ((r - 2.5) / spawn_range).clamp(0.0, 1.0);
        let spawn_smooth = spawn_weight * spawn_weight * (3.0 - 2.0 * spawn_weight);

        let edge_dist = (self.half_size - wu.abs()).min(self.half_size - wv.abs());
        let border_weight = ((edge_dist - 1.5) / 8.0).clamp(0.0, 1.0);
        let border_smooth = border_weight * border_weight * (3.0 - 2.0 * border_weight);

        let weight = spawn_smooth * border_smooth;
        if weight <= 0.0 {
            return 0;
        }

        let s = self.sample_at(wu, wv);
        let normalized = (s + 1.0) * 0.5;
        let (min_elev, max_elev) = self.bounds;
        let range = (max_elev - min_elev) as f32;
        let raw = (normalized * range * weight).round() as i32;
        raw.clamp(min_elev, max_elev)
    }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        let cell = GridCell::at_planar(u, v);
        let (cell_u, cell_v) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let (frac_u, frac_v) = cell.local_fraction(u, v);
        NaturalCrown::at(cell_u, cell_v, |gu, gv| self.elevation_at(gu, gv))
            .surface_height(self.elevation_at(cell_u, cell_v), frac_u, frac_v)
    }
}
