use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use super::super::cantilever_rig::{RigPlacement, RigRegistry};
use super::super::craft_cluster::ParkedFleet;
use super::super::craft_entity::CraftBlueprint;
use super::super::molded_structure::MoldedStructure;
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
    fleet: &'a ParkedFleet,
}

impl<'a> BlueprintSite<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(rigs: &'a RigRegistry, fleet: &'a ParkedFleet) -> Self {
        Self { rigs, fleet }
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
            .capture_blueprint(u, v, layer_elev, eu, ev, frame)
            .or_else(|| self.fleet.capture_blueprint_at(u, v, layer_elev, ray))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn aimed_empty_rig(
        &self,
        u: i32,
        v: i32,
        ground_top: f32,
        sub: bool,
        raw_floor: f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> bool {
        self.rigs.resolve_empty_rig(u, v, ground_top, sub, raw_floor, ray, frame).is_some()
    }
}

/// Collaborator assembling a saved `CraftBlueprint` onto an aimed empty crane rig and clearing turf scars.
pub(super) struct BlueprintCommission<'a> {
    rigs: &'a mut RigRegistry,
    scars: &'a mut TurfScars,
}

impl<'a> BlueprintCommission<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(rigs: &'a mut RigRegistry, scars: &'a mut TurfScars) -> Self {
        Self { rigs, scars }
    }

    #[allow(clippy::too_many_arguments)]
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
        let mast = self.rigs.resolve_empty_rig(u, v, ground_top, sub, raw_floor, ray, frame)?;
        let final_mast = self.rigs.build_blueprint(mast, blueprint, strata, shafts, frame)?;
        self.scars.clear_stage(final_mast.0, final_mast.1);
        Some(final_mast)
    }
}

/// Collaborator resolving crane/fleet vehicle boarding and seated crane re-docking (`<= 4` fields).
pub(super) struct VehicleEmbarkation<'a> {
    rigs: &'a mut RigRegistry,
    fleet: &'a mut ParkedFleet,
}

impl<'a> VehicleEmbarkation<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(rigs: &'a mut RigRegistry, fleet: &'a mut ParkedFleet) -> Self {
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
        if let Some(assembly) = self.rigs.board_drivable(strata, shafts, u, v, layer_elev, ray, frame) {
            return (Some(assembly), true);
        }
        if let Some(assembly) = self.fleet.board_drivable(u, v, layer_elev, ray) {
            return (Some(assembly), false);
        }
        let Some(mast) = self.rigs.find_aimed_rig(u, v, ray, frame) else {
            return (None, false);
        };
        let mounted = self.rigs.reconnect_seated_car(strata, shafts, mast, ray, frame, self.fleet);
        let changed = mounted.is_some();
        (mounted, changed)
    }
}
