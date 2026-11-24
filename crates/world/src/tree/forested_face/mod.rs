mod cache;

use std::sync::RwLock;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{
    BlockMedium, BlockMold, CraftBlueprint, Face, MeshTarget, PlannedMount, Quad, VehicleAssembly,
    VesselSink,
};
use super::arboreal::Arboreal;
use super::sector_forest::SectorForest;
use super::timber_yield::{CleaveOutcome, SurfaceFind};
use self::cache::LitterLanding;

/// Composite decorator populating a planetary face lazily per local `4m × 4m` sector (`<= 4` fields).
pub struct ForestedFace {
    origin: Box<dyn Face>,
    frame: SurfaceFrame,
    forest: RwLock<SectorForest>,
}

impl ForestedFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_forest(
        origin: Box<dyn Face>,
        frame: SurfaceFrame,
        forest: RwLock<SectorForest>,
    ) -> Self {
        Self { origin, frame, forest }
    }

    /// Secondary constructor seeding custom specimens into sector `(0, 0)` for deterministic unit tests.
    pub fn new(
        origin: Box<dyn Face>,
        frame: SurfaceFrame,
        trees: Vec<Box<dyn Arboreal>>,
        _unused_cache: RwLock<Vec<Vec<Quad>>>,
    ) -> Self {
        let half = frame.point_at(0.0, 0.0, 0.0).abs().max_element().max(8.0);
        Self::with_forest(origin, frame, RwLock::new(SectorForest::with_specimens(half, trees)))
    }

    /// Secondary constructor initializing a lazy sector-partitioned woodland (`0` eager trees at startup).
    pub fn in_zone(origin: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let frame = zone.surface_frame(half_size);
        Self::with_forest(origin, frame, RwLock::new(SectorForest::lazy_zone(zone, half_size)))
    }

    pub fn materialized_grove_count(&self) -> usize {
        self.forest.read().expect("SectorForest lock poisoned").materialized_grove_count()
    }
}

