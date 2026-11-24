use glam::Vec3;
use world::{CosmicFrame, DrivenCraft, PilotCraft, PlanetaryWorld, WorldFact};
use crate::aura::HostAura;
use crate::chronicle_log::ChronicleStore;

/// Focused custodian docking vehicles abandoned by disconnected companions on Earth or off-world bodies (`<= 4` fields).
pub(super) struct AbandonedDock<'a> {
    chronicle: &'a mut dyn ChronicleStore,
    aura: &'a mut HostAura,
}

impl<'a> AbandonedDock<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(chronicle: &'a mut dyn ChronicleStore, aura: &'a mut HostAura) -> Self {
        Self { chronicle, aura }
    }

    pub fn dock_expired(
        &mut self,
        abandoned: Vec<(u32, CosmicFrame, PilotCraft)>,
        world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let mut parked_any = false;
        for (cid, frame, pilot_craft) in abandoned {
            if let PilotCraft::Driving(dc) = pilot_craft {
                parked_any |= self.park_craft(frame, dc, world);
            }
            self.aura.fleet_mut().release_slot(cid);
        }
        parked_any
    }

    fn park_craft(&mut self, frame: CosmicFrame, dc: DrivenCraft, world: &mut dyn PlanetaryWorld) -> bool {
        match frame.anchored_body() {
            Some(b) if !frame.earth_centered() => {
                if let Some(assembly) = self.aura.fleet().reconstruct_assembly(dc.identity(), dc.pose()) {
                    let _ = world.enact_offworld(
                        b,
                        &WorldFact::CraftCommissioned { blueprint: assembly.blueprint().clone() },
                    );
                }
                self.aura.fleet_mut().force_release(dc.identity());
                let rel = WorldFact::HelmReleased { craft: dc.identity(), pose: dc.pose() };
                let ok = world.enact_offworld(b, &rel);
                if ok {
                    self.chronicle.record(WorldFact::OffWorld { body: b, fact: Box::new(rel) });
                }
                ok
            }
            _ => {
                let ok = self.aura.fleet_mut().park_by_id(dc.identity(), dc.pose(), world);
                if ok {
                    self.chronicle.record(WorldFact::HelmReleased { craft: dc.identity(), pose: dc.pose() });
                }
                ok
            }
        }
    }

    pub fn scope_standby_release(frame: CosmicFrame, dc: DrivenCraft) -> WorldFact {
        let release = WorldFact::HelmReleased { craft: dc.identity(), pose: dc.pose() };
        match frame.anchored_body() {
            Some(b) if !frame.earth_centered() => WorldFact::OffWorld { body: b, fact: Box::new(release) },
            _ => release,
        }
    }

    pub fn valid_boar_reach(frame: CosmicFrame, eye: Vec3, direction: Vec3, ward: &crate::ward::GuestWard) -> bool {
        frame.earth_centered() && ward.eye_within_reach(eye, direction)
    }
}
