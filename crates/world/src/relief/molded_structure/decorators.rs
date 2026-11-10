use glam::Vec3;
use crate::MeshTarget;
use crate::block::{
    BlockMedium, CuboidShape, ElevatedShape, GridCell, MediumShape, VoxelShape,
};
use crate::frame::SurfaceFrame;
use super::super::cantilever_rig::RigRegistry;
use super::super::craft_cluster::ParkedFleet;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::{MoldedStructure, VoxelCells, NICHE_CONTACT_TOLERANCE};

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Static World-Cube molded blocks (buildings, terrain stacks, benches, and decorative parts).
pub struct WorldBlocks {
    grid: VoxelCells,
}

impl WorldBlocks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(grid: VoxelCells) -> Self {
        Self { grid }
    }

    pub fn untouched() -> Self {
        Self::new(VoxelCells::untouched())
    }

    pub fn insert(&mut self, u: i32, v: i32, floor_elev: f32, shape: Box<dyn VoxelShape>) {
        self.grid.insert(u, v, floor_elev, shape);
    }

    pub fn insert_elevated(&mut self, u: i32, v: i32, shape: ElevatedShape) {
        self.grid.insert_elevated(u, v, shape);
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta: i32) -> Option<(i32, i32)> {
        self.grid.step_camera_turn(u, v, delta)
    }

    pub fn confine_niche_elevation(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        self.grid.confine_niche(u, v, ground_h, elev)
    }

    pub fn remove_layer(&mut self, u: i32, v: i32, layer_elev: f32, ground_top: f32, scars: &mut TurfScars) -> Option<((i32, i32), BlockMedium)> {
        let (removed, column_vacated) = self.grid.remove_at(u, v, layer_elev)?;
        if column_vacated && removed.floor() >= ground_top - NICHE_CONTACT_TOLERANCE {
            scars.expose_dirt(u, v);
        }
        Some((scars.clear_stage(u, v), removed.block_medium()))
    }

    pub fn retain_overhang(&mut self, strata: &NaturalStrata, u: i32, v: i32, next_elev: i32, elev: i32) {
        let (nat, vault) = (strata.elevation_at(u, v), strata.vault());
        for step in (next_elev + 1)..elev {
            let shape: Box<dyn VoxelShape> = if vault.crust_layer_at(u, v, step) {
                Box::new(MediumShape::new(Box::new(CuboidShape::new(BLOCK_STEP)), BlockMedium::CrustRock, true, false))
            } else if step == nat - 1 {
                strata.crown_shape_at(u, v)
            } else {
                Box::new(CuboidShape::new(BLOCK_STEP))
            };
            self.insert(u, v, (step as f32) * BLOCK_STEP, shape);
        }
    }

    pub fn settle_gravity(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation) -> Vec<(i32, i32)> {
        self.grid.settle_gravity(|u, v| (shafts.elevation_at(strata, u, v) as f32) * BLOCK_STEP)
    }
}

impl MoldedStructure for WorldBlocks {
    fn vacant(&self) -> bool { self.grid.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.grid.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 { self.grid.top_steps_at(u, v, floor_steps) }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 { self.grid.constrain_support(cell, planar, h, span) }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.grid.layer_occupied_at(u, v, layer_elev) }
    fn locked_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> { self.grid.seat_backrest_cell(u, v, layer_elev) }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> { self.grid.medium_at(u, v, layer_elev) }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> { self.grid.hit_top_at(cu, cv, u, v, ray_elev) }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> { self.grid.perch_at(u, v, layer_elev, eu, ev, frame) }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.grid.tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        self.grid.emit_at(cell, frame, colors, scars, target)
    }
}

/// Decorator layering parked independent vehicle objects over an inner molded structure.
pub struct ParkedStructure<'a, S: MoldedStructure> {
    inner: S,
    fleet: &'a ParkedFleet,
}

impl<'a, S: MoldedStructure> ParkedStructure<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: S, fleet: &'a ParkedFleet) -> Self {
        Self { inner, fleet }
    }
}

impl<S: MoldedStructure> MoldedStructure for ParkedStructure<'_, S> {
    fn vacant(&self) -> bool { self.fleet.vacant() && self.inner.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.fleet.present_at(u, v) || self.inner.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        self.fleet.top_steps_at(u, v, self.inner.top_steps_at(u, v, floor_steps))
    }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.fleet.constrain_support(cell, planar, self.inner.constrain_support(cell, planar, h, span), span)
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.fleet.layer_occupied_at(u, v, layer_elev) || self.inner.layer_occupied_at(u, v, layer_elev)
    }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.fleet.layer_occupied_at(u, v, layer_elev) || self.inner.locked_at(u, v, layer_elev)
    }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.fleet.seat_backrest_cell(u, v, layer_elev).or_else(|| self.inner.seat_backrest_cell(u, v, layer_elev))
    }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.fleet.medium_at(u, v, layer_elev).or_else(|| self.inner.medium_at(u, v, layer_elev))
    }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.fleet.hit_top_at(cu, cv, u, v, ray_elev).or_else(|| self.inner.hit_top_at(cu, cv, u, v, ray_elev))
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.fleet.perch_at(u, v, layer_elev, eu, ev, frame).or_else(|| self.inner.perch_at(u, v, layer_elev, eu, ev, frame))
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.fleet.tripod_hub(frame).or_else(|| self.inner.tripod_hub(frame))
    }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let inner_emitted = self.inner.emit_at(cell, frame, colors, scars, target);
        let fleet_emitted = self.fleet.emit_at(cell, frame, colors, scars, target);
        inner_emitted || fleet_emitted
    }
}

/// Decorator layering active cantilever cranes and their suspended objects over an inner structure.
pub struct RiggedStructure<'a, S: MoldedStructure> {
    inner: S,
    rigs: &'a RigRegistry,
}

impl<'a, S: MoldedStructure> RiggedStructure<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: S, rigs: &'a RigRegistry) -> Self {
        Self { inner, rigs }
    }
}

impl<S: MoldedStructure> MoldedStructure for RiggedStructure<'_, S> {
    fn vacant(&self) -> bool { self.rigs.vacant() && self.inner.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.rigs.present_at(u, v) || self.inner.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        self.rigs.top_steps_at(u, v, self.inner.top_steps_at(u, v, floor_steps))
    }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.rigs.constrain_support(cell, planar, self.inner.constrain_support(cell, planar, h, span), span)
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.rigs.layer_occupied_at(u, v, layer_elev) || self.inner.layer_occupied_at(u, v, layer_elev)
    }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.inner.locked_at(u, v, layer_elev) }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.rigs.seat_backrest_cell(u, v, layer_elev).or_else(|| self.inner.seat_backrest_cell(u, v, layer_elev))
    }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.rigs.medium_at(u, v, layer_elev).or_else(|| self.inner.medium_at(u, v, layer_elev))
    }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.rigs.hit_top_at(cu, cv, u, v, ray_elev).or_else(|| self.inner.hit_top_at(cu, cv, u, v, ray_elev))
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.rigs.perch_at(u, v, layer_elev, eu, ev, frame).or_else(|| self.inner.perch_at(u, v, layer_elev, eu, ev, frame))
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.rigs.tripod_hub(frame).or_else(|| self.inner.tripod_hub(frame))
    }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let rigs_emitted = self.rigs.emit_at(cell, frame, colors, scars, target);
        let inner_emitted = self.inner.emit_at(cell, frame, colors, scars, target);
        rigs_emitted || inner_emitted
    }
}