impl Face for ForestedFace {
    fn matches(&self, point: Vec3) -> bool { self.origin.matches(point) }
    fn surface_up(&self) -> Vec3 { self.origin.surface_up() }
    fn pull_direction(&self) -> Vec3 { self.origin.pull_direction() }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let pos = self.origin.constrain(point, distance);
        let (u, v) = self.frame.decompose(point);
        self.forest
            .read()
            .expect("SectorForest lock poisoned")
            .constrain_near(pos, u, v, distance, self.origin.as_ref(), &self.frame)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_surface(target);
        self.emit_flora(target);
    }

    fn emit_terrain(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_terrain(target);
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.origin.emit_meadow_around(center, view_dir, target);
    }

    fn terrain_patch_count(&self) -> usize { self.origin.terrain_patch_count() }
    fn emit_terrain_patch(&self, patch_slot: usize, target: &mut dyn MeshTarget) { self.origin.emit_terrain_patch(patch_slot, target); }
    fn pending_terrain_patches(&self) -> bool { self.origin.pending_terrain_patches() }
    fn drain_terrain_patch(&mut self, target: &mut dyn MeshTarget) -> Option<usize> { self.origin.drain_terrain_patch(target) }
    fn meadow_center_sector(&self, center: Vec3) -> (i32, i32) { self.origin.meadow_center_sector(center) }

    fn meadow_sector_revision(&self, gsu: i32, gsv: i32) -> u32 {
        let base = self.origin.meadow_sector_revision(gsu, gsv);
        let flora = self.forest.read().expect("SectorForest lock poisoned").sector_revision(gsu, gsv);
        base.wrapping_add(flora.wrapping_mul(65_537))
    }

    fn emit_meadow_sector(&self, gsu: i32, gsv: i32, lod: u8, target: &mut dyn MeshTarget) {
        self.origin.emit_meadow_sector(gsu, gsv, lod, target);
        let mut forest = self.forest.write().expect("SectorForest lock poisoned");
        if forest.continental() {
            forest.emit_sector((gsu, gsv), self.origin.as_ref(), self.frame, target);
        }
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.forest
            .write()
            .expect("SectorForest lock poisoned")
            .perch_near(center, selector, self.origin.as_ref(), &self.frame)
    }

    fn emit_flora(&self, target: &mut dyn MeshTarget) {
        self.forest
            .write()
            .expect("SectorForest lock poisoned")
            .emit_compact_face(self.origin.as_ref(), self.frame, target);
    }

    fn canopy_drag(&self, point: Vec3) -> f32 {
        let (u, v) = self.frame.decompose(point);
        let elev = self.frame.elevation_of(point);
        let base = self.origin.canopy_drag(point);
        self.forest
            .read()
            .expect("SectorForest lock poisoned")
            .canopy_drag_near(u, v, elev, base, self.origin.as_ref(), &self.frame)
    }

    fn harvest(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let harvested = self.origin.harvest(eye, direction);
        if harvested.is_some() {
            self.forest
                .get_mut()
                .expect("SectorForest lock poisoned")
                .settle_near(eye, self.origin.as_ref(), &self.frame);
        }
        harvested
    }

    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.origin.loosen(eye, direction, stage)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.forest
            .get_mut()
            .expect("SectorForest lock poisoned")
            .strike_near(eye, direction, self.origin.as_ref(), &self.frame)
    }

    fn forage_timber(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.forest
            .get_mut()
            .expect("SectorForest lock poisoned")
            .forage_near(eye, direction, self.origin.as_ref(), &self.frame)
    }

    fn cast_litter(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        if !find.gathered() {
            return false;
        }
        let forest = self.forest.get_mut().expect("SectorForest lock poisoned");
        let landing = LitterLanding::new(self.origin.as_ref(), self.frame);
        let Some(spot) = landing.resolve(eye, direction, forest) else {
            return false;
        };
        forest.append_litter(spot, find, self.origin.as_ref(), self.frame);
        true
    }

    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        self.origin.plan_molded_block(eye, direction, avatar_pos, mold)
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let changed = self.origin.place_molded_block(eye, direction, avatar_pos, mold);
        if changed {
            self.forest
                .get_mut()
                .expect("SectorForest lock poisoned")
                .settle_near(eye, self.origin.as_ref(), &self.frame);
        }
        changed
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.origin.adjust_rig(eye, direction, delta_steps, rotate_quarter)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool { self.origin.lower_rig(eye, direction) }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.origin.board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool { self.origin.park_vehicle(assembly) }

    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        self.origin.capture_vehicle_blueprint(eye, direction)
    }

    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.origin.aimed_empty_rig(eye, direction, avatar_pos)
    }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        let changed = self.origin.build_rig_blueprint(eye, direction, blueprint);
        if changed {
            self.forest
                .get_mut()
                .expect("SectorForest lock poisoned")
                .settle_near(eye, self.origin.as_ref(), &self.frame);
        }
        changed
    }

    fn present_vessels(&self, sink: &mut dyn VesselSink) { self.origin.present_vessels(sink); }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.origin.placed_camera_tripod() }
    fn collect_spawn_points(&self, out: &mut Vec<Vec3>) { self.origin.collect_spawn_points(out); }
    fn step_surface(&mut self, dt: f32) -> bool { self.origin.step_surface(dt) }

    fn step_flora(&mut self, dt: f32) -> bool {
        self.forest.get_mut().expect("SectorForest lock poisoned").step_active(dt)
    }

    fn lethal_at(&self, point: Vec3) -> bool { self.origin.lethal_at(point) }
    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 { self.origin.confine_beneath_crust(point) }
    fn accept_disclosure(&mut self, stratum: crate::ExposedStratum) { self.origin.accept_disclosure(stratum); }
    fn drain_disclosures(&mut self, out: &mut Vec<crate::ExposedStratum>) { self.origin.drain_disclosures(out); }
}
