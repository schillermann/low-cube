use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::grid_body::{GridBody, GridFleet};
use super::super::cantilever_rig::{RigPlacement, RigRegistry};
use super::super::craft_entity::CraftBlueprint;
use super::super::molded_structure::{MoldedStructure, SanctuaryStructure};
use super::super::natural_strata::NaturalStrata;
use super::super::planned_mount::MountDomain;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::sight_ray::SurfaceSightRay;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::MAX_STACK_ELEVATION;

/// Read-only collaborator resolving blueprint hologram previews, vehicle capture, and empty-rig queries.
pub(super) struct BlueprintSite<'a> {
    rigs: &'a RigRegistry,
    fleet: &'a GridFleet,
}

impl<'a> BlueprintSite<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(rigs: &'a RigRegistry, fleet: &'a GridFleet) -> Self {
        Self { rigs, fleet }
    }

    pub(super) fn resolve_world_floor(
        &self,
        crust_body: &GridBody,
        (u, v): (i32, i32),
        (ground_top, raw_floor): (f32, f32),
        sub: bool,
    ) -> Option<f32> {
        if SanctuaryStructure::new(crust_body).locked_at(u, v, ground_top) {
            return None;
        }
        let mut z = if sub { raw_floor } else { raw_floor.max(ground_top) };
        while crust_body.layer_occupied_at(u, v, z + 0.25) {
            z += if sub { -GridCell::SPAN } else { GridCell::SPAN };
        }
        (z < MAX_STACK_ELEVATION && !self.fleet.blocks_placement_at(u, v, z)).then_some(z)
    }

    pub(super) fn resolve_any_site(
        &self,
        crust_body: &GridBody,
        (u, v): (i32, i32),
        (ground_top, raw_floor): (f32, f32),
        sub: bool,
        medium: crate::block::BlockMedium,
    ) -> Option<(GridCell, f32, MountDomain)> {
        if SanctuaryStructure::new(crust_body).locked_at(u, v, ground_top) {
            return None;
        }
        if medium == crate::block::BlockMedium::CantileverRig {
            return Some((GridCell::new(u, v), ground_top, MountDomain::WorldCube));
        }
        if medium == crate::block::BlockMedium::VehicleBlueprint {
            return self.resolve_planned_site(u, v, ground_top, sub, raw_floor);
        }
        match self.rigs.resolve_placement(u, v, ground_top, sub, raw_floor) {
            RigPlacement::BlockedMast => None,
            RigPlacement::MountFirst { cell, floor_elev, .. } => {
                Some((GridCell::new(cell.0, cell.1), floor_elev, MountDomain::CraftRig))
            }
            RigPlacement::AttachCraft { cell, floor_elev, .. } => (floor_elev < MAX_STACK_ELEVATION)
                .then_some((GridCell::new(cell.0, cell.1), floor_elev, MountDomain::CraftRig)),
            RigPlacement::WorldCube => self
                .resolve_world_floor(crust_body, (u, v), (ground_top, raw_floor), sub)
                .map(|z| (GridCell::new(u, v), z, MountDomain::WorldCube)),
        }
    }

    pub(super) fn resolve_planned_site(
        &self,
        u: i32,
        v: i32,
        ground_top: f32,
        sub: bool,
        raw_floor: f32,
    ) -> Option<(GridCell, f32, MountDomain)> {
        match self.rigs.resolve_placement(u, v, ground_top, sub, raw_floor) {
            RigPlacement::BlockedMast => self
                .rigs
                .present_at(u, v)
                .then_some((GridCell::new(u, v), ground_top, MountDomain::CraftRig)),
            RigPlacement::MountFirst { cell, floor_elev, .. } => {
                Some((GridCell::new(cell.0, cell.1), floor_elev, MountDomain::CraftRig))
            }
            RigPlacement::AttachCraft { cell, floor_elev, .. } => (floor_elev < MAX_STACK_ELEVATION)
                .then_some((GridCell::new(cell.0, cell.1), floor_elev, MountDomain::CraftRig)),
            RigPlacement::WorldCube => self
                .fleet
                .surface_view()
                .present_at(u, v)
                .then_some((GridCell::new(u, v), ground_top, MountDomain::CraftRig)),
        }
    }

    pub(super) fn capture(
        &self,
        frame: &SurfaceFrame,
        u: i32,
        v: i32,
        layer_elev: f32,
        ray: SurfaceSightRay,
    ) -> Option<CraftBlueprint> {
        let (eu, ev) = frame.decompose(ray.sample_point(0.0));
        self.rigs
            .capture_blueprint((u, v, layer_elev), (eu, ev), frame)
            .or_else(|| self.fleet.capture_blueprint_at(u, v, layer_elev, ray))
    }

    pub(super) fn aimed_empty_rig(
        &self,
        (u, v): (i32, i32),
        (ground_top, raw_floor): (f32, f32),
        sub: bool,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> bool {
        self.rigs.resolve_empty_rig((u, v), (ground_top, raw_floor), sub, ray, frame).is_some()
    }
}

/// Collaborator assembling a saved `CraftBlueprint` onto an aimed empty flange / rig and clearing turf scars.
pub(super) struct BlueprintCommission<'a> {
    rigs: &'a mut RigRegistry,
    scars: &'a mut TurfScars,
}

impl<'a> BlueprintCommission<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(rigs: &'a mut RigRegistry, scars: &'a mut TurfScars) -> Self {
        Self { rigs, scars }
    }

    pub(super) fn build(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        cell_floor: (i32, i32, f32, f32),
        sub: bool,
        ray: SurfaceSightRay,
        blueprint: &CraftBlueprint,
    ) -> Option<(i32, i32)> {
        let (u, v, ground_top, raw_floor) = cell_floor;
        let mast = self.rigs.resolve_empty_rig((u, v), (ground_top, raw_floor), sub, ray, frame)?;
        let final_mast = self.rigs.build_blueprint(mast, blueprint, strata, shafts, frame)?;
        self.scars.clear_stage(final_mast.0, final_mast.1);
        Some(final_mast)
    }
}

/// Collaborator resolving flange/fleet boarding, crane-free flange release, and seated re-docking (`2` fields).
pub(super) struct VehicleEmbarkation<'a> {
    rigs: &'a mut RigRegistry,
    fleet: &'a mut GridFleet,
}

impl<'a> VehicleEmbarkation<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(rigs: &'a mut RigRegistry, fleet: &'a mut GridFleet) -> Self {
        Self { rigs, fleet }
    }

    pub(super) fn embark(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        target_cell: (i32, i32, f32),
        ray: SurfaceSightRay,
    ) -> (Option<VehicleAssembly>, bool) {
        let (u, v, layer_elev) = target_cell;
        if let Some(assembly) = self.rigs.board_drivable(strata, shafts, target_cell, ray, frame) {
            return (Some(assembly), true);
        }
        if let Some(assembly) = self.fleet.board_drivable(u, v, layer_elev, ray) {
            return (Some(assembly), false);
        }
        if let Some(assembly) = self.rigs.unlock_magnetic_flange(strata, shafts, (u, v), ray, frame) {
            let (body, _) = GridBody::settled_on_surface(strata, shafts, frame, assembly);
            self.fleet.insert_body(body);
            return (None, true);
        }
        let Some(mast) = self.rigs.find_aimed_rig(u, v, ray, frame) else {
            return (None, false);
        };
        let mounted = self.rigs.reconnect_seated_car(strata, shafts, mast, ray, frame, self.fleet);
        let changed = mounted.is_some();
        (mounted, changed)
    }
}
