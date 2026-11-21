use std::net::SocketAddr;
use glam::Vec3;
use world::{CompanionPresence, DrivenCraft, PilotCraft, PlanetaryWorld};
use crate::envelope::CrackNotice;
use crate::fleet_ledger::FleetLedger;
use crate::host_sounder::HostSounder;
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

/// Strictly encapsulated local host seat, crack queue, `FleetLedger`, and authoritative `HostSounder`.
pub struct HostAura {
    seat: (HostPresence, PilotCraft),
    cracks: Vec<PeerCrack>,
    fleet: FleetLedger,
    wildlife: HostSounder,
}

impl HostAura {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        seat: (HostPresence, PilotCraft),
        cracks: Vec<PeerCrack>,
        fleet: FleetLedger,
        wildlife: HostSounder,
    ) -> Self {
        Self { seat, cracks, fleet, wildlife }
    }

    pub const fn headless() -> Self {
        Self::new(
            (HostPresence::Headless, PilotCraft::Pedestrian),
            Vec::new(),
            FleetLedger::empty(),
            HostSounder::seeded(),
        )
    }

    pub fn occupy(&mut self, presence: CompanionPresence) {
        self.seat.0 = HostPresence::Occupied(presence.with_slot(1));
    }

    pub fn pilot(&mut self, craft: PilotCraft) {
        self.seat.1 = craft.with_slot(1);
    }

    pub const fn presence(&self) -> HostPresence {
        self.seat.0
    }

    pub fn pedestrian_observer(&self, world: &dyn PlanetaryWorld) -> Option<(Vec3, bool)> {
        if matches!(self.seat.1, PilotCraft::Driving(_)) {
            return None;
        }
        let HostPresence::Occupied(p) = self.seat.0 else {
            return None;
        };
        p.transfer(|_, kin, _, _| {
            kin.transfer(|pos, _, _, eye_h, _| {
                world.woodland_habitat(pos).then_some((pos, eye_h > 0.05 && eye_h < 0.62))
            })
        })
    }

    pub const fn fleet(&self) -> &FleetLedger { &self.fleet }
    pub fn fleet_mut(&mut self) -> &mut FleetLedger { &mut self.fleet }
    pub const fn wildlife(&self) -> &HostSounder { &self.wildlife }
    pub fn wildlife_mut(&mut self) -> &mut HostSounder { &mut self.wildlife }

    pub fn push_host_craft(&self, out: &mut Vec<DrivenCraft>) {
        self.seat.1.push_active(out);
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
