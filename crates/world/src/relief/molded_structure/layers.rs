use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use super::super::cantilever_rig::RigRegistry;
use super::super::craft_cluster::ParkedFleet;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::{MoldedStructure, WorldBlocks};

/// Decorator guarding columns that carry an indestructible world sanctuary plinth (`OriginStone`).
pub struct SanctuaryStructure<'a> {
    blocks: &'a WorldBlocks,
}

impl<'a> SanctuaryStructure<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(blocks: &'a WorldBlocks) -> Self {
        Self { blocks }
    }
}

impl MoldedStructure for SanctuaryStructure<'_> {
    fn vacant(&self) -> bool { self.blocks.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.blocks.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 { self.blocks.top_steps_at(u, v, floor_steps) }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.blocks.constrain_support(cell, planar, h, span)
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.blocks.layer_occupied_at(u, v, layer_elev) }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.blocks.sanctuary_at(u, v) || self.blocks.locked_at(u, v, layer_elev)
    }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.blocks.seat_backrest_cell(u, v, layer_elev)
    }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> { self.blocks.medium_at(u, v, layer_elev) }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.blocks.hit_top_at(cu, cv, u, v, ray_elev)
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.blocks.perch_at(u, v, layer_elev, eu, ev, frame)
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.blocks.tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        self.blocks.emit_at(cell, frame, colors, scars, target)
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
