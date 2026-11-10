use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use super::super::molded_structure::MoldedStructure;
use super::super::sight_ray::SurfaceSightRay;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::CraftCluster;

/// Registry of independent vehicle objects parked across a planetary face.
pub struct ParkedFleet {
    crafts: Vec<CraftCluster>,
}

impl ParkedFleet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(crafts: Vec<CraftCluster>) -> Self {
        Self { crafts }
    }

    pub const fn untouched() -> Self {
        Self::new(Vec::new())
    }

    pub fn park(&mut self, craft: CraftCluster) {
        if !craft.vacant() {
            self.crafts.push(craft);
        }
    }

    pub fn blocks_placement_at(&self, u: i32, v: i32, floor_elev: f32) -> bool {
        self.crafts.iter().any(|craft| craft.touches_placement(u, v, floor_elev))
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        self.crafts.iter_mut().find_map(|craft| craft.step_camera_turn(u, v, delta_steps))
    }

    pub fn take_seated_near(&mut self, eu: f32, ev: f32, max_dist: f32) -> Option<CraftCluster> {
        let idx = self
            .crafts
            .iter()
            .enumerate()
            .filter(|(_, c)| c.steerable())
            .filter_map(|(i, c)| {
                let d = c.nearest_seat_distance(eu, ev)?;
                (d <= max_dist).then_some((i, d))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)?;
        Some(self.crafts.swap_remove(idx))
    }

    pub fn board_drivable(
        &mut self,
        u: i32,
        v: i32,
        layer_elev: f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
        ground_fn: impl Fn(i32, i32) -> f32,
    ) -> Option<VehicleAssembly> {
        let idx = self
            .crafts
            .iter()
            .position(|c| c.targets_steering(u, v, layer_elev, ray, frame) && c.drivable())?;
        let craft = self.crafts.swap_remove(idx);
        match craft.into_vehicle(ground_fn, ray, frame) {
            Ok(assembly) => Some(assembly),
            Err(unmoved) => {
                self.crafts.push(unmoved);
                None
            }
        }
    }
}

impl MoldedStructure for ParkedFleet {
    fn vacant(&self) -> bool { self.crafts.is_empty() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.crafts.iter().any(|c| c.present_at(u, v)) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        self.crafts.iter().fold(floor_steps, |acc, c| c.top_steps_at(u, v, acc))
    }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.crafts.iter().fold(h, |acc, c| c.constrain_support(cell, planar, acc, span))
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.crafts.iter().any(|c| c.layer_occupied_at(u, v, layer_elev))
    }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.layer_occupied_at(u, v, layer_elev) }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.crafts.iter().find_map(|c| c.seat_backrest_cell(u, v, layer_elev))
    }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.crafts.iter().find_map(|c| c.medium_at(u, v, layer_elev))
    }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.crafts.iter().find_map(|c| c.hit_top_at(cu, cv, u, v, ray_elev))
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.crafts.iter().find_map(|c| c.perch_at(u, v, layer_elev, eu, ev, frame))
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.crafts.iter().find_map(|c| c.tripod_hub(frame))
    }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let mut emitted = false;
        for craft in &self.crafts {
            if craft.emit_at(cell, frame, colors, scars, target) {
                emitted = true;
            }
        }
        emitted
    }
}
