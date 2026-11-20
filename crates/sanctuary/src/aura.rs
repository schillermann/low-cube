use std::net::SocketAddr;
use world::{CompanionPresence, DrivenCraft, PilotCraft};
use crate::envelope::CrackNotice;
use crate::fleet_ledger::FleetLedger;
use crate::seal::HandshakeIdentity;

/// Cryptographic identity, grid resolution, and advertised P2P endpoint of a `SanctuaryHost`.
pub struct HostIdentity {
    keys: HandshakeIdentity,
    blocks_per_edge: u32,
    endpoint: SocketAddr,
}

impl HostIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(keys: HandshakeIdentity, blocks_per_edge: u32, endpoint: SocketAddr) -> Self {
        Self { keys, blocks_per_edge, endpoint }
    }

    pub const fn keys(&self) -> &HandshakeIdentity { &self.keys }
    pub const fn blocks_per_edge(&self) -> u32 { self.blocks_per_edge }
    pub const fn endpoint(&self) -> SocketAddr { self.endpoint }
}

/// Polymorphic local-host avatar presence (`Headless` for dedicated servers, `Occupied` for `--host`).
#[derive(Clone, Copy, Debug)]
pub enum HostPresence {
    Headless,
    Occupied(CompanionPresence),
}

/// Author-tagged transient dig crack preventing echo feedback to its originating companion.
#[derive(Clone, Copy, Debug)]
pub struct PeerCrack {
    author_slot: u32,
    notice: CrackNotice,
}

impl PeerCrack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(author_slot: u32, notice: CrackNotice) -> Self {
        Self { author_slot, notice }
    }

    pub const fn authored_by(self, slot: u32) -> bool {
        self.author_slot == slot
    }

    pub const fn notice(self) -> CrackNotice {
        self.notice
    }
}

/// Strictly encapsulated local host presence, piloted craft, crack queue, and authoritative `FleetLedger`.
pub struct HostAura {
    presence: HostPresence,
    craft: PilotCraft,
    cracks: Vec<PeerCrack>,
    fleet: FleetLedger,
}

impl HostAura {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        presence: HostPresence,
        craft: PilotCraft,
        cracks: Vec<PeerCrack>,
        fleet: FleetLedger,
    ) -> Self {
        Self { presence, craft, cracks, fleet }
    }

    pub const fn headless() -> Self {
        Self::new(HostPresence::Headless, PilotCraft::Pedestrian, Vec::new(), FleetLedger::empty())
    }

    pub fn occupy(&mut self, presence: CompanionPresence) {
        self.presence = HostPresence::Occupied(presence.with_slot(1));
    }

    pub fn pilot(&mut self, craft: PilotCraft) {
        self.craft = craft.with_slot(1);
    }

    pub const fn presence(&self) -> HostPresence {
        self.presence
    }

    pub const fn fleet(&self) -> &FleetLedger {
        &self.fleet
    }

    pub fn fleet_mut(&mut self) -> &mut FleetLedger {
        &mut self.fleet
    }

    pub fn push_host_craft(&self, out: &mut Vec<DrivenCraft>) {
        self.craft.push_active(out);
    }

    pub fn record_crack(&mut self, author_slot: u32, notice: CrackNotice) {
        if self.cracks.len() >= 16 {
            self.cracks.remove(0);
        }
        self.cracks.push(PeerCrack::new(author_slot, notice));
    }

    pub fn drain_cracks(&mut self) -> Vec<PeerCrack> {
        std::mem::take(&mut self.cracks)
    }
}
