use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use crate::noise::SpatialFrequency;
use super::corner::NaturalCrown;
use super::sight_ray::SurfaceSightRay;
use super::vehicle_assembly::VehicleAssembly;

/// Behavioral interface for planetary relief elevation evaluation ("Tell, Don't Ask").
pub trait ReliefElevation: Send + Sync {
    /// Discretized integer block elevation relative to base face datum.
    fn elevation_at(&self, u: i32, v: i32) -> i32;

    /// Pristine natural block elevation prior to player earthworks.
    fn natural_elevation_at(&self, u: i32, v: i32) -> i32 { self.elevation_at(u, v) }

    /// Continuous surface elevation in meters for physical avatar constraint.
    fn surface_height_at(&self, u: f32, v: f32) -> f32;

    /// Continuous supporting surface elevation in meters for an entity at foot_elev with body span.
    fn support_height_at(&self, u: f32, v: f32, _foot_elev: f32, _body_span: f32) -> f32 { self.surface_height_at(u, v) }

    /// Indicates whether a placed block occupies the vertical layer at (u, v).
    fn layer_occupied_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }

    /// Adjacent cell directly behind the backrest of a cabin seat at (u, v, layer_elev), if present.
    fn seat_backrest_cell(&self, _u: i32, _v: i32, _layer_elev: f32) -> Option<(i32, i32)> { None }

    /// Checks whether a 3D sight ray point (u, v, ray_elev) hits solid terrain or a placed block, returning its top elevation.
    fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let surf = self.surface_height_at(u, v);
        if ray_elev <= surf + 0.02 { Some(surf) } else { None }
    }

    /// Indicates whether the discrete cell (u, v) was excavated by earthworks.
    fn reworked_at(&self, _u: i32, _v: i32) -> bool { false }

    /// Indicates whether an entity at tangential (u, v) and elevation above datum contacts the lethal exposed core floor.
    fn lethal_at(&self, _u: f32, _v: f32, _elev: f32) -> bool { false }

    /// Deepest diggable crust step of the discrete cell (u, v).
    fn crust_floor_at(&self, _u: i32, _v: i32) -> i32 { i32::MIN }

    /// Tangential coordinates and elevation of an eye at (u, v, elev) kept inside enclosed rooms and niches.
    fn confine(&self, u: f32, v: f32, elev: f32) -> (f32, f32, f32) { (u, v, elev) }

    /// Indicates whether the discrete cell (u, v) has bare exposed soil without regrown grass.
    fn bare_dirt_at(&self, _u: i32, _v: i32) -> bool { false }

    /// Indicates whether the planetary ground surface at (u, v) is scarred or smothered beneath a block.
    fn ground_bare_at(&self, u: i32, v: i32) -> bool { self.bare_dirt_at(u, v) }

    /// Emits molded voxel shapes at cell if present, returning true when emitted.
    fn emit_molded_top(&self, _cell: GridCell, _frame: &SurfaceFrame, _top_col: Vec3, _wall_col: Vec3, _target: &mut dyn MeshTarget) -> bool { false }

    /// Advances grass regrowth across bare dirt cells bordering grass within 1 block height.
    fn step_grass(&mut self, _dt: f32) -> Vec<(i32, i32)> { Vec::new() }

    /// Partial digging stage (0..=3) at a specific vertical layer within (u, v).
    fn layer_dig_stage_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> u8 { 0 }

    /// Clears any active partial digging stage, returning the affected cell and whether a stage was active.
    fn clear_dig_stage(&mut self) -> ((i32, i32), bool) { ((0, 0), false) }

    /// Marks partial digging progress (0..=3) at a specific vertical layer within (u, v).
    fn mark_dig_layer(&mut self, _u: i32, _v: i32, _layer_elev: f32, _stage: u8) -> ((i32, i32), Option<BlockMedium>) { ((0, 0), None) }

    /// Excavates a specific vertical block layer within (u, v).
    fn excavate_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> ((i32, i32), bool) {
        let (cell, dug) = self.harvest_layer(u, v, layer_elev);
        (cell, dug.is_some())
    }

    /// Excavates a specific vertical block layer within (u, v), returning the harvested block medium.
    fn harvest_layer(&mut self, _u: i32, _v: i32, _layer_elev: f32) -> ((i32, i32), Option<BlockMedium>) {
        ((0, 0), None)
    }

    /// Stacks one molded voxel block layer at (u, v).
    fn stack_molded_cell(&mut self, _u: i32, _v: i32, _base: f32, _shape: Box<dyn VoxelShape>) -> bool { false }

    /// Raises, lowers, or rotates a polyhedral cantilever rig at or holding (u, v).
    fn adjust_rig_at(&mut self, _u: i32, _v: i32, _delta_steps: i32, _rotate_quarter: bool) -> (bool, (i32, i32)) { (false, (0, 0)) }

    /// Tripod hub position, base forward axis, and up axis of a placed field camera, if present.
    fn placed_camera_tripod(&self, _frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { None }

    /// Lowers a polyhedral cantilever rig assembly onto the ground when its mast at (u, v) is clicked.
    fn lower_rig_at(&mut self, _u: i32, _v: i32) -> bool { false }

    /// Boards a cabin seat or engages a car engine at (u, v, layer_elev).
    fn board_vehicle_at(
        &mut self, _frame: &SurfaceFrame, _u: i32, _v: i32, _layer_elev: f32, _ray: SurfaceSightRay, _engage_engine: bool,
    ) -> Option<VehicleAssembly> {
        None
    }

    /// Deposits a driven vehicle assembly back into the terrain grid on dismount.
    fn park_vehicle_at(&mut self, _frame: &SurfaceFrame, _assembly: VehicleAssembly) -> (i32, i32) {
        (0, 0)
    }
}

