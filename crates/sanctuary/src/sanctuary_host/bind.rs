use std::net::{SocketAddr, UdpSocket};
use world::{Callsign, PlanetaryWorld, SolarProportions};
use crate::aura::{HostAura, HostIdentity};
use crate::chronicle_decorators::CompactedChronicle;
use crate::chronicle_log::{ChronicleLog, ChronicleStore};
use crate::cohort::GuestCohort;
use crate::host_state::HostState;
use crate::host_wire::HostWire;
use crate::nat::NatAperture;
use crate::seal::HandshakeIdentity;
use crate::snapshot_chronicle::SnapshottedChronicle;
use crate::survivor_ledger::{MemorySurvivors, SurvivorStore};
use crate::wormhole::{PortalCode, WormholeGate};
use super::SanctuaryHost;

/// Encapsulated host UDP socket bind and initial `HostState` assembly (`<= 4` fields).
pub(super) struct HostSocket<'a> {
    bind_addr: &'a str,
}

impl<'a> HostSocket<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(bind_addr: &'a str) -> Self {
        Self { bind_addr }
    }

    pub(super) fn bind_cosmos_with_identity(
        self,
        host_keys: HandshakeIdentity,
        callsign: Callsign,
        world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
        survivors: Box<dyn SurvivorStore>,
    ) -> std::io::Result<SanctuaryHost> {
        let secret = PortalCode::draw_secret();
        let socket = HostWire::bind_slot_socket(self.bind_addr, secret)?;
        let endpoint = NatAperture::interactive().open_host(&socket);
        let identity = HostIdentity::with_callsign(host_keys, proportions, endpoint, callsign);
        Ok(Self::assemble(socket, secret, identity, world, chronicle, survivors))
    }

    pub(super) fn bind_with_endpoint(
        self,
        advertised: SocketAddr,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
    ) -> std::io::Result<SanctuaryHost> {
        let secret = PortalCode::draw_secret();
        let socket = HostWire::bind_slot_socket(self.bind_addr, secret)?;
        let store = Box::new(SnapshottedChronicle::wrapping(CompactedChronicle::wrapping(ChronicleLog::empty())));
        let identity = HostIdentity::with_callsign(
            HandshakeIdentity::generate(),
            SolarProportions::true_scale(blocks_per_edge),
            advertised,
            Callsign::host(),
        );
        Ok(Self::assemble(socket, secret, identity, world, store, Box::new(MemorySurvivors::empty())))
    }

    fn assemble(
        socket: UdpSocket,
        secret: u8,
        identity: HostIdentity,
        world: Box<dyn PlanetaryWorld>,
        chronicle: Box<dyn ChronicleStore>,
        survivors: Box<dyn SurvivorStore>,
    ) -> SanctuaryHost {
        let code = PortalCode::from_endpoint(identity.endpoint(), secret);
        let mut aura = HostAura::headless();
        let initial_epoch = world.celestial_atlas().ephemeris().map_or(0.0, |e| e.epoch_secs());
        aura.sync_epoch(initial_epoch);
        aura.beacons_mut().refresh_from(world.as_ref());
        let state = HostState::with_store(
            chronicle,
            GuestCohort::with_store(survivors),
            aura,
            WormholeGate::with_code(code),
        );
        SanctuaryHost::new(socket, world, identity, state)
    }
}
