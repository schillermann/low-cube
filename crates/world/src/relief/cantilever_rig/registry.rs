use std::collections::HashMap;
use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::brick_arena::{ModuleSpec, RawAtom};
use crate::frame::SurfaceFrame;
use super::super::craft_cluster::ParkedFleet;
use super::super::craft_entity::CraftBlueprint;
use super::super::molded_structure::{MoldedStructure, WorldBlocks};
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::sight_ray::SurfaceSightRay;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::blueprint_mount::{RigBlueprintQuery, SeatedCarMount};
use super::{CantileverRig, BLOCK_STEP, MAX_HOIST_STEPS};

/// Placement routing decision resolved across active cantilever cranes.
pub enum RigPlacement {
    BlockedMast,
    MountFirst { mast: (i32, i32), cell: (i32, i32), floor_elev: f32 },
    AttachCraft { mast: (i32, i32), cell: (i32, i32), floor_elev: f32 },
    WorldCube,
}

/// Collection of active polyhedral cantilever rigs and their suspended craft objects.
pub struct RigRegistry {
    rigs: HashMap<(i32, i32), CantileverRig>,
}

impl RigRegistry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(rigs: HashMap<(i32, i32), CantileverRig>) -> Self {
        Self { rigs }
    }

    pub fn untouched() -> Self {
        Self::new(HashMap::new())
    }

    pub fn deploy(&mut self, u: i32, v: i32, ground_top: f32) {
        self.rigs.insert((u, v), CantileverRig::deployed(ground_top, 0));
    }

    pub fn capture_sector(&self, gsu: i32, gsv: i32) -> Vec<crate::RigSnapshot> {
        let (u0, v0) = (gsu * 8, gsv * 8);
        let mut out: Vec<_> = self
            .rigs
            .iter()
            .filter(|&(&(u, v), _)| u >= u0 && u < u0 + 8 && v >= v0 && v < v0 + 8)
            .map(|(&(u, v), r)| r.snapshot_at(u, v, gsu, gsv))
            .collect();
        out.sort_unstable_by_key(crate::RigSnapshot::local_uv);
        out
    }

    pub fn apply_sector(&mut self, gsu: i32, gsv: i32, rigs: &[crate::RigSnapshot]) {
        let (u0, v0) = (gsu * 8, gsv * 8);
        self.rigs.retain(|&(u, v), _| u < u0 || u >= u0 + 8 || v < v0 || v >= v0 + 8);
        for snap in rigs {
            let (mast, rig) = CantileverRig::from_snapshot(gsu, gsv, snap);
            self.rigs.insert(mast, rig);
        }
    }

    pub fn collect_modified_sectors(&self, out: &mut Vec<(i32, i32)>) {
        for &(u, v) in self.rigs.keys() {
            out.push((u.div_euclid(8), v.div_euclid(8)));
        }
    }

    pub fn boom_slot_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.rigs.iter().any(|(&(mu, mv), rig)| rig.boom_slot_at(mu, mv, u, v, layer_elev))
    }

    pub fn resolve_placement(&self, u: i32, v: i32, ground_top: f32, sub: bool, raw_floor: f32) -> RigPlacement {
        if let Some((mast, (eu, ev, floor_elev))) = self
            .rigs
            .iter()
            .find_map(|(&(mu, mv), rig)| rig.mount_first_target(mu, mv, u, v).map(|t| ((mu, mv), t)))
        {
            return RigPlacement::MountFirst { mast, cell: (eu, ev), floor_elev };
        }
        if let Some((mast, floor_elev)) = self.rigs.iter().find_map(|(&(mu, mv), rig)| {
            rig.resolve_craft_attach(mu, mv, u, v, ground_top, sub, raw_floor).map(|z| ((mu, mv), z))
        }) {
            return RigPlacement::AttachCraft { mast, cell: (u, v), floor_elev };
        }
        if self.rigs.contains_key(&(u, v)) {
            return RigPlacement::BlockedMast;
        }
        RigPlacement::WorldCube
    }

    pub fn place_on_rig(&mut self, mast: (i32, i32), cell: (i32, i32), floor_elev: f32, atom: RawAtom) {
        let relocated = self
            .rigs
            .get_mut(&mast)
            .and_then(|rig| rig.place_craft_block(mast.0, mast.1, cell.0, cell.1, floor_elev, atom));
        if let Some(new_mast) = relocated {
            if let Some(rig) = self.rigs.remove(&mast) {
                self.rigs.insert(new_mast, rig);
            }
        }
    }

    pub fn place_module_on_rig(&mut self, mast: (i32, i32), cell: (i32, i32), floor_elev: f32, spec: ModuleSpec) -> bool {
        let Some((ok, relocated)) = self
            .rigs
            .get_mut(&mast)
            .and_then(|rig| rig.place_craft_module(mast.0, mast.1, cell.0, cell.1, floor_elev, spec))
        else {
            return false;
        };
        if let Some(new_mast) = relocated {
            if let Some(rig) = self.rigs.remove(&mast) {
                self.rigs.insert(new_mast, rig);
            }
        }
        ok
    }

    pub fn remove_craft_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.rigs.iter_mut().find_map(|(&(mu, mv), rig)| rig.remove_craft_layer(mu, mv, u, v, layer_elev))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pack_up(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        u: i32,
        v: i32,
        layer_elev: f32,
        world_blocks: &mut WorldBlocks,
        scars: &mut TurfScars,
    ) -> bool {
        if !self.rigs.get(&(u, v)).is_some_and(|rig| rig.occupies_mast_layer(layer_elev)) {
            return false;
        }
        let Some(rig) = self.rigs.remove(&(u, v)) else { return false };
        rig.pack_up(u, v, strata, shafts, world_blocks, scars);
        true
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        self.rigs.values_mut().find_map(|rig| rig.step_camera_turn(u, v, delta_steps))
    }

    pub fn lower_mast_at(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32) -> bool {
        self.rigs.contains_key(&(u, v)) && self.adjust_at(strata, shafts, u, v, -MAX_HOIST_STEPS, false)
    }

    pub fn adjust_at(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        u: i32,
        v: i32,
        delta_steps: i32,
        rotate_quarter: bool,
    ) -> bool {
        let target_mast = if self.rigs.contains_key(&(u, v)) {
            Some((u, v))
        } else {
            self.rigs.iter().find_map(|(&(mu, mv), rig)| rig.boom_or_craft_at(mu, mv, u, v).then_some((mu, mv)))
        };
        let Some((mu, mv)) = target_mast else { return false };
        let Some(rig) = self.rigs.get_mut(&(mu, mv)) else { return false };
        rig.adjust(mu, mv, delta_steps, rotate_quarter, |cu, cv| {
            (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
        })
    }

    pub fn find_aimed_rig(&self, u: i32, v: i32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> Option<(i32, i32)> {
        RigBlueprintQuery::new(&self.rigs).find_aimed(u, v, ray, frame)
    }

    pub fn reconnect_seated_car(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        mast: (i32, i32),
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
        fleet: &mut ParkedFleet,
    ) -> Option<VehicleAssembly> {
        SeatedCarMount::new(&mut self.rigs, fleet).reconnect(strata, shafts, mast, ray, frame)
    }

    pub fn board_drivable(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        u: i32,
        v: i32,
        layer_elev: f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> Option<VehicleAssembly> {
        self.rigs.iter_mut().find_map(|(&(mu, mv), rig)| {
            rig.release_drivable(mu, mv, u, v, layer_elev, ray, frame, |cu, cv| {
                (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
            })
        })
    }

    pub fn capture_blueprint(
        &self,
        u: i32,
        v: i32,
        layer_elev: f32,
        eu: f32,
        ev: f32,
        frame: &SurfaceFrame,
    ) -> Option<CraftBlueprint> {
        RigBlueprintQuery::new(&self.rigs).capture((u, v, layer_elev), (eu, ev), frame)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_empty_rig(
        &self,
        u: i32,
        v: i32,
        ground_top: f32,
        sub: bool,
        raw_floor: f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> Option<(i32, i32)> {
        if let RigPlacement::MountFirst { mast, .. } = self.resolve_placement(u, v, ground_top, sub, raw_floor) {
            return Some(mast);
        }
        if self.rigs.get(&(u, v)).is_some_and(CantileverRig::vacant_craft) {
            return Some((u, v));
        }
        self.find_aimed_rig(u, v, ray, frame)
    }

    pub fn build_blueprint(
        &mut self,
        mast: (i32, i32),
        blueprint: &CraftBlueprint,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
    ) -> Option<(i32, i32)> {
        let mut rig = self.rigs.remove(&mast)?;
        let mounted = rig.mount_blueprint(mast.0, mast.1, blueprint, frame, |cu, cv| {
            (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
        });
        let final_mast = mounted.map(|rel| rel.unwrap_or(mast));
        self.rigs.insert(final_mast.unwrap_or(mast), rig);
        final_mast
    }
}

impl MoldedStructure for RigRegistry {
    fn vacant(&self) -> bool { self.rigs.is_empty() }
    fn present_at(&self, u: i32, v: i32) -> bool {
        self.rigs.contains_key(&(u, v)) || self.rigs.values().any(|r| r.craft_present_at(u, v))
    }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        self.rigs.values().fold(floor_steps, |acc, r| r.top_steps_at(u, v, acc))
    }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.rigs.iter().fold(h, |acc, (&(mu, mv), r)| r.constrain_at(mu, mv, cell, planar, acc, span))
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.rigs.iter().any(|(&(mu, mv), r)| r.layer_occupied_at(mu, mv, u, v, layer_elev))
    }
    fn locked_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.rigs.values().find_map(|r| r.seat_backrest_cell(u, v, layer_elev))
    }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        if let Some(med) = self.rigs.values().find_map(|r| r.craft_medium_at(u, v, layer_elev)) {
            return Some(med);
        }
        self.rigs.get(&(u, v)).is_some_and(|r| r.occupies_mast_layer(layer_elev)).then_some(BlockMedium::CantileverRig)
    }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.rigs.iter().find_map(|(&(mu, mv), rig)| rig.hit_boom_or_craft(mu, mv, cu, cv, u, v, ray_elev))
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.rigs.values().find_map(|r| r.perch_at(u, v, layer_elev, eu, ev, frame))
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.rigs.values().find_map(|r| r.tripod_hub(frame))
    }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let mut emitted = false;
        for (&(mu, mv), rig) in &self.rigs {
            if rig.emit_mast_and_craft(mu, mv, cell, frame, colors, scars, target) {
                emitted = true;
            }
        }
        emitted
    }
}
