use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use crate::relief::{MoldedStructure, TurfScars, VehicleAssembly};
use super::motion::BodyMooring;
use super::GridBody;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// `MoldedStructure` view over surface-settled `GridBody`s inside a `GridFleet` (`1` field).
#[derive(Clone, Copy)]
pub struct SurfaceFleetView<'a> {
    bodies: &'a [GridBody],
}

impl<'a> SurfaceFleetView<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bodies: &'a [GridBody]) -> Self {
        Self { bodies }
    }
}

impl MoldedStructure for SurfaceFleetView<'_> {
    fn vacant(&self) -> bool {
        self.bodies.is_empty()
    }

    fn present_at(&self, u: i32, v: i32) -> bool {
        self.bodies.iter().any(|b| {
            let BodyMooring::SurfaceSettled(ff) = b.mooring() else { return false };
            let (uc, vc) = GridCell::new(u, v).center();
            b.blueprint().constrain_at(b.pose(), ff.point_at(uc, vc, 64.0), 0.0).is_some()
        })
    }

    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        let (uc, vc) = GridCell::new(u, v).center();
        self.bodies.iter().fold(floor_steps, |acc, b| {
            let BodyMooring::SurfaceSettled(ff) = b.mooring() else { return acc };
            match b.blueprint().constrain_at(b.pose(), ff.point_at(uc, vc, 64.0), 0.0) {
                Some(sup) => acc.max((ff.elevation_of(sup) / BLOCK_STEP).ceil() as i32),
                None => acc,
            }
        })
    }

    fn constrain_support(&self, _cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.bodies.iter().fold(h, |acc, b| {
            let BodyMooring::SurfaceSettled(ff) = b.mooring() else { return acc };
            let probe = ff.point_at(planar.0, planar.1, acc + span.0);
            match b.blueprint().constrain_at(b.pose(), probe, span.0) {
                Some(sup) => acc.max(ff.elevation_of(sup) - span.0),
                None => acc,
            }
        })
    }

    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.bodies.iter().any(|b| b.occupies_surface_layer(u, v, layer_elev))
    }

    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.layer_occupied_at(u, v, layer_elev)
    }

    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.bodies.iter().find_map(|b| {
            let (ff, pt) = b.surface_cell_center(u, v, layer_elev)?;
            let back = b.posed_hull().backrest_world_at(pt)?;
            Some(ff.cell_at(back).feed_coordinates(|gu, gv| (gu, gv)))
        })
    }

    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.bodies.iter().find_map(|b| {
            let (_, pt) = b.surface_cell_center(u, v, layer_elev)?;
            b.posed_hull().medium_at_world(pt)
        })
    }

    fn hit_top_at(&self, _cu: i32, _cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.bodies.iter().find_map(|b| {
            let BodyMooring::SurfaceSettled(ff) = b.mooring() else { return None };
            let pt = ff.point_at(u, v, ray_elev);
            b.posed_hull().hit_ceiling_at(pt).map(|top| ff.elevation_of(top))
        })
    }

    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, _frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.bodies.iter().find_map(|b| {
            let (ff, hit) = b.surface_cell_center(u, v, layer_elev)?;
            let eye = ff.point_at(eu, ev, layer_elev + 1.0);
            b.posed_hull().perch_at(hit, eye)
        })
    }

    fn tripod_hub(&self, _frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.bodies.iter().find_map(|b| b.posed_hull().tripod_hub())
    }

    fn emit_at(&self, _cell: GridCell, _frame: &SurfaceFrame, _colors: (Vec3, Vec3), _scars: &TurfScars, _target: &mut dyn MeshTarget) -> bool {
        false
    }
}
