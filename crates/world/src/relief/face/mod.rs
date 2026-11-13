mod cache;

use glam::Vec3;
use crate::block::{BlockMedium, BlockMold, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget};
use super::elevation::{ReliefElevation, ZonedRelief};
use super::mineral_vein::{ExposedStratum, HollowVeins, LivingVeins, SubterraneanVeins};
use super::palette::ReliefPalette;
use super::sight_ray::SurfaceSightRay;
use super::strata::ReliefStrata;
use super::terraced::TerracedRelief;
use super::vehicle_assembly::VehicleAssembly;
use self::cache::ReliefCellCache;

struct ReliefDomain {
    frame: SurfaceFrame,
    relief: Box<dyn ReliefElevation>,
}

impl ReliefDomain {
    fn new(frame: SurfaceFrame, relief: Box<dyn ReliefElevation>) -> Self {
        Self { frame, relief }
    }
}

/// Planetary relief face generating undulating climbable terrain and subterranean cavities ("Tell, Don't Ask").
pub struct FacetedReliefFace {
    domain: ReliefDomain,
    palette: ReliefPalette,
    cache: ReliefCellCache,
}

impl FacetedReliefFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(domain: ReliefDomain, palette: ReliefPalette, cache: ReliefCellCache) -> Self {
        Self { domain, palette, cache }
    }

    /// Secondary constructor assembling a relief face with explicit subterranean vein authority.
    pub fn in_zone_with_veins(
        zone: ClimateZone,
        half_size: f32,
        divisions: u32,
        veins: Box<dyn SubterraneanVeins>,
    ) -> Self {
        let frame = zone.surface_frame(half_size);
        let zoned = Box::new(ZonedRelief::in_climate(zone, half_size));
        let relief = Box::new(TerracedRelief::with_veins(zoned, (divisions as i32) / 2, veins));
        Self::new(
            ReliefDomain::new(frame, relief),
            ReliefPalette::for_zone(zone),
            ReliefCellCache::empty(ReliefStrata::new(), divisions),
        )
    }

    /// Secondary constructor assembling an authoritative relief face with living mineral veins.
    pub fn in_zone(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        Self::in_zone_with_veins(zone, half_size, divisions, Box::new(LivingVeins::ambient(zone.index() as u64 + 1)))
    }

    /// Secondary constructor assembling a zero-knowledge client relief face without unexposed ores.
    pub fn hollow_in_zone(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        Self::in_zone_with_veins(zone, half_size, divisions, Box::new(HollowVeins::untouched()))
    }

    /// Secondary constructor assembling a relief face calibrated for Tron wireframe visualization.
    pub fn tron(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        let mut face = Self::in_zone(zone, half_size, divisions);
        face.palette = ReliefPalette::tron();
        face
    }

    /// Secondary constructor assembling a zero-knowledge Tron wireframe relief face.
    pub fn hollow_tron(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        let mut face = Self::hollow_in_zone(zone, half_size, divisions);
        face.palette = ReliefPalette::tron();
        face
    }

    /// Returns this relief face with an additional molded voxel shape stacked at `(u, v, base)`.
    pub fn with_molded_block(mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>) -> Self {
        self.domain.relief.stack_molded_cell(u, v, base, shape);
        self
    }

    fn refresh_neighborhood(&mut self, cu: i32, cv: i32, half: i32, radius: i32) {
        self.cache.refresh_neighborhood(cu, cv, half, radius, &self.domain.frame, self.domain.relief.as_ref(), &self.palette);
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
        self.cache.emit_surface(&self.domain.frame, self.domain.relief.as_ref(), &self.palette, target);
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.cache.emit_meadow_around(center, view_dir, &self.domain.frame, target);
    }

    fn harvest(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let half = self.cache.half_blocks();
        let half_size = self.domain.frame.step_elevation(half);
        if let Some((cu, cv, layer)) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.domain.frame, self.domain.relief.as_ref(), half, half_size)
        {
            let ((prev_u, prev_v), harvested) = self.domain.relief.harvest_layer(cu, cv, layer);
            if harvested.is_some() {
                if (prev_u, prev_v) != (cu, cv) {
                    self.refresh_neighborhood(prev_u, prev_v, half, 0);
                }
                self.refresh_neighborhood(cu, cv, half, 4);
            }
            return harvested;
        }
        None
    }

    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let half = self.cache.half_blocks();
        if stage == 0 {
            let ((prev_u, prev_v), changed) = self.domain.relief.clear_dig_stage();
            if changed {
                self.refresh_neighborhood(prev_u, prev_v, half, 0);
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
                    self.refresh_neighborhood(prev_u, prev_v, half, 0);
                }
                self.refresh_neighborhood(cu, cv, half, 0);
            }
            return loosened;
        }
        None
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let half = self.cache.half_blocks();
        let half_size = self.domain.frame.step_elevation(half);
        if let Some((cu, cv, base)) = SurfaceSightRay::new(eye, direction)
            .placement_cell(&self.domain.frame, self.domain.relief.as_ref(), half, half_size, avatar_pos)
        {
            let shape = mold.shape_for(direction, &self.domain.frame);
            let changed = self.domain.relief.stack_molded_cell(cu, cv, base, shape);
            if changed {
                self.refresh_neighborhood(cu, cv, half, 4);
            }
            return changed;
        }
        false
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        let half = self.cache.half_blocks();
        let half_size = self.domain.frame.step_elevation(half);
        let (cu, cv) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.domain.frame, self.domain.relief.as_ref(), half, half_size)
            .map_or((0, 0), |(cu, cv, _)| (cu, cv));
        let (changed, (ru, rv)) = self.domain.relief.adjust_rig_at(cu, cv, delta_steps, rotate_quarter);
        if changed {
            self.refresh_neighborhood(ru, rv, half, 4);
        }
        changed
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.domain.relief.placed_camera_tripod(&self.domain.frame)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let half = self.cache.half_blocks();
        let half_size = self.domain.frame.step_elevation(half);
        if let Some((cu, cv, _)) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.domain.frame, self.domain.relief.as_ref(), half, half_size)
        {
            if self.domain.relief.lower_rig_at(cu, cv) {
                self.refresh_neighborhood(cu, cv, half, 4);
                return true;
            }
        }
        false
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, engage_engine: bool) -> Option<VehicleAssembly> {
        let half = self.cache.half_blocks();
        let half_size = self.domain.frame.step_elevation(half);
        let ray = SurfaceSightRay::new(eye, direction);
        let ray_hit = ray.target_layer(&self.domain.frame, self.domain.relief.as_ref(), half, half_size);
        let (cu, cv, layer) = if engage_engine {
            ray_hit.unwrap_or_else(|| {
                let (eu, ev) = self.domain.frame.decompose(eye);
                GridCell::at_planar(eu, ev).feed_coordinates(|gu, gv| (gu, gv, -1000.0))
            })
        } else {
            ray_hit?
        };
        let assembly = self.domain.relief.board_vehicle_at(&self.domain.frame, cu, cv, layer, ray, engage_engine)?;
        if engage_engine {
            let (eu, ev) = self.domain.frame.cell_at(eye).feed_coordinates(|gu, gv| (gu, gv));
            let (su, sv) = self.domain.frame.cell_at(assembly.origin()).feed_coordinates(|gu, gv| (gu, gv));
            self.refresh_neighborhood(eu, ev, half, 4);
            self.refresh_neighborhood(cu, cv, half, 4);
            self.refresh_neighborhood(su, sv, half, 4);
        }
        Some(assembly)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        let half = self.cache.half_blocks();
        let (cu, cv) = self.domain.relief.park_vehicle_at(&self.domain.frame, assembly);
        self.refresh_neighborhood(cu, cv, half, 4);
        true
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let (half, cells) = (self.cache.half_blocks(), self.domain.relief.step_grass(dt));
        for &(cu, cv) in &cells {
            self.refresh_neighborhood(cu, cv, half, 0);
        }
        !cells.is_empty()
    }

    fn lethal_at(&self, point: Vec3) -> bool {
        let (u, v) = self.domain.frame.decompose(point);
        self.domain.relief.lethal_at(u, v, self.domain.frame.elevation_of(point))
    }

    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 {
        let ((u, v), elev) = (self.domain.frame.decompose(point), self.domain.frame.elevation_of(point));
        let (cu, cv, z) = self.domain.relief.confine(u, v, elev);
        self.domain.frame.point_at(cu, cv, z)
    }

    fn accept_disclosure(&mut self, stratum: ExposedStratum) {
        let half = self.cache.half_blocks();
        let (cu, cv) = stratum.transfer(|u, v, _, _| (u, v));
        self.domain.relief.accept_disclosure(stratum);
        self.refresh_neighborhood(cu, cv, half, 1);
    }

    fn drain_disclosures(&mut self, out: &mut Vec<ExposedStratum>) {
        self.domain.relief.drain_disclosures(out);
    }
}
