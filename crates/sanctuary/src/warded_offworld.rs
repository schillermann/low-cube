use glam::Vec3;
use world::{BodyId, WorldEarthworks, WorldFact};
use crate::fleet_ledger::FleetLedger;
use crate::ward::GuestWard;
use crate::warded_planet::WardedPlanet;

/// Collaborator validating and enacting body-local `OffWorld` facts under `GuestWard` anti-cheat rules (`4` fields).
pub(super) struct OffworldWard<'a> {
    origin: &'a mut dyn WorldEarthworks,
    ward: &'a mut GuestWard,
    fleet: &'a mut FleetLedger,
    slot: u32,
}

impl<'a> OffworldWard<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        origin: &'a mut dyn WorldEarthworks,
        ward: &'a mut GuestWard,
        fleet: &'a mut FleetLedger,
        slot: u32,
    ) -> Self {
        Self { origin, ward, fleet, slot }
    }

    pub(super) fn enact(self, body: BodyId, fact: &WorldFact) -> bool {
        let c = if self.ward.cosmic_frame().anchored_body() == Some(body) {
            Vec3::ZERO
        } else if let Some(center) = self.origin.body_center(body) {
            center
        } else {
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
                let Some(medium) = self.origin.harvest_offworld(body, eye, direction) else {
                    return false;
                };
                self.ward.credit_blocks(1);
                self.ward.manifest_mut().credit_medium(medium);
                true
            }
            WorldFact::MoldPlaced { eye, direction, mold, .. } => {
                if !self.ward.eye_within_reach(eye + c, direction) {
                    return false;
                }
                let residue = mold.medium().flight_residue() && self.ward.piloting_craft();
                if !residue && !self.ward.spend_block() {
                    return false;
                }
                if self.origin.enact_offworld(body, fact) {
                    if !residue {
                        self.ward.manifest_mut().deduct_medium_if_present(mold.medium());
                    }
                    true
                } else {
                    if !residue {
                        self.ward.credit_blocks(1);
                    }
                    false
                }
            }
            WorldFact::ModulePlaced { eye, direction, spec, .. } => {
                if !self.ward.eye_within_reach(eye + c, direction) || !self.ward.spend_block() {
                    return false;
                }
                if self.origin.enact_offworld(body, fact) {
                    self.ward.manifest_mut().deduct_medium_if_present(spec.medium());
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
                    self.ward.manifest_mut().deduct_find_if_present(find);
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
                    self.ward.manifest_mut().deduct_blueprint_if_present(blueprint);
                    true
                } else {
                    self.ward.credit_blocks(needed);
                    false
                }
            }
            WorldFact::HelmClaimed { craft, pilot_slot, eye, direction } => {
                if self.ward.cosmic_frame().anchored_body() == Some(body) {
                    if !self.ward.eye_within_reach(eye, direction) {
                        return false;
                    }
                    if craft.assigned() && !self.fleet.claim_helm(craft, self.slot) {
                        return false;
                    }
                    let ok = self.origin.enact_offworld(body, fact);
                    if !ok && craft.assigned() {
                        let _ = self.fleet.release_helm(craft, self.slot);
                    }
                    ok
                } else {
                    WardedPlanet::new(self.origin, self.ward, self.fleet, self.slot)
                        .claim_vehicle_helm(craft, pilot_slot, eye + c, direction)
                }
            }
            WorldFact::HelmReleased { craft, pose } => {
                if self.ward.cosmic_frame().anchored_body() == Some(body) {
                    if !self.ward.pose_within_reach(pose) || !self.fleet.helmed_by(craft, self.slot) {
                        return false;
                    }
                    if let Some(assembly) = self.fleet.reconstruct_assembly(craft, pose) {
                        let _ = self.origin.enact_offworld(
                            body,
                            &WorldFact::CraftCommissioned { blueprint: assembly.blueprint().clone() },
                        );
                    }
                    self.fleet.force_release(craft);
                    self.origin.enact_offworld(body, fact)
                } else {
                    WardedPlanet::new(self.origin, self.ward, self.fleet, self.slot)
                        .park_craft_by_id(craft, pose.with_origin(pose.origin() + c))
                }
            }
            _ => false,
        }
    }
}
