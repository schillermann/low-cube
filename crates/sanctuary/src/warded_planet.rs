use glam::Vec3;
use world::{
    BlockMedium, BlockMold, BodyId, CleaveOutcome, CraftBlueprint, CraftIdentity, CraftPose,
    ExposedStratum, SurfaceFind, WorldEarthworks, WorldFact,
};
use crate::fleet_ledger::FleetLedger;
use crate::ward::GuestWard;

/// Authoritative `WorldEarthworks` decorator enforcing per-companion reach, cadence tokens, block credits, and helm locks.
pub struct WardedPlanet<'a> {
    origin: &'a mut dyn WorldEarthworks,
    ward: &'a mut GuestWard,
    fleet: &'a mut FleetLedger,
    slot: u32,
}

impl<'a> WardedPlanet<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: &'a mut dyn WorldEarthworks,
        ward: &'a mut GuestWard,
        fleet: &'a mut FleetLedger,
        slot: u32,
    ) -> Self {
        Self { origin, ward, fleet, slot }
    }
}

impl WorldEarthworks for WardedPlanet<'_> {
    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        if !self.ward.eye_within_reach(eye, direction) {
            return None;
        }
        self.origin.loosen_surface(eye, direction, stage.min(4))
    }

    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        if !self.ward.eye_within_reach(eye, direction) || !self.ward.consume_harvest_token() {
            return None;
        }
        let medium = self.origin.harvest_surface(eye, direction)?;
        self.ward.credit_blocks(1);
        Some(medium)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        if !self.ward.eye_within_reach(eye, direction) {
            return CleaveOutcome::Missed;
        }
        let outcome = self.origin.strike_timber(eye, direction);
        if outcome.felled() {
            self.ward.credit_blocks(8);
        }
        outcome
    }

    fn strike_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        if !self.ward.eye_within_reach(eye, direction) {
            return (false, false);
        }
        self.origin.strike_boar(eye, direction, power.clamp(1, 4), max_reach.clamp(0.5, 2.75))
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        if !self.ward.eye_within_reach(eye, direction) {
            return SurfaceFind::EmptyHanded;
        }
        let find = self.origin.forage_surface(eye, direction);
        if find.gathered() {
            self.ward.credit_blocks(4);
        }
        find
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        if !find.gathered() || !self.ward.eye_within_reach(eye, direction) || !self.ward.spend_block() {
            return false;
        }
        if self.origin.cast_surface_find(eye, direction, find) {
            return true;
        }
        self.ward.credit_blocks(1);
        false
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        if !self.ward.eye_within_reach(eye, direction) || !self.ward.spend_block() {
            return false;
        }
        if self.origin.place_molded_surface_block(eye, direction, avatar_pos, mold) {
            return true;
        }
        self.ward.credit_blocks(1);
        false
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.ward.eye_within_reach(eye, direction)
            && self.origin.adjust_rig(eye, direction, delta_steps.clamp(-1, 1), rotate_quarter)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.ward.eye_within_reach(eye, direction) && self.origin.lower_rig(eye, direction)
    }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        let needed = blueprint.part_count() as u32;
        if needed == 0 || !self.ward.eye_within_reach(eye, direction) || !self.ward.spend_blocks(needed) {
            return false;
        }
        if self.origin.build_rig_blueprint(eye, direction, blueprint) {
            return true;
        }
        self.ward.credit_blocks(needed);
        false
    }

    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        blueprint.drivable() && self.fleet.register_blueprint(blueprint)
    }

    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, _pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        if !self.ward.eye_within_reach(eye, direction) {
            return false;
        }
        if craft.assigned() && !self.fleet.claim_helm(craft, self.slot) {
            return false;
        }
        let Some(assembly) = self.origin.board_vehicle(eye, direction, true) else {
            if craft.assigned() {
                let _ = self.fleet.release_helm(craft, self.slot);
            }
            return false;
        };
        if assembly.identity().assigned() {
            let _ = self.fleet.register_blueprint(assembly.blueprint().clone());
        }
        true
    }

    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        if !self.ward.pose_within_reach(pose) || !self.fleet.helmed_by(craft, self.slot) {
            return false;
        }
        self.fleet.park_by_id(craft, pose, self.origin)
    }

    fn disclose_stratum(&mut self, _face_index: usize, _stratum: ExposedStratum) {}

    fn offworld_frame(&self, probe: Vec3) -> Option<(BodyId, Vec3)> {
        self.origin.offworld_frame(probe)
    }

    fn body_center(&self, body: BodyId) -> Option<Vec3> {
        self.origin.body_center(body)
    }

    fn enact_offworld(&mut self, body: BodyId, fact: &WorldFact) -> bool {
        let Some(c) = self.origin.body_center(body) else {
            return false;
        };
        match *fact {
            WorldFact::Loosened { eye, direction, stage } => {
                self.ward.eye_within_reach(eye + c, direction)
                    && self.origin.enact_offworld(body, &WorldFact::Loosened { eye, direction, stage: stage.min(4) })
            }
            WorldFact::Harvested { eye, direction } => {
                if !self.ward.eye_within_reach(eye + c, direction) || !self.ward.consume_harvest_token() {
                    return false;
                }
                let ok = self.origin.enact_offworld(body, fact);
                if ok {
                    self.ward.credit_blocks(1);
                }
                ok
            }
            WorldFact::MoldPlaced { eye, direction, .. } => {
                if !self.ward.eye_within_reach(eye + c, direction) || !self.ward.spend_block() {
                    return false;
                }
                if self.origin.enact_offworld(body, fact) {
                    true
                } else {
                    self.ward.credit_blocks(1);
                    false
                }
            }
            WorldFact::LitterCast { eye, direction, find } => {
                if !find.gathered() || !self.ward.eye_within_reach(eye + c, direction) || !self.ward.spend_block() {
                    return false;
                }
                if self.origin.enact_offworld(body, fact) {
                    true
                } else {
                    self.ward.credit_blocks(1);
                    false
                }
            }
            WorldFact::RigAdjusted { eye, direction, delta_steps, rotate_quarter } => {
                self.ward.eye_within_reach(eye + c, direction)
                    && self.origin.enact_offworld(
                        body,
                        &WorldFact::RigAdjusted { eye, direction, delta_steps: delta_steps.clamp(-1, 1), rotate_quarter },
                    )
            }
            WorldFact::RigLowered { eye, direction } => {
                self.ward.eye_within_reach(eye + c, direction) && self.origin.enact_offworld(body, fact)
            }
            WorldFact::RigBlueprintBuilt { eye, direction, ref blueprint } => {
                let needed = blueprint.part_count() as u32;
                if needed == 0 || !self.ward.eye_within_reach(eye + c, direction) || !self.ward.spend_blocks(needed) {
                    return false;
                }
                if self.origin.enact_offworld(body, fact) {
                    true
                } else {
                    self.ward.credit_blocks(needed);
                    false
                }
            }
            WorldFact::HelmClaimed { craft, pilot_slot, eye, direction } => {
                self.claim_vehicle_helm(craft, pilot_slot, eye + c, direction)
            }
            WorldFact::HelmReleased { craft, pose } => {
                self.park_craft_by_id(craft, pose.with_origin(pose.origin() + c))
            }
            _ => false,
        }
    }
}
