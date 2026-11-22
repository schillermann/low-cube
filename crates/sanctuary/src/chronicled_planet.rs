use glam::Vec3;
use world::{
    BlockMedium, BlockMold, BodyId, CleaveOutcome, CraftBlueprint, CraftIdentity, CraftPose,
    ExposedStratum, SurfaceFind, VehicleAssembly, WorldEarthworks, WorldFact,
};

/// Focused contract absorbing locally enacted world mutations and fleet ownership into a chronicle or petition stream.
pub trait FactJournal: Send {
    fn inscribe_fact(&mut self, fact: WorldFact);
    fn local_slot(&self) -> u32 { 1 }
    fn register_local_blueprint(&mut self, _blueprint: CraftBlueprint) -> bool { true }
    fn claim_local_helm(&mut self, _craft: CraftIdentity, _pilot_slot: u32) -> bool { true }
    fn release_local_helm(&mut self, _craft: CraftIdentity) {}
}

/// Focused `WorldEarthworks` decorator recording successful local surface mutations and helm claims into a `FactJournal`.
pub struct ChronicledPlanet<'a> {
    origin: &'a mut dyn WorldEarthworks,
    journal: &'a mut dyn FactJournal,
}

impl<'a> ChronicledPlanet<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: &'a mut dyn WorldEarthworks, journal: &'a mut dyn FactJournal) -> Self {
        Self { origin, journal }
    }

    fn record_scoped(&mut self, probe: Vec3, fact: WorldFact) {
        let scoped = fact.scoped_to(self.origin.offworld_frame(probe));
        self.journal.inscribe_fact(scoped);
    }
}

impl WorldEarthworks for ChronicledPlanet<'_> {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let medium = self.origin.harvest_surface(eye, direction)?;
        self.record_scoped(eye, WorldFact::Harvested { eye, direction });
        Some(medium)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let medium = self.origin.loosen_surface(eye, direction, stage)?;
        self.record_scoped(eye, WorldFact::Loosened { eye, direction, stage });
        Some(medium)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        let out = self.origin.strike_timber(eye, direction);
        if out.struck() {
            self.journal.inscribe_fact(WorldFact::TimberStruck { eye, direction });
        }
        out
    }

    fn strike_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        let out = self.origin.strike_boar(eye, direction, power, max_reach);
        if out.0 {
            let reach_cm = (max_reach.clamp(0.0, 10.0) * 100.0).round() as u16;
            self.journal.inscribe_fact(WorldFact::BoarStruck { eye, direction, power, reach_cm });
        }
        out
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        let find = self.origin.forage_surface(eye, direction);
        if find.gathered() {
            self.journal.inscribe_fact(WorldFact::Foraged { eye, direction });
        }
        find
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        let ok = self.origin.cast_surface_find(eye, direction, find);
        if ok {
            self.record_scoped(eye, WorldFact::LitterCast { eye, direction, find });
        }
        ok
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let ok = self.origin.place_molded_surface_block(eye, direction, avatar_pos, mold);
        if ok {
            self.record_scoped(eye, WorldFact::MoldPlaced { eye, direction, avatar_pos, mold });
        }
        ok
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        let ok = self.origin.adjust_rig(eye, direction, delta_steps, rotate_quarter);
        if ok {
            self.record_scoped(eye, WorldFact::RigAdjusted { eye, direction, delta_steps, rotate_quarter });
        }
        ok
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let ok = self.origin.lower_rig(eye, direction);
        if ok {
            self.record_scoped(eye, WorldFact::RigLowered { eye, direction });
        }
        ok
    }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        let ok = self.origin.build_rig_blueprint(eye, direction, blueprint);
        if ok {
            self.record_scoped(
                eye,
                WorldFact::RigBlueprintBuilt { eye, direction, blueprint: blueprint.clone() },
            );
        }
        ok
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        let assembly = self.origin.board_vehicle(eye, direction, holding_block)?;
        if holding_block {
            let slot = self.journal.local_slot();
            if assembly.identity().assigned() {
                if self.journal.register_local_blueprint(assembly.blueprint().clone()) {
                    self.journal.inscribe_fact(WorldFact::CraftCommissioned {
                        blueprint: assembly.blueprint().clone(),
                    });
                }
                if !self.journal.claim_local_helm(assembly.identity(), slot) {
                    let _ = self.origin.park_vehicle(assembly);
                    return None;
                }
            }
            self.record_scoped(
                eye,
                WorldFact::HelmClaimed {
                    craft: assembly.identity(),
                    pilot_slot: slot,
                    eye,
                    direction,
                },
            );
        }
        Some(assembly)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        let id = assembly.identity();
        let pose = assembly.pose();
        let bp = assembly.blueprint().clone();
        let ok = self.origin.park_vehicle(assembly);
        if ok && id.assigned() {
            if self.journal.register_local_blueprint(bp.clone()) {
                self.journal.inscribe_fact(WorldFact::CraftCommissioned { blueprint: bp });
            }
            self.journal.release_local_helm(id);
            self.record_scoped(pose.origin(), WorldFact::HelmReleased { craft: id, pose });
        }
        ok
    }

    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        let ok = self.origin.park_craft_by_id(craft, pose);
        if ok {
            self.journal.release_local_helm(craft);
            self.record_scoped(pose.origin(), WorldFact::HelmReleased { craft: id_from(craft), pose });
        }
        ok
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.origin.disclose_stratum(face_index, stratum);
    }

    fn offworld_frame(&self, probe: Vec3) -> Option<(BodyId, Vec3)> {
        self.origin.offworld_frame(probe)
    }

    fn body_center(&self, body: BodyId) -> Option<Vec3> {
        self.origin.body_center(body)
    }

    fn enact_offworld(&mut self, body: BodyId, fact: &WorldFact) -> bool {
        let ok = self.origin.enact_offworld(body, fact);
        if ok {
            self.journal.inscribe_fact(WorldFact::OffWorld { body, fact: Box::new(fact.clone()) });
        }
        ok
    }
}

const fn id_from(craft: CraftIdentity) -> CraftIdentity {
    craft
}
