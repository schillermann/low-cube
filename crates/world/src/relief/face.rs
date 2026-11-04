use std::sync::RwLock;
use glam::Vec3;
use crate::block::{BlockMedium, BlockMold, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use crate::noise::{CoherentNoise, DomainWarp, FractalHarmonics};
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget, Quad};
use super::elevation::{ReliefElevation, ZonedRelief};
use super::palette::ReliefPalette;
use super::sight_ray::SurfaceSightRay;
use super::strata::ReliefStrata;
use super::terraced::TerracedRelief;

struct CellQuadTarget<'a>(&'a mut Vec<Quad>);

impl MeshTarget for CellQuadTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.0.push(quad);
    }
}

struct ReliefDomain {
    frame: SurfaceFrame,
    relief: Box<dyn ReliefElevation>,
}

struct ReliefCellCache {
    strata: ReliefStrata,
    divisions: u32,
    cached_cells: RwLock<Vec<Vec<Quad>>>,
}

/// Planetary relief face generating undulating climbable terrain and subterranean cavities ("Tell, Don't Ask").
pub struct FacetedReliefFace {
    domain: ReliefDomain,
    palette: ReliefPalette,
    cache: ReliefCellCache,
}

impl FacetedReliefFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        frame: SurfaceFrame,
        relief: Box<dyn ReliefElevation>,
        palette: ReliefPalette,
        strata: ReliefStrata,
        divisions: u32,
        cached_cells: RwLock<Vec<Vec<Quad>>>,
    ) -> Self {
        Self {
            domain: ReliefDomain { frame, relief },
            palette,
            cache: ReliefCellCache { strata, divisions, cached_cells },
        }
    }

    /// Secondary constructor assembling a relief face calibrated for a specific planetary climate zone.
    pub fn in_zone(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        let frame = zone.surface_frame(half_size);
        let palette = ReliefPalette::for_zone(zone);
        let strata = ReliefStrata::new();
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
        let base_noise = Box::new(CoherentNoise::new(seed_offset, freq));
        let harmonics = Box::new(FractalHarmonics::new(base_noise, 3, 0.5, 2.0));
        let warp_x = Box::new(CoherentNoise::new(seed_offset + 10, freq * 0.5));
        let warp_z = Box::new(CoherentNoise::new(seed_offset + 20, freq * 0.5));
        let warped = Box::new(DomainWarp::new(harmonics, warp_x, warp_z, 2.5));
        let max_elev = if half_size >= 48.0 {
            max_large
        } else if half_size >= 24.0 {
            max_med
        } else if matches!(zone, ClimateZone::Volcanic | ClimateZone::Arctic) {
            3
        } else {
            2
        };
        let zoned = Box::new(ZonedRelief::new(warped, frame.normal(), frame.tangent(), frame.bitangent(), half_size, min_elev, max_elev));
        let relief = Box::new(TerracedRelief::in_span(zoned, (divisions as i32) / 2));
        Self::new(frame, relief, palette, strata, divisions, RwLock::new(Vec::new()))
    }

    /// Secondary constructor assembling a relief face calibrated for Tron wireframe visualization.
    pub fn tron(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        let mut face = Self::in_zone(zone, half_size, divisions);
        face.palette = ReliefPalette::tron();
        face
    }

    /// Returns this relief face with an additional molded voxel shape stacked at `(u, v, base)`.
    pub fn with_molded_block(mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>) -> Self {
        self.domain.relief.stack_molded_cell(u, v, base, shape);
        self
    }

    fn build_cell(&self, u: i32, v: i32, half: i32) -> Vec<Quad> {
        let mut cell_quads = Vec::with_capacity(8);
        let mut target = CellQuadTarget(&mut cell_quads);
        self.cache.strata.emit_cell(
            GridCell::new(u, v),
            &self.domain.frame,
            self.domain.relief.as_ref(),
            &self.palette,
            half,
            &mut target,
        );
        cell_quads
    }

    fn refresh_neighborhood(&mut self, cu: i32, cv: i32, half: i32) {
        let side = self.cache.divisions as usize;
        let total = side * side;
        let populated = self.cache.cached_cells.get_mut().expect("Relief cell cache poisoned").len() == total;
        if populated {
            for v in (cv - 1).max(-half)..=(cv + 1).min(half - 1) {
                for u in (cu - 1).max(-half)..=(cu + 1).min(half - 1) {
                    let quads = self.build_cell(u, v, half);
                    let slot = ((v + half) as usize) * side + ((u + half) as usize);
                    self.cache.cached_cells.get_mut().expect("Relief cell cache poisoned")[slot] = quads;
                }
            }
        }
    }
}

