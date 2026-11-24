use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use world::{
    CompanionPresence, CraftBlueprint, CraftIdentity, DrivenCraft, PilotCraft, PlanetaryWorld,
    WorldFact,
};
use crate::chronicled_planet::FactJournal;
use crate::fleet_ledger::FleetLedger;
use crate::mailbox::{PortalBeat, PortalMailbox, SocketLoop};
use crate::remote_link::RemoteLink;
use crate::standby_link::StandbyLink;

/// Optimistic guest petition journal decoupling `StandbyLink` UDP heartbeats from `PlanetaryWorld`.
pub struct GuestJournal {
    wire: Arc<Mutex<StandbyLink>>,
    mailbox: Arc<PortalMailbox>,
    standby: SocketLoop,
    ledger: (VecDeque<WorldFact>, FleetLedger),
}

impl GuestJournal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn from_parts(
        wire: Arc<Mutex<StandbyLink>>,
        mailbox: Arc<PortalMailbox>,
        standby: SocketLoop,
        ledger: (VecDeque<WorldFact>, FleetLedger),
    ) -> Self {
        Self { wire, mailbox, standby, ledger }
    }

    pub fn with_fleet(link: RemoteLink, predicted: VecDeque<WorldFact>, fleet: FleetLedger) -> Self {
        let wire = Arc::new(Mutex::new(StandbyLink::wrapping(link)));
        let mailbox = Arc::new(PortalMailbox::active());
        let standby = SocketLoop::detached(Arc::clone(&mailbox));
        Self::from_parts(wire, mailbox, standby, (predicted, fleet))
    }

    pub fn new(link: RemoteLink, predicted: VecDeque<WorldFact>) -> Self {
        Self::with_fleet(link, predicted, FleetLedger::empty())
    }

    pub fn spawn_standby(&mut self) {
        let beat: Arc<Mutex<dyn PortalBeat>> = Arc::clone(&self.wire) as _;
        self.standby = SocketLoop::spawn(beat, Arc::clone(&self.mailbox));
    }

    fn read_link<R>(&self, op: impl FnOnce(&RemoteLink) -> R) -> R {
        let guard = self.wire.lock().expect("StandbyLink mutex poisoned");
        op(guard.link())
    }

    fn write_link<R>(&self, op: impl FnOnce(&mut RemoteLink) -> R) -> R {
        let mut guard = self.wire.lock().expect("StandbyLink mutex poisoned");
        op(guard.link_mut())
    }

    pub fn connected(&self) -> bool { self.read_link(RemoteLink::connected) }
    pub const fn fleet(&self) -> &FleetLedger { &self.ledger.1 }
    pub fn fleet_mut(&mut self) -> &mut FleetLedger { &mut self.ledger.1 }
    pub fn sync_local_presence(&mut self, presence: CompanionPresence) { self.write_link(|l| l.sync_local_presence(presence)); }
    pub fn sync_local_craft(&mut self, craft: PilotCraft) { self.write_link(|l| l.sync_local_craft(craft)); }
    pub fn companions(&self, out: &mut Vec<CompanionPresence>) { self.read_link(|l| l.companions(out)); }
    pub fn crafts(&self, out: &mut Vec<DrivenCraft>) { self.read_link(|l| l.crafts(out)); }
    pub fn assigned_spawn(&self) -> Option<glam::Vec3> { self.read_link(RemoteLink::assigned_spawn) }
    pub fn utter_nearby(&mut self, text: &str) { self.write_link(|l| l.send_utterance(text)); }
    pub fn drain_utterances(&self, out: &mut Vec<world::ProximityUtterance>) { self.write_link(|l| l.drain_utterances(out)); }
    pub fn active_craft_count(&self) -> usize { self.mailbox.active_crafts() }

    pub fn constrain_crafts(&self, position: glam::Vec3, player_height: f32, up: glam::Vec3) -> Option<glam::Vec3> {
        if self.mailbox.active_crafts() == 0 {
            return None;
        }
        let fleet = &self.ledger.1;
        self.read_link(|l| l.roster().constrain_crafts(fleet, position, player_height, up))
    }

    pub fn sample_deck_at(&self, position: glam::Vec3, player_height: f32) -> Option<world::DeckFrame> {
        if self.mailbox.active_crafts() == 0 {
            return None;
        }
        let fleet = &self.ledger.1;
        self.read_link(|l| l.roster().sample_deck_at(fleet, position, player_height))
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        let blend = if self.mailbox.standby_needed(Instant::now()) { 0.0 } else { subtick_secs };
        self.write_link(|l| l.blend_subtick(blend));
    }

    pub fn step_wildlife(&mut self, dt: f32, listener: glam::Vec3, view_dir: glam::Vec3, world: &dyn PlanetaryWorld) -> Option<glam::Vec3> {
        self.write_link(|l| l.step_wildlife(dt, listener, view_dir, world))
    }

    pub fn intercept_boar_strike(&mut self, eye: glam::Vec3, view_dir: glam::Vec3, max_reach: f32, world: &dyn PlanetaryWorld) -> bool {
        self.write_link(|l| l.intercept_boar_strike(eye, view_dir, max_reach, world))
    }

    pub fn emit_wildlife(&self, cone: world::SightCone, world: &dyn PlanetaryWorld, target: &mut dyn world::MeshTarget) {
        self.read_link(|l| l.emit_wildlife(cone, world, target));
    }

    pub fn ticket_string(&self) -> Option<String> { self.read_link(RemoteLink::ticket_string) }
    pub fn phrase_string(&self) -> Option<String> { self.read_link(RemoteLink::phrase_string) }

    pub fn pump(&mut self, dt: f32, world: &mut dyn PlanetaryWorld) -> bool {
        self.mailbox.record_main_beat(Instant::now());
        let (ref mut predicted, ref mut fleet) = self.ledger;
        let (changed, crafts) = {
            let mut guard = self.wire.lock().expect("StandbyLink mutex poisoned");
            guard.pump(dt, world, fleet, predicted)
        };
        self.mailbox.note_active_crafts(crafts);
        changed
    }
}

impl FactJournal for GuestJournal {
    fn inscribe_fact(&mut self, fact: WorldFact) {
        if fact.durable() {
            if self.ledger.0.len() >= 64 {
                let _ = self.ledger.0.pop_front();
            }
            self.ledger.0.push_back(fact.clone());
        }
        self.write_link(|l| l.send_petition(fact));
    }

    fn local_slot(&self) -> u32 {
        self.read_link(RemoteLink::companion_slot)
    }

    fn register_local_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.ledger.1.register_blueprint(blueprint)
    }

    fn claim_local_helm(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.ledger.1.claim_helm(craft, pilot_slot)
    }

    fn release_local_helm(&mut self, craft: CraftIdentity) {
        self.ledger.1.force_release(craft);
    }
}
