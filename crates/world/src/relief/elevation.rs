use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, MiningEdge, VoxelShape};
use crate::frame::SurfaceFrame;
use super::craft_entity::{CraftBlueprint, VesselSink};
use super::mineral_vein::ExposedStratum;
use super::planned_mount::MountDomain;
use super::sight_ray::SurfaceSightRay;
use super::vehicle_assembly::VehicleAssembly;
pub use super::zoned::ZonedRelief;

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

    /// Indicates whether an unoccupied cantilever crane boom slot sits at (u, v, layer_elev).
    fn boom_slot_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }

    /// Resolves the effective cell and base elevation where a non-crane block aimed at (u, v, base) will mount.
    fn effective_placement_cell(&self, u: i32, v: i32, base: f32) -> Option<(i32, i32, f32)> {
        Some((u, v, base))
    }

    /// Resolves the verified target cell, final floor elevation, and construction domain for a candidate block.
    fn resolve_planned_site(&self, u: i32, v: i32, base: f32, _medium: BlockMedium) -> Option<(GridCell, f32, MountDomain)> {
        Some((GridCell::new(u, v), base, MountDomain::WorldCube))
    }

    /// Presents parked surface vessels to a GPU vessel sink.
    fn present_vessels(&self, _sink: &mut dyn VesselSink) {}

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

    /// Marks or clears partial digging progress (0..=3) at a specific vertical layer within (u, v).
    fn mark_dig_layer(&mut self, _u: i32, _v: i32, _layer_elev: f32, _stage: u8) -> Option<BlockMedium> { None }

    /// Indicates whether the targeted block layer at (u, v) yields to `edge`.
    fn layer_yields_to(&self, _u: i32, _v: i32, _layer_elev: f32, _edge: MiningEdge) -> bool { true }

    /// Excavates a specific vertical block layer within (u, v).
    fn excavate_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.harvest_layer(u, v, layer_elev).is_some()
    }

    /// Excavates a specific vertical block layer within (u, v), returning the harvested block medium.
    fn harvest_layer(&mut self, _u: i32, _v: i32, _layer_elev: f32) -> Option<BlockMedium> {
        None
    }

    /// Stacks one molded voxel block layer at (u, v).
    fn stack_molded_cell(&mut self, _u: i32, _v: i32, _base: f32, _shape: Box<dyn VoxelShape>) -> bool { false }

    /// Raises, lowers, or rotates a polyhedral cantilever rig at or holding (u, v).
    fn adjust_rig_at(&mut self, _u: i32, _v: i32, _delta_steps: i32, _rotate_quarter: bool) -> (bool, (i32, i32)) { (false, (0, 0)) }

    /// Tripod hub position, base forward axis, and up axis of a placed field camera, if present.
    fn placed_camera_tripod(&self, _frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { None }

    /// Collects world-space `(ceiling_point, surface_normal)` sites atop placed blocks of `medium`.
    fn collect_medium_sites(&self, _medium: BlockMedium, _frame: &SurfaceFrame, _out: &mut Vec<(Vec3, Vec3)>) {}

    /// Lowers a polyhedral cantilever rig assembly onto the ground when its mast at (u, v) is clicked.
    fn lower_rig_at(&mut self, _u: i32, _v: i32) -> bool { false }

    /// Boards a cabin seat or engages a car engine at (u, v, layer_elev), returning `(assembly, terrain_changed)`.
    fn board_vehicle_on_face(
        &mut self, frame: &SurfaceFrame, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, engage_engine: bool,
    ) -> (Option<VehicleAssembly>, bool) {
        (self.board_vehicle_at(frame, u, v, layer_elev, ray, engage_engine), false)
    }

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

    /// Captures a non-destructive blueprint from an aimed crane craft or parked vehicle.
    fn capture_blueprint_at(
        &self,
        _frame: &SurfaceFrame,
        _u: i32,
        _v: i32,
        _layer_elev: f32,
        _ray: SurfaceSightRay,
    ) -> Option<CraftBlueprint> {
        None
    }

    /// Indicates whether the sight ray targets an empty cantilever crane arm ready for blueprint construction.
    fn aimed_empty_rig_at(
        &self,
        _frame: &SurfaceFrame,
        _u: i32,
        _v: i32,
        _base: f32,
        _ray: SurfaceSightRay,
    ) -> bool {
        false
    }

    /// Constructs `blueprint` onto an aimed empty cantilever crane arm, returning the mast cell to refresh.
    fn build_rig_blueprint_at(
        &mut self,
        _frame: &SurfaceFrame,
        _u: i32,
        _v: i32,
        _base: f32,
        _ray: SurfaceSightRay,
        _blueprint: &CraftBlueprint,
    ) -> Option<(i32, i32)> {
        None
    }

    /// Air-exposed subterranean mineral ore at `(u, v, step)`, if uncovered.
    fn exposed_ore_at(&self, _u: i32, _v: i32, _step: i32) -> Option<BlockMedium> {
        None
    }

    /// Applies an authoritative sanctuary stratum disclosure onto a client relief face.
    fn accept_disclosure(&mut self, _stratum: ExposedStratum) {}

    /// Drains newly uncovered subterranean strata produced since the previous drain.
    fn drain_disclosures(&mut self, _out: &mut Vec<ExposedStratum>) {}

    /// Advances grass regrowth and returns `(refreshed_cells, simulation_still_active)`.
    fn step_grass_activity(&mut self, dt: f32) -> (Vec<(i32, i32)>, bool) {
        let cells = self.step_grass(dt);
        let active = !cells.is_empty();
        (cells, active)
    }

    /// Captures raycast-free `(bare_mask, SectorColumnState)` for `4m × 4m` sector `(gsu, gsv)`.
    fn capture_sector_state(&self, _gsu: i32, _gsv: i32) -> (u64, crate::SectorColumnState) {
        (0, crate::SectorColumnState::untouched())
    }

    /// Applies raycast-free `(bare_mask, SectorColumnState)` onto `4m × 4m` sector `(gsu, gsv)`.
    fn apply_sector_state(&mut self, _gsu: i32, _gsv: i32, _bare_mask: u64, _cols: &crate::SectorColumnState) {}

    /// Appends all modified `(gsu, gsv)` `4m × 4m` sector keys on this relief surface.
    fn collect_modified_sectors(&self, _out: &mut Vec<(i32, i32)>) {}
}
