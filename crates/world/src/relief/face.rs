use std::sync::RwLock;
use glam::Vec3;
use crate::block::{BlockMold, GridCell};
use crate::frame::SurfaceFrame;
use crate::noise::{CoherentNoise, DomainWarp, FractalHarmonics};
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget, Quad};
use super::cavity::{NullCavity, SubterraneanCavity};
use super::elevation::{ReliefElevation, ZonedRelief};
use super::palette::ReliefPalette;
use super::sight_ray::SurfaceSightRay;
use super::strata::ReliefStrata;
use super::terraced::TerracedRelief;

struct RowQuadTarget<'a>(&'a mut Vec<Quad>);

impl MeshTarget for RowQuadTarget<'_> {
    fn push_quad(&mut self, quad: Quad) { self.0.push(quad); }
}

/// Planetary relief face generating undulating climbable terrain and subterranean cavities ("Tell, Don't Ask").
pub struct FacetedReliefFace {
    frame: SurfaceFrame, relief: Box<dyn ReliefElevation>, cavity: Box<dyn SubterraneanCavity>,
    palette: ReliefPalette, strata: ReliefStrata, divisions: u32, cached_rows: RwLock<Vec<Vec<Quad>>>,
}

impl FacetedReliefFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        frame: SurfaceFrame, relief: Box<dyn ReliefElevation>, cavity: Box<dyn SubterraneanCavity>,
        palette: ReliefPalette, strata: ReliefStrata, divisions: u32, cached_rows: RwLock<Vec<Vec<Quad>>>,
    ) -> Self {
        Self { frame, relief, cavity, palette, strata, divisions, cached_rows }
    }

    /// Secondary constructor assembling a relief face calibrated for a specific planetary climate zone.
    pub fn in_zone(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        let frame = zone.surface_frame(half_size);
        let palette = ReliefPalette::for_zone(zone);
        let strata = ReliefStrata::new();

        let seed_offset = match zone {
            ClimateZone::Temperate => 101, ClimateZone::Volcanic => 202, ClimateZone::Desert => 303,
            ClimateZone::Arctic => 404, ClimateZone::Jungle => 505, ClimateZone::Steppe => 606,
        };
        let freq = if half_size >= 48.0 { 0.025 } else { 0.04 };
        let base_noise = Box::new(CoherentNoise::new(seed_offset, freq));
        let harmonics = Box::new(FractalHarmonics::new(base_noise, 3, 0.5, 2.0));
        let warp_x = Box::new(CoherentNoise::new(seed_offset + 10, freq * 0.5));
        let warp_z = Box::new(CoherentNoise::new(seed_offset + 20, freq * 0.5));
        let warped = Box::new(DomainWarp::new(harmonics, warp_x, warp_z, 2.5));

        let (min_elev, max_elev) = if half_size >= 48.0 {
            match zone {
                ClimateZone::Volcanic => (0, 16), ClimateZone::Arctic => (0, 14), ClimateZone::Steppe => (0, 10),
                ClimateZone::Temperate => (0, 9), ClimateZone::Jungle => (-1, 8), ClimateZone::Desert => (0, 7),
            }
        } else if half_size >= 24.0 {
            match zone {
                ClimateZone::Volcanic => (0, 12), ClimateZone::Arctic => (0, 10), ClimateZone::Steppe => (0, 8),
                ClimateZone::Temperate => (0, 7), ClimateZone::Jungle => (-1, 6), ClimateZone::Desert => (0, 5),
            }
        } else {
            match zone {
                ClimateZone::Volcanic | ClimateZone::Arctic => (0, 3),
                ClimateZone::Jungle => (-1, 2),
                _ => (0, 2),
            }
        };

        let zoned = Box::new(ZonedRelief::new(
            warped, frame.normal(), frame.tangent(), frame.bitangent(),
            half_size, min_elev, max_elev,
        ));
        let relief = Box::new(TerracedRelief::in_span(zoned, (divisions as i32) / 2));
        Self::new(frame, relief, Box::new(NullCavity), palette, strata, divisions, RwLock::new(Vec::new()))
    }

    /// Secondary constructor assembling a relief face calibrated for Tron wireframe visualization.
    pub fn tron(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        let mut face = Self::in_zone(zone, half_size, divisions);
        face.palette = ReliefPalette::tron();
        face
    }

    fn build_row(&self, v: i32, half: i32) -> Vec<Quad> {
        let mut row = Vec::with_capacity((self.divisions as usize) * 2);
        let mut target = RowQuadTarget(&mut row);
        for u in -half..half {
            self.strata.emit_cell(
                GridCell::new(u, v), &self.frame, self.relief.as_ref(),
                self.cavity.as_ref(), &self.palette, half, &mut target,
            );
        }
        row
    }

    fn refresh_rows(&mut self, cv: i32, half: i32) {
        let populated = self.cached_rows.get_mut().expect("Relief row cache poisoned").len() == self.divisions as usize;
        if populated {
            for v in (cv - 1).max(-half)..=(cv + 1).min(half - 1) {
                let row = self.build_row(v, half);
                self.cached_rows.get_mut().expect("Relief row cache poisoned")[(v + half) as usize] = row;
            }
        }
    }
}

