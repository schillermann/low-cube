use std::collections::HashMap;
use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::super::craft_cluster::ParkedFleet;
use super::super::molded_structure::{MoldedStructure, WorldBlocks};
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::sight_ray::SurfaceSightRay;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::{VehicleAssembly, VehicleAxes};
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

    pub fn place_on_rig(&mut self, mast: (i32, i32), cell: (i32, i32), floor_elev: f32, shape: Box<dyn VoxelShape>) {
        let relocated = self
            .rigs
            .get_mut(&mast)
            .and_then(|rig| rig.place_craft_block(mast.0, mast.1, cell.0, cell.1, floor_elev, shape));
        if let Some(new_mast) = relocated {
            if let Some(rig) = self.rigs.remove(&mast) {
                self.rigs.insert(new_mast, rig);
            }
        }
    }

    pub fn remove_craft_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.rigs.iter_mut().find_map(|(&(mu, mv), rig)| rig.remove_craft_layer(mu, mv, u, v, layer_elev))
    }

    pub fn pack_up(&mut self, u: i32, v: i32, layer_elev: f32, ground_top: f32, world_blocks: &mut WorldBlocks) -> bool {
        if !self.rigs.get(&(u, v)).is_some_and(|rig| rig.occupies_mast_layer(layer_elev)) {
            return false;
        }
        let Some(rig) = self.rigs.remove(&(u, v)) else { return false };
        rig.pack_up(u, v, ground_top, world_blocks);
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
            self.rigs.iter().find_map(|(&(mu, mv), rig)| rig.craft_present_at(u, v).then_some((mu, mv)))
        };
        let Some((mu, mv)) = target_mast else { return false };
        let Some(rig) = self.rigs.get_mut(&(mu, mv)) else { return false };
        rig.adjust(mu, mv, delta_steps, rotate_quarter, |cu, cv| {
            (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
        })
    }

    pub fn find_aimed_rig(&self, u: i32, v: i32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> Option<(i32, i32)> {
        if let Some((&mast, _)) = self.rigs.iter().find(|(&(mu, mv), r)| r.aimed_empty_boom(mu, mv, u, v)) {
            return Some(mast);
        }
        (1..=60).find_map(|step| {
            let sample = ray.sample_point((step as f32) * 0.08);
            let (su, sv) = frame.decompose(sample);
            let (ru, rv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            self.rigs.iter().find_map(|(&(mu, mv), r)| r.aimed_empty_boom(mu, mv, ru, rv).then_some((mu, mv)))
        })
    }

    pub fn reconnect_seated_car(
        &mut self,
        mast: (i32, i32),
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
        fleet: &mut ParkedFleet,
    ) -> Option<VehicleAssembly> {
        let (eu, ev) = frame.decompose(ray.sample_point(0.0));
        let parked = fleet.take_seated_near(eu, ev, 5.0)?;
        let Some(mut rig) = self.rigs.remove(&mast) else {
            fleet.park(parked);
            return None;
        };
        match rig.mount_parked(mast.0, mast.1, parked, eu, ev, frame) {
            Ok((relocated, origin, heading)) => {
                self.rigs.insert(relocated.unwrap_or(mast), rig);
                let axes = VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal());
                Some(VehicleAssembly::new(origin, axes, heading, Vec::new()))
            }
            Err(unmoved) => {
                self.rigs.insert(mast, rig);
                fleet.park(unmoved);
                None
            }
        }
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