/// Zoned planetary relief evaluating continuous spatial frequency into stepped voxel strata.
pub struct ZonedRelief {
    frequency: Box<dyn SpatialFrequency>,
    frame: SurfaceFrame,
    half_size: f32,
    bounds: (i32, i32),
}

impl ZonedRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        frequency: Box<dyn SpatialFrequency>,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        half_size: f32,
        min_elevation: i32,
        max_elevation: i32,
    ) -> Self {
        Self {
            frequency,
            frame: SurfaceFrame::new(tangent, bitangent, normal, half_size),
            half_size,
            bounds: (min_elevation, max_elevation),
        }
    }

    fn sample_at(&self, u: f32, v: f32) -> f32 {
        self.frequency.sample(self.frame.point_at(u, v, 0.0))
    }
}

impl ReliefElevation for ZonedRelief {
    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        let wu = self.frame.step_elevation(u);
        let wv = self.frame.step_elevation(v);
        let r = wu.hypot(wv);
        let spawn_range = if self.half_size >= 48.0 {
            16.0
        } else if self.half_size >= 24.0 {
            10.0
        } else {
            4.0
        };
        let spawn_weight = ((r - 2.5) / spawn_range).clamp(0.0, 1.0);
        let spawn_smooth = spawn_weight * spawn_weight * (3.0 - 2.0 * spawn_weight);

        let edge_dist = (self.half_size - wu.abs()).min(self.half_size - wv.abs());
        let border_weight = ((edge_dist - 1.5) / 8.0).clamp(0.0, 1.0);
        let border_smooth = border_weight * border_weight * (3.0 - 2.0 * border_weight);

        let weight = spawn_smooth * border_smooth;
        if weight <= 0.0 {
            return 0;
        }

        let s = self.sample_at(wu, wv);
        let normalized = (s + 1.0) * 0.5;
        let (min_elev, max_elev) = self.bounds;
        let range = (max_elev - min_elev) as f32;
        let raw = (normalized * range * weight).round() as i32;
        raw.clamp(min_elev, max_elev)
    }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        let cell = GridCell::at_planar(u, v);
        let (cell_u, cell_v) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let (frac_u, frac_v) = cell.local_fraction(u, v);
        NaturalCrown::at(cell_u, cell_v, |gu, gv| self.elevation_at(gu, gv))
            .surface_height(self.elevation_at(cell_u, cell_v), frac_u, frac_v)
    }
}
