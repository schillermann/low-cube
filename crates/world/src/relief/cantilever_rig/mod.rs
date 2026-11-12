mod arm;
mod mesh;
mod registry;

use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::craft_cluster::CraftCluster;
use super::molded_structure::{MoldedStructure, WorldBlocks};
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::sight_ray::SurfaceSightRay;
use super::turf_scars::TurfScars;
use super::vehicle_assembly::VehicleAssembly;
pub use self::arm::RigArm;
pub use self::registry::{RigPlacement, RigRegistry};

pub(super) const BLOCK_STEP: f32 = GridCell::SPAN;
pub(super) const MAX_HOIST_STEPS: i32 = 8;

/// Polyhedral 3D cantilever crane owning its independent multi-block craft object.
pub struct CantileverRig {
    arm: RigArm,
    craft: CraftCluster,
}

impl CantileverRig {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(arm: RigArm, craft: CraftCluster) -> Self {
        Self { arm, craft }
    }

    pub fn deployed(ground_top: f32, turn: u8) -> Self {
        Self::new(RigArm::deployed(ground_top, turn), CraftCluster::untouched())
    }

    pub fn occupies_mast_layer(&self, layer_elev: f32) -> bool { self.arm.occupies_mast_layer(layer_elev) }
    pub fn craft_present_at(&self, u: i32, v: i32) -> bool { self.craft.present_at(u, v) }
    pub fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 { self.craft.top_steps_at(u, v, floor_steps) }
    pub fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> { self.craft.seat_backrest_cell(u, v, layer_elev) }
    pub fn craft_medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> { self.craft.medium_at(u, v, layer_elev) }
    pub fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.craft.tripod_hub(frame) }
    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> { self.craft.step_camera_turn(u, v, delta_steps) }
    pub fn remove_craft_layer(&mut self, mast_u: i32, mast_v: i32, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        let removed = self.craft.remove_layer(u, v, layer_elev)?;
        if self.craft.vacant() {
            self.arm.reset_reach();
        } else {
            let hoist_z = self.arm.hoist_elevation();
            self.arm.advance_toward_craft(mast_u, mast_v, |cu, cv| {
                self.craft.occupies_claw((cu, cv), hoist_z) || self.craft.present_at(cu, cv)
            });
        }
        Some(removed)
    }

    pub fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.craft.perch_at(u, v, layer_elev, eu, ev, frame)
    }

    pub fn constrain_at(&self, mast_u: i32, mast_v: i32, cell: GridCell, planar: (f32, f32), mut h: f32, span: (f32, f32)) -> f32 {
        let (cu, cv) = cell.feed_coordinates(|gu, gv| (gu, gv));
        if (cu, cv) == (mast_u, mast_v) {
            h = self.arm.constrain_mast_support(h, span.0);
        }
        self.craft.constrain_support(cell, planar, h, span)
    }

    pub fn layer_occupied_at(&self, mast_u: i32, mast_v: i32, u: i32, v: i32, layer_elev: f32) -> bool {
        ((u, v) == (mast_u, mast_v) && self.arm.occupies_mast_layer(layer_elev))
            || self.craft.layer_occupied_at(u, v, layer_elev)
    }

    pub fn aimed_empty_boom(&self, mast_u: i32, mast_v: i32, u: i32, v: i32) -> bool {
        self.craft.vacant() && self.arm.occupies_boom_cell(mast_u, mast_v, u, v)
    }

    pub fn mount_first_target(&self, mast_u: i32, mast_v: i32, u: i32, v: i32) -> Option<(i32, i32, f32)> {
        (self.craft.vacant() && self.arm.hoisted() && self.arm.near_boom(mast_u, mast_v, u, v)).then(|| {
            let (eu, ev) = self.arm.gripped_cell(mast_u, mast_v);
            (eu, ev, self.arm.hoist_elevation())
        })
    }

    pub fn resolve_craft_attach(&self, mast_u: i32, mast_v: i32, u: i32, v: i32, ground_top: f32, sub: bool, raw_floor: f32) -> Option<f32> {
        if self.craft.vacant() || ((u, v) == (mast_u, mast_v) && self.arm.claw_slot(mast_u, mast_v) != (mast_u, mast_v)) {
            return None;
        }
        let mut floor_elev = if sub { raw_floor } else { raw_floor.max(ground_top) };
        if !self.craft.touches_placement(u, v, floor_elev) {
            if !sub && (floor_elev - ground_top).abs() <= 0.05 {
                floor_elev = self.craft.adjacent_hoisted_floor(u, v, |min_z| self.arm.above_base(min_z))?;
            } else {
                return None;
            }
        }
        while self.craft.layer_occupied_at(u, v, floor_elev + 0.25) {
            floor_elev += if sub { -BLOCK_STEP } else { BLOCK_STEP };
        }
        Some(floor_elev)
    }

    pub fn place_craft_block(&mut self, mast_u: i32, mast_v: i32, u: i32, v: i32, floor_elev: f32, shape: Box<dyn VoxelShape>) -> Option<(i32, i32)> {
        let relocated = self.arm.retract_for_placed_block(mast_u, mast_v, u, v, floor_elev);
        self.craft.insert(u, v, floor_elev, shape);
        relocated
    }

    pub fn hit_boom_or_craft(&self, mast_u: i32, mast_v: i32, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        if let Some(craft_top) = self.craft.hit_top_at(cu, cv, u, v, ray_elev) {
            return Some(craft_top);
        }
        if (cu, cv) == (mast_u, mast_v) && self.arm.occupies_mast_layer(ray_elev) {
            return Some(self.arm.mast_ceiling());
        }
        let claw = self.arm.gripped_cell(mast_u, mast_v);
        let hoist_z = self.arm.hoist_elevation();
        (self.arm.hoisted()
            && !self.craft.occupies_claw(claw, hoist_z)
            && self.arm.occupies_boom_cell(mast_u, mast_v, cu, cv)
            && (ray_elev - (hoist_z + 0.25)).abs() <= 0.30)
            .then_some(hoist_z + BLOCK_STEP)
    }

    pub fn emit_mast_and_craft(&self, mast_u: i32, mast_v: i32, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|gu, gv| (gu, gv));
        let mast_emitted = if (u, v) == (mast_u, mast_v) {
            self.arm.geometry().emit(cell, frame, target);
            true
        } else {
            false
        };
        let craft_emitted = self.craft.emit_at(cell, frame, colors, scars, target);
        mast_emitted || craft_emitted
    }

    pub fn adjust(&mut self, mast_u: i32, mast_v: i32, delta_steps: i32, rotate_quarter: bool, ground_fn: impl Fn(i32, i32) -> f32) -> bool {
        if self.craft.vacant() {
            return false;
        }
        if delta_steps != 0 {
            let raw_dz = self.arm.clamped_step_delta(delta_steps);
            let dz = self.craft.clamped_vertical_delta(raw_dz, ground_fn);
            if dz.abs() < 1e-4 {
                return false;
            }
            self.arm.apply_vertical_shift(dz);
            self.craft.shift_elevation(dz);
            return true;
        }
        if rotate_quarter {
            let (piv_u, piv_v) = self.arm.gripped_cell(mast_u, mast_v);
            self.craft.rotate_around(piv_u, piv_v);
            return true;
        }
        false
    }

    pub fn mount_parked(
        &mut self,
        mast_u: i32,
        mast_v: i32,
        mut parked: CraftCluster,
        eu: f32,
        ev: f32,
        frame: &SurfaceFrame,
    ) -> Result<(Option<(i32, i32)>, Vec3, Vec3), CraftCluster> {
        if !self.craft.vacant() {
            return Err(parked);
        }
        self.arm.reset();
        let (gu, gv) = self.arm.gripped_cell(mast_u, mast_v);
        let hoist_z = self.arm.hoist_elevation();
        let Some((moved_floors, origin, heading)) = parked.relocate_to_claw((gu, gv, hoist_z), eu, ev, frame) else {
            return Err(parked);
        };
        let (mut cur_u, mut cur_v, mut relocated) = (mast_u, mast_v, None);
        for (ncu, ncv, nf) in moved_floors {
            if let Some(next_mast) = self.arm.retract_for_placed_block(cur_u, cur_v, ncu, ncv, nf) {
                (cur_u, cur_v) = next_mast;
                relocated = Some(next_mast);
            }
        }
        self.craft = parked;
        Ok((relocated, origin, heading))
    }

    pub fn release_drivable(
        &mut self,
        mast_u: i32,
        mast_v: i32,
        u: i32,
        v: i32,
        layer_elev: f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
        ground_fn: impl Fn(i32, i32) -> f32,
    ) -> Option<VehicleAssembly> {
        if !self.craft.targets_steering(u, v, layer_elev, ray, frame) || !self.craft.drivable() {
            return None;
        }
        let _ = self.adjust(mast_u, mast_v, -MAX_HOIST_STEPS, false, &ground_fn);
        let craft = std::mem::replace(&mut self.craft, CraftCluster::untouched());
        match craft.into_vehicle(&ground_fn, ray, frame) {
            Ok(assembly) => {
                self.arm.reset();
                Some(assembly)
            }
            Err(unmoved) => {
                self.craft = unmoved;
                None
            }
        }
    }

    pub fn pack_up(
        mut self,
        mast_u: i32,
        mast_v: i32,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        world_blocks: &mut WorldBlocks,
        scars: &mut TurfScars,
    ) {
        let _ = self.adjust(mast_u, mast_v, -MAX_HOIST_STEPS, false, |cu, cv| {
            (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
        });
        self.craft.dismantle_into(strata, shafts, world_blocks, scars);
    }
}
