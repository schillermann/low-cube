use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use crate::grid_body::{GridBody, SurfaceFleetView};
use super::super::cantilever_rig::RigRegistry;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::MoldedStructure;

/// Decorator guarding columns that carry an indestructible world sanctuary plinth (`OriginStone`).
pub struct SanctuaryStructure<'a> {
    body: &'a GridBody,
}

impl<'a> SanctuaryStructure<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(body: &'a GridBody) -> Self {
        Self { body }
    }
}

impl MoldedStructure for SanctuaryStructure<'_> {
    fn vacant(&self) -> bool { self.body.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.body.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 { self.body.top_steps_at(u, v, floor_steps) }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.body.constrain_support(cell, planar, h, span)
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.body.layer_occupied_at(u, v, layer_elev) }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.body.crust().sanctuary_at(u, v) || self.body.locked_at(u, v, layer_elev)
    }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.body.seat_backrest_cell(u, v, layer_elev)
    }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> { self.body.medium_at(u, v, layer_elev) }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.body.hit_top_at(cu, cv, u, v, ray_elev)
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.body.perch_at(u, v, layer_elev, eu, ev, frame)
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.body.tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        self.body.emit_at(cell, frame, colors, scars, target)
    }
}

/// Decorator layering settled `GridBody`s (`SurfaceFleetView`) over an inner molded structure.
pub struct ParkedStructure<'a, S: MoldedStructure> {
    inner: S,
    fleet: SurfaceFleetView<'a>,
}

impl<'a, S: MoldedStructure> ParkedStructure<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: S, fleet: SurfaceFleetView<'a>) -> Self {
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

/// Decorator layering active `MagneticFlange` `GridBody` assembly sites over an inner structure.
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