impl Face for FacetedReliefFace {
    fn matches(&self, point: Vec3) -> bool {
        let height = self.domain.frame.height_of(point);
        let (u, v) = self.domain.frame.decompose(point);
        height > 0.0 && height >= u.abs() && height >= v.abs()
    }

    fn surface_up(&self) -> Vec3 { self.domain.frame.normal() }
    fn pull_direction(&self) -> Vec3 { -self.domain.frame.normal() }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let (u, v) = self.domain.frame.decompose(point);
        let surface_h = self.domain.relief.support_height_at(u, v, self.domain.frame.elevation_of(point) - distance, distance);
        self.domain.frame.point_at(u, v, surface_h + distance)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let half = (self.cache.divisions as i32) / 2;
        let side = self.cache.divisions as usize;
        let total = side * side;
        if let Ok(cells) = self.cache.cached_cells.read() {
            if cells.len() == total {
                for cell_quads in cells.iter() {
                    for &quad in cell_quads {
                        target.push_quad(quad);
                    }
                }
                return;
            }
        }
        let mut cells = self.cache.cached_cells.write().expect("Relief cell cache poisoned");
        if cells.len() != total {
            cells.clear();
            cells.reserve(total);
            for v in -half..half {
                for u in -half..half {
                    cells.push(self.build_cell(u, v, half));
                }
            }
        }
        for cell_quads in cells.iter() {
            for &quad in cell_quads {
                target.push_quad(quad);
            }
        }
    }

    fn harvest(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let half = (self.cache.divisions as i32) / 2;
        let half_size = self.domain.frame.step_elevation(half);
        if let Some((cu, cv, layer)) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.domain.frame, self.domain.relief.as_ref(), half, half_size)
        {
            let ((prev_u, prev_v), harvested) = self.domain.relief.harvest_layer(cu, cv, layer);
            if harvested.is_some() {
                if (prev_u, prev_v) != (cu, cv) {
                    self.refresh_neighborhood(prev_u, prev_v, half);
                }
                self.refresh_neighborhood(cu, cv, half);
            }
            return harvested;
        }
        None
    }

    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let half = (self.cache.divisions as i32) / 2;
        if stage == 0 {
            let ((prev_u, prev_v), changed) = self.domain.relief.clear_dig_stage();
            if changed {
                self.refresh_neighborhood(prev_u, prev_v, half);
            }
            return changed.then_some(BlockMedium::TurfEarth);
        }
        let half_size = self.domain.frame.step_elevation(half);
        if let Some((cu, cv, layer)) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.domain.frame, self.domain.relief.as_ref(), half, half_size)
        {
            let ((prev_u, prev_v), loosened) = self.domain.relief.mark_dig_layer(cu, cv, layer, stage);
            if loosened.is_some() {
                if (prev_u, prev_v) != (cu, cv) {
                    self.refresh_neighborhood(prev_u, prev_v, half);
                }
                self.refresh_neighborhood(cu, cv, half);
            }
            return loosened;
        }
        None
    }

    fn place_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.place_molded_block(eye, direction, avatar_pos, BlockMold::cube())
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let half = (self.cache.divisions as i32) / 2;
        let half_size = self.domain.frame.step_elevation(half);
        if let Some((cu, cv, base)) = SurfaceSightRay::new(eye, direction)
            .placement_cell(&self.domain.frame, self.domain.relief.as_ref(), half, half_size, avatar_pos)
        {
            let changed = self.domain.relief.stack_molded_cell(
                cu,
                cv,
                base,
                mold.shape_for(direction, &self.domain.frame),
            );
            if changed {
                self.refresh_neighborhood(cu, cv, half);
            }
            return changed;
        }
        false
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let (half, cells) = ((self.cache.divisions as i32) / 2, self.domain.relief.step_grass(dt));
        for &(cu, cv) in &cells {
            self.refresh_neighborhood(cu, cv, half);
        }
        !cells.is_empty()
    }

    fn weightless_at(&self, point: Vec3) -> bool {
        let (u, v) = self.domain.frame.decompose(point);
        self.domain.relief.weightless_at(u, v, self.domain.frame.elevation_of(point))
    }

    fn lethal_at(&self, point: Vec3) -> bool {
        let (u, v) = self.domain.frame.decompose(point);
        self.domain.relief.lethal_at(u, v, self.domain.frame.elevation_of(point))
    }

    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 {
        let ((u, v), elev) = (self.domain.frame.decompose(point), self.domain.frame.elevation_of(point));
        let (cu, cv, z) = self.domain.relief.confine(u, v, elev);
        let (grid_u, grid_v) = GridCell::at_planar(cu, cv).feed_coordinates(|gu, gv| (gu, gv));
        self.domain.frame.confine_radius(
            self.domain.frame.point_at(cu, cv, z),
            self.domain.relief.ceiling_inset(grid_u, grid_v, 0.30),
        )
    }
}
