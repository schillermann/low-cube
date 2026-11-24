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
use crate::survivor_ledger::{MemorySurvivors, SurvivorStore};
use crate::wormhole::{PortalCode, WormholeGate};
use super::SanctuaryHost;

impl SanctuaryHost {
    /// Binds a non-blocking UDP socket and discovers the external P2P endpoint via STUN & UPnP/NAT-PMP.
    pub fn bind(bind_addr: &str, world: Box<dyn PlanetaryWorld>, blocks_per_edge: u32) -> std::io::Result<Self> {
        Self::bind_cosmos(bind_addr, world, SolarProportions::true_scale(blocks_per_edge))
    }

    /// Binds a non-blocking UDP socket advertising full `SolarProportions` during guest handshakes.
    pub fn bind_cosmos(
        bind_addr: &str,
        world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions,
    ) -> std::io::Result<Self> {
        let store = Box::new(CompactedChronicle::wrapping(ChronicleLog::empty()));
        Self::bind_cosmos_with_store(bind_addr, world, proportions, store)
    }

    /// Binds a non-blocking UDP socket backed by an explicit `ChronicleStore` decorator chain.
    pub fn bind_cosmos_with_store(
        bind_addr: &str,
        world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
    ) -> std::io::Result<Self> {
        Self::bind_cosmos_with_stores(bind_addr, world, proportions, chronicle, Box::new(MemorySurvivors::empty()))
    }

    /// Binds a non-blocking UDP socket advertising a caller-supplied `Callsign` for in-game P2P co-op hosts.
    pub fn bind_named_cosmos(
        bind_addr: &str,
        callsign: Callsign,
        world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions,
    ) -> std::io::Result<Self> {
        let store = Box::new(CompactedChronicle::wrapping(ChronicleLog::empty()));
        let survivors = Box::new(MemorySurvivors::empty());
        Self::bind_cosmos_with_identity(bind_addr, HandshakeIdentity::generate(), callsign, world, proportions, store, survivors)
    }

    /// Binds a non-blocking UDP socket backed by explicit `ChronicleStore` and `SurvivorStore` decorator chains.
    pub fn bind_cosmos_with_stores(
        bind_addr: &str,
        world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
        survivors: Box<dyn SurvivorStore>,
    ) -> std::io::Result<Self> {
        Self::bind_cosmos_with_identity(bind_addr, HandshakeIdentity::generate(), Callsign::host(), world, proportions, chronicle, survivors)
    }

    /// Binds a non-blocking UDP socket with an explicit `HandshakeIdentity` and host `Callsign`.
    pub fn bind_cosmos_with_identity(
        bind_addr: &str,
        host_keys: HandshakeIdentity,
        callsign: Callsign,
        world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
        survivors: Box<dyn SurvivorStore>,
    ) -> std::io::Result<Self> {
        let secret = PortalCode::draw_secret();
        let socket = HostWire::bind_slot_socket(bind_addr, secret)?;
        let endpoint = NatAperture::interactive().open_host(&socket);
        let identity = HostIdentity::with_callsign(host_keys, proportions, endpoint, callsign);
        Ok(Self::assemble_with_stores(socket, secret, identity, world, chronicle, survivors))
    }

    /// Binds a UDP socket advertising a known external NAT endpoint.
    pub fn bind_with_endpoint(
        bind_addr: &str,
        advertised: SocketAddr,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
    ) -> std::io::Result<Self> {
        let secret = PortalCode::draw_secret();
        let socket = HostWire::bind_slot_socket(bind_addr, secret)?;
        let store = Box::new(CompactedChronicle::wrapping(ChronicleLog::empty()));
        let identity = HostIdentity::with_callsign(
            HandshakeIdentity::generate(),
            SolarProportions::true_scale(blocks_per_edge),
            advertised,
            Callsign::host(),
        );
        Ok(Self::assemble_with_stores(socket, secret, identity, world, store, Box::new(MemorySurvivors::empty())))
    }

    fn assemble_with_stores(
        socket: UdpSocket,
        secret: u8,
        identity: HostIdentity,
        world: Box<dyn PlanetaryWorld>,
        chronicle: Box<dyn ChronicleStore>,
        survivors: Box<dyn SurvivorStore>,
    ) -> Self {
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
        Self::new(socket, world, identity, state)
    }
}