impl Face for FacetedReliefFace {
    fn matches(&self, point: Vec3) -> bool {
        let height = point.dot(self.frame.normal());
        height > 0.0 && height >= point.dot(self.frame.tangent()).abs() && height >= point.dot(self.frame.bitangent()).abs()
    }

    fn surface_up(&self) -> Vec3 { self.frame.normal() }
    fn pull_direction(&self) -> Vec3 { -self.frame.normal() }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let (u, v) = self.frame.decompose(point);
        let surface_h = self.relief.surface_height_at(u, v);
        let current_h = self.frame.height_of(point);
        let depth_h = self.cavity.constrain_depth(u, v, current_h, surface_h);
        self.frame.point_at(u, v, depth_h + distance)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let half = (self.divisions as i32) / 2;
        if let Ok(rows) = self.cached_rows.read() {
            if rows.len() == self.divisions as usize {
                for row in rows.iter() {
                    for &quad in row { target.push_quad(quad); }
                }
                return;
            }
        }
        let mut rows = self.cached_rows.write().expect("Relief row cache poisoned");
        if rows.len() != self.divisions as usize {
            rows.clear();
            rows.reserve(self.divisions as usize);
            for v in -half..half { rows.push(self.build_row(v, half)); }
        }
        for row in rows.iter() {
            for &quad in row { target.push_quad(quad); }
        }
    }

    fn excavate(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let half = (self.divisions as i32) / 2;
        if let Some((cu, cv, layer)) = SurfaceSightRay::new(eye, direction).target_layer(&self.frame, self.relief.as_ref(), half, (half as f32) * 0.5) {
            let (prev_v, changed) = self.relief.excavate_layer(cu, cv, layer);
            if changed {
                if prev_v != cv { self.refresh_rows(prev_v, half); }
                self.refresh_rows(cv, half);
            }
            return changed;
        }
        false
    }

    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> bool {
        let half = (self.divisions as i32) / 2;
        if stage == 0 {
            let (prev_v, changed) = self.relief.mark_dig_stage(0, 0, 0);
            if changed { self.refresh_rows(prev_v, half); }
            return changed;
        }
        if let Some((cu, cv, layer)) = SurfaceSightRay::new(eye, direction).target_layer(&self.frame, self.relief.as_ref(), half, (half as f32) * 0.5) {
            let (prev_v, changed) = self.relief.mark_dig_layer(cu, cv, layer, stage);
            if changed {
                if prev_v != cv { self.refresh_rows(prev_v, half); }
                self.refresh_rows(cv, half);
            }
            return changed;
        }
        false
    }

    fn place_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.place_molded_block(eye, direction, avatar_pos, BlockMold::cube())
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let half = (self.divisions as i32) / 2;
        if let Some((cu, cv, base)) = SurfaceSightRay::new(eye, direction).placement_cell(&self.frame, self.relief.as_ref(), half, (half as f32) * 0.5, avatar_pos) {
            let shape = mold.shape_for(direction, &self.frame);
            let changed = self.relief.stack_molded_cell(cu, cv, base, shape, mold.full_height_flat());
            if changed { self.refresh_rows(cv, half); }
            return changed;
        }
        false
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let half = (self.divisions as i32) / 2;
        let rows = self.relief.step_grass(dt);
        for &cv in &rows { self.refresh_rows(cv, half); }
        !rows.is_empty()
    }
}
