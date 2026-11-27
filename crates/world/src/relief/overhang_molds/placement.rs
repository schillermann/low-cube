use crate::block::{BlockMedium, CardinalTurn, GridCell};
use crate::brick_arena::{CellCoord, ModuleSpec, RawAtom};
use super::super::cantilever_rig::{RigPlacement, RigRegistry};
use super::super::craft_cluster::ParkedFleet;
use super::super::molded_structure::WorldBlocks;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::turf_scars::TurfScars;
use super::blueprints::BlueprintSite;
use super::MAX_STACK_ELEVATION;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Collaborator routing atom/module placement and layer removal across world bricks and cantilever rigs (`4` fields).
pub(super) struct MoldStack<'a> {
    world_blocks: &'a mut WorldBlocks,
    fleet: &'a ParkedFleet,
    rigs: &'a mut RigRegistry,
    scars: &'a mut TurfScars,
}

impl<'a> MoldStack<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        world_blocks: &'a mut WorldBlocks,
        fleet: &'a ParkedFleet,
        rigs: &'a mut RigRegistry,
        scars: &'a mut TurfScars,
    ) -> Self {
        Self { world_blocks, fleet, rigs, scars }
    }

    pub(super) fn remove_layer(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        u: i32,
        v: i32,
        layer_elev: f32,
    ) -> Option<BlockMedium> {
        if let Some(med) = self.rigs.remove_craft_layer(u, v, layer_elev) {
            self.scars.clear_stage(u, v);
            return Some(med);
        }
        if self.rigs.pack_up(strata, shafts, u, v, layer_elev, self.world_blocks, self.scars) {
            self.scars.clear_stage(u, v);
            return Some(BlockMedium::CantileverRig);
        }
        let ground_top = (shafts.elevation_at(strata, u, v) as f32) * BLOCK_STEP;
        self.world_blocks.remove_layer(u, v, layer_elev, ground_top, self.scars)
    }

    pub(super) fn stack_atom(
        self,
        (u, v): (i32, i32),
        (ground_top, raw_floor): (f32, f32),
        sub: bool,
        atom: RawAtom,
    ) -> bool {
        if atom.medium() == BlockMedium::VehicleBlueprint {
            return false;
        }
        if atom.medium() == BlockMedium::CantileverRig {
            self.rigs.deploy(u, v, ground_top);
            self.scars.clear_stage(u, v);
            return true;
        }
        if let Some(spec) = ModuleSpec::from_medium_turn(atom.medium(), CardinalTurn::from_code(atom.heading().code())) {
            return self.stack_module((u, v), (ground_top, raw_floor), sub, spec);
        }
        match self.rigs.resolve_placement(u, v, ground_top, sub, raw_floor) {
            RigPlacement::BlockedMast => false,
            RigPlacement::MountFirst { mast, cell, floor_elev } => {
                self.rigs.place_on_rig(mast, cell, floor_elev, atom);
                self.scars.clear_stage(cell.0, cell.1);
                true
            }
            RigPlacement::AttachCraft { mast, cell, floor_elev } => {
                if floor_elev >= MAX_STACK_ELEVATION {
                    return false;
                }
                self.rigs.place_on_rig(mast, cell, floor_elev, atom);
                self.scars.clear_stage(cell.0, cell.1);
                true
            }
            RigPlacement::WorldCube => {
                if atom.medium().indestructible() {
                    self.world_blocks.stack_bare(u, v, raw_floor, ground_top, sub, atom, self.scars);
                    return true;
                }
                let site = BlueprintSite::new(self.rigs, self.fleet);
                let Some(next_floor) = site.resolve_world_floor(self.world_blocks, (u, v), (ground_top, raw_floor), sub) else {
                    return false;
                };
                self.world_blocks.stack_bare(u, v, next_floor, ground_top, sub, atom, self.scars);
                self.scars.clear_stage(u, v);
                true
            }
        }
    }

    pub(super) fn stack_module(
        self,
        (u, v): (i32, i32),
        (ground_top, raw_floor): (f32, f32),
        sub: bool,
        spec: ModuleSpec,
    ) -> bool {
        match self.rigs.resolve_placement(u, v, ground_top, sub, raw_floor) {
            RigPlacement::BlockedMast => false,
            RigPlacement::MountFirst { mast, cell, floor_elev }
            | RigPlacement::AttachCraft { mast, cell, floor_elev } => {
                if floor_elev >= MAX_STACK_ELEVATION {
                    return false;
                }
                let ok = self.rigs.place_module_on_rig(mast, cell, floor_elev, spec);
                if ok {
                    self.scars.clear_stage(cell.0, cell.1);
                }
                ok
            }
            RigPlacement::WorldCube => {
                let site = BlueprintSite::new(self.rigs, self.fleet);
                let Some(next_floor) = site.resolve_world_floor(self.world_blocks, (u, v), (ground_top, raw_floor), sub) else {
                    return false;
                };
                let anchor = CellCoord::from_floor(u, v, next_floor, 0.0);
                self.world_blocks.place_module(anchor, spec, ground_top, self.scars)
            }
        }
    }
}
