mod cells;
mod decorators;

use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use super::turf_scars::TurfScars;
use super::vehicle_assembly::VehicleAssembly;
pub use self::cells::VoxelCells;
pub use self::decorators::{ParkedStructure, RiggedStructure, WorldBlocks};

pub(super) const NICHE_CONTACT_TOLERANCE: f32 = 0.05;

/// Polymorphic 3D surface structure contract composed across world blocks, parked crafts, and cranes.
pub trait MoldedStructure {
    fn vacant(&self) -> bool;
    fn present_at(&self, u: i32, v: i32) -> bool;
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32;
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32;
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool;
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool;
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)>;
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium>;
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32>;
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly>;
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)>;
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool;
}

impl<T: MoldedStructure + ?Sized> MoldedStructure for &T {
    fn vacant(&self) -> bool { (**self).vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { (**self).present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 { (**self).top_steps_at(u, v, floor_steps) }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 { (**self).constrain_support(cell, planar, h, span) }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { (**self).layer_occupied_at(u, v, layer_elev) }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { (**self).locked_at(u, v, layer_elev) }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> { (**self).seat_backrest_cell(u, v, layer_elev) }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> { (**self).medium_at(u, v, layer_elev) }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> { (**self).hit_top_at(cu, cv, u, v, ray_elev) }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> { (**self).perch_at(u, v, layer_elev, eu, ev, frame) }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { (**self).tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        (**self).emit_at(cell, frame, colors, scars, target)
    }
}
