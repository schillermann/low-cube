use glam::Vec3;
use crate::block::{BlockMedium, BlockMold, MiningEdge};
use crate::{Face, MeshTarget};
use super::super::craft_entity::CraftBlueprint;
use super::super::mineral_vein::ExposedStratum;
use super::super::planned_mount::PlannedMount;
use super::super::vehicle_assembly::VehicleAssembly;
use super::super::VesselSink;
use super::FacetedReliefFace;

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

    fn terrain_patch_count(&self) -> usize {
        self.cache.sector_count()
    }

    fn emit_terrain_patch(&self, patch_slot: usize, target: &mut dyn MeshTarget) {
        self.cache.emit_sector(patch_slot, &self.domain.frame, self.domain.relief.as_ref(), &self.palette, target);
    }

    fn pending_terrain_patches(&self) -> bool {
        self.cache.pending_sectors()
    }

    fn drain_terrain_patch(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        self.cache.drain_dirty_sector(&self.domain.frame, self.domain.relief.as_ref(), &self.palette, target)
    }

    fn meadow_center_sector(&self, center: Vec3) -> (i32, i32) {
        self.cache.center_sector(center, &self.domain.frame)
    }

    fn meadow_sector_revision(&self, gsu: i32, gsv: i32) -> u32 {
        self.cache.sector_revision(gsu, gsv)
    }

    fn emit_meadow_sector(&self, gsu: i32, gsv: i32, lod: u8, target: &mut dyn MeshTarget) {
        self.cache.emit_meadow_sector(gsu, gsv, lod, &self.domain.frame, self.domain.relief.as_ref(), &self.palette, target);
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.cache.emit_meadow_around(center, view_dir, &self.domain.frame, self.domain.relief.as_ref(), &self.palette, target);
    }

    fn harvest(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let half = self.cache.half_blocks();
        let (cu, cv, harvested) = self.domain.harvest(half, eye, direction)?;
        self.refresh_neighborhood(cu, cv, half, 4);
        Some(harvested)
    }

    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let half = self.cache.half_blocks();
        let (cu, cv, loosened) = self.domain.loosen(half, eye, direction, stage)?;
        self.refresh_neighborhood(cu, cv, half, 0);
        Some(loosened)
    }

    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> Option<bool> {
        self.domain.surface_yields_to(self.cache.half_blocks(), eye, direction, edge)
    }

    fn aimed_surface_medium(&self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.domain.aimed_surface_medium(self.cache.half_blocks(), eye, direction)
    }

    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        self.domain.plan_molded(self.cache.half_blocks(), eye, direction, avatar_pos, mold)
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let half = self.cache.half_blocks();
        let Some((cu, cv)) = self.domain.place_molded(half, eye, direction, avatar_pos, mold) else {
            return false;
        };
        self.refresh_neighborhood(cu, cv, half, 4);
        true
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        let half = self.cache.half_blocks();
        let Some((ru, rv)) = self.domain.adjust_rig(half, eye, direction, delta_steps, rotate_quarter) else {
            return false;
        };
        self.refresh_neighborhood(ru, rv, half, 4);
        true
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.domain.relief.placed_camera_tripod(&self.domain.frame)
    }

    fn collect_medium_sites(&self, medium: BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        self.domain.relief.collect_medium_sites(medium, &self.domain.frame, out);
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let half = self.cache.half_blocks();
        let Some((cu, cv)) = self.domain.lower_rig(half, eye, direction) else {
            return false;
        };
        self.refresh_neighborhood(cu, cv, half, 4);
        true
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, engage_engine: bool) -> Option<VehicleAssembly> {
        let half = self.cache.half_blocks();
        let (assembly, cells, terrain_changed) = self.domain.board_vehicle(half, eye, direction, engage_engine)?;
        if terrain_changed {
            for (cu, cv) in cells {
                self.refresh_neighborhood(cu, cv, half, 4);
            }
        }
        Some(assembly)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        let _ = self.domain.relief.park_vehicle_at(&self.domain.frame, assembly);
        true
    }

    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        self.domain.capture_blueprint(self.cache.half_blocks(), eye, direction)
    }

    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.domain.aimed_empty_rig(self.cache.half_blocks(), eye, direction, avatar_pos)
    }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        let half = self.cache.half_blocks();
        let Some((mu, mv)) = self.domain.build_blueprint(half, eye, direction, blueprint) else {
            return false;
        };
        self.refresh_neighborhood(mu, mv, half, 8);
        true
    }

    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        self.domain.relief.present_vessels(sink);
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let (half, cells) = (self.cache.half_blocks(), self.domain.step_grass(dt));
        for &(cu, cv) in &cells {
            self.refresh_neighborhood(cu, cv, half, 0);
        }
        !cells.is_empty()
    }

    fn active_simulation(&self) -> bool {
        self.domain.awake
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

    fn capture_sector_snapshot(&self, face_idx: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        let (bare_mask, cols) = self.domain.relief.capture_sector_state(gsu, gsv);
        crate::SectorSnapshot::new(face_idx, (gsu, gsv), bare_mask, cols)
    }

    fn apply_sector_snapshot(&mut self, snapshot: &crate::SectorSnapshot) -> bool {
        let (gsu, gsv) = snapshot.sector();
        self.domain.relief.apply_sector_state(gsu, gsv, snapshot.bare_mask(), snapshot.columns());
        self.domain.wake();
        let half = self.cache.half_blocks();
        self.refresh_neighborhood(gsu * 8 + 4, gsv * 8 + 4, half, 7);
        true
    }

    fn collect_modified_sectors(&self, out: &mut Vec<(i32, i32)>) {
        self.domain.relief.collect_modified_sectors(out);
    }
}
