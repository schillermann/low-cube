use std::time::Duration;
use sanctuary::{
    DiskPassports, HostedPlanet, JoinTarget, MemoryPassports, PassportVault, RemoteLink,
    RemotePlanet, SanctuaryHost,
};
use world::{Callsign, CelestialSystem, PlanetaryWorld, SolarProportions};
use crate::command_flags::CommandFlags;
use crate::menu::PortalInvite;
use crate::toml_document::TomlDocument;

/// Local profile source resolving the player's preferred `Callsign` from `character.toml` or OS login (`<= 4` fields).
struct LocalCallsignSource {
    profile_path: &'static str,
}

impl LocalCallsignSource {
    const fn new(profile_path: &'static str) -> Self {
        Self { profile_path }
    }

    fn resolve(&self) -> Callsign {
        if cfg!(test) {
            return Callsign::unnamed();
        }
        if let Some(doc) = TomlDocument::from_file(self.profile_path) {
            if let Some(raw) = doc.property("name") {
                let cs = Callsign::sanitized(&raw);
                if cs.present() {
                    return cs;
                }
            }
        }
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .ok()
            .map(|u| Callsign::sanitized(&u))
            .filter(Callsign::present)
            .unwrap_or_else(Callsign::unnamed)
    }
}

/// Client passport keyring resolving player `Callsign` and persisting server-issued identities (`<= 4` fields).
pub struct ClientPassportBook {
    path: String,
    callsign: Callsign,
}

impl ClientPassportBook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(path: String, callsign: Callsign) -> Self {
        Self { path, callsign }
    }

    pub fn from_process() -> Self {
        let flags = CommandFlags::process();
        let path = flags
            .value_of("--passports")
            .or_else(|| flags.value_of("--identity"))
            .or_else(|| std::env::var("LOW_CUBE_PASSPORTS").ok())
            .or_else(|| std::env::var("LOW_CUBE_IDENTITY").ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "passports.toml".to_string());
        let explicit_name = flags
            .value_of("--name")
            .or_else(|| std::env::var("LOW_CUBE_NAME").ok())
            .map(|s| Callsign::sanitized(&s))
            .filter(Callsign::present);
        let fallback = explicit_name.unwrap_or_else(|| LocalCallsignSource::new("character.toml").resolve());
        Self::new(path, fallback)
    }

    pub fn host_callsign(&self) -> Callsign {
        let vault = self.open_vault();
        let preferred = vault.preferred_callsign();
        if preferred.present() { preferred } else { Callsign::host() }
    }

    pub fn open_vault(&self) -> Box<dyn PassportVault> {
        if cfg!(test) {
            Box::new(MemoryPassports::with_callsign(self.callsign))
        } else {
            Box::new(DiskPassports::open(&self.path, self.callsign))
        }
    }
}

/// Outcome of initiating an in-game P2P portal join.
pub enum DialOutcome {
    Connected(Box<dyn PlanetaryWorld>),
    Punching(String),
    Failed,
}

/// Polymorphic outcome of advancing a live background NAT hole-punching dial.
pub enum DialStep {
    Idle,
    Listening,
    Connected(Box<dyn PlanetaryWorld>),
    TimedOut,
}

/// Non-blocking NAT hole-punching state keeping the guest's STUN-mapped UDP socket alive across menu closes.
pub enum PortalDial {
    Idle,
    Punching {
        link: RemoteLink,
        elapsed: f32,
    },
}

impl PortalDial {
    fn wrap_remote(link: RemoteLink, proportions: SolarProportions) -> Box<dyn PlanetaryWorld> {
        Box::new(RemotePlanet::from_cosmos_link(link, proportions, 1.0).spawning())
    }

    #[cfg(test)]
    pub fn from_link(link: RemoteLink) -> (Self, String) {
        let reply_code = link
            .punch_code()
            .expect("wormhole link must carry punch code")
            .encode();
        (Self::Punching { link, elapsed: 0.0 }, reply_code)
    }

    pub fn begin(target: JoinTarget) -> (Self, DialOutcome) {
        let vault = ClientPassportBook::from_process().open_vault();
        let Ok(mut link) = RemoteLink::bind_with_vault(target, vault) else {
            return (Self::Idle, DialOutcome::Failed);
        };
        if let Some(prop) = link.try_welcome_cosmos_within(Duration::from_millis(240)) {
            println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
            return (Self::Idle, DialOutcome::Connected(Self::wrap_remote(link, prop)));
        }
        if let Some(punch) = link.punch_code() {
            let reply_code = punch.encode();
            println!("=== CO-OP SIGNAL OPEN - SEND REPLY CODE TO HOST: {reply_code} ===");
            return (Self::Punching { link, elapsed: 0.0 }, DialOutcome::Punching(reply_code));
        }
        if let Some(prop) = link.try_welcome_cosmos_within(Duration::from_millis(210)) {
            println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
            return (Self::Idle, DialOutcome::Connected(Self::wrap_remote(link, prop)));
        }
        (Self::Idle, DialOutcome::Failed)
    }

    pub fn cancel(&mut self) {
        *self = Self::Idle;
    }

    pub fn step(&mut self, dt: f32) -> DialStep {
        let ready_prop = match *self {
            Self::Idle => return DialStep::Idle,
            Self::Punching { ref mut link, ref mut elapsed } => {
                *elapsed += dt;
                link.step_cosmos_handshake(dt)
            }
        };
        if let Some(prop) = ready_prop {
            if let Self::Punching { link, .. } = std::mem::replace(self, Self::Idle) {
                println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
                return DialStep::Connected(Self::wrap_remote(link, prop));
            }
        }
        if let Self::Punching { elapsed, .. } = *self {
            if elapsed > 180.0 {
                *self = Self::Idle;
                return DialStep::TimedOut;
            }
        }
        DialStep::Listening
    }
}

/// Encapsulated live co-op portal assembly wrapping a running `PlanetaryWorld` inside `HostedPlanet`.
#[derive(Clone, Copy, Debug)]
pub struct LivePortal {
    proportions: SolarProportions,
}

impl LivePortal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(proportions: SolarProportions) -> Self {
        Self { proportions }
    }

    pub fn open(&self, world: Box<dyn PlanetaryWorld>) -> (Box<dyn PlanetaryWorld>, PortalInvite) {
        if let Some(existing_ticket) = world.portal_ticket() {
            let existing_code = world.portal_phrase().unwrap_or_default();
            println!("=== LOW-CUBE CO-OP PORTAL ===");
            if !existing_code.is_empty() {
                println!("Portal Code:    {existing_code}");
            }
            println!("Invite Ticket:  {existing_ticket}");
            return (world, PortalInvite::hosted(existing_code, existing_ticket));
        }
        let host_name = ClientPassportBook::from_process().host_callsign();
        let host = SanctuaryHost::bind_named_cosmos("0.0.0.0:0", host_name, world, self.proportions)
            .expect("Failed to bind live co-op portal socket");
        let code = host.code().encode();
        let ticket = host.ticket().encode();
        println!("=== LOW-CUBE CO-OP PORTAL OPEN ===");
        println!("Portal Code:    {code}");
        println!("Direct Address: {}", host.ticket().direct());
        println!("Invite Ticket:  {ticket}");
        (Box::new(HostedPlanet::spawning(host)), PortalInvite::hosted(code, ticket))
    }
}

/// Polymorphic session topology unifying solitary play, P2P co-op hosting, and direct remote join.
#[derive(Clone, Debug)]
pub enum PortalConfig {
    Solitary,
    HostSanctuary { port: u16 },
    JoinRemote { target: JoinTarget },
}

impl PortalConfig {
    pub fn from_process() -> Self {
        let flags = CommandFlags::process();
        if let Some(join_spec) = flags.value_of("--join").or_else(|| std::env::var("LOWCUBE_JOIN").ok()) {
            let target = JoinTarget::parse(&join_spec)
                .unwrap_or_else(|| panic!("Invalid --join 12-char code, ticket, or host:port `{join_spec}`"));
            return Self::JoinRemote { target };
        }
        if flags.contains_switch("--host") || flags.value_of("--port").is_some() {
            let port = flags
                .value_of("--port")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);
            return Self::HostSanctuary { port };
        }
        Self::Solitary
    }

    pub fn assemble_planet(&self, proportions: SolarProportions) -> Box<dyn PlanetaryWorld> {
        let local_authoritative = || -> Box<dyn PlanetaryWorld> {
            CelestialSystem::heliosphere(proportions, 1.0)
        };
        match *self {
            Self::Solitary => local_authoritative(),
            Self::HostSanctuary { port } => {
                let bind_spec = format!("0.0.0.0:{port}");
                let host_name = ClientPassportBook::from_process().host_callsign();
                let host = SanctuaryHost::bind_named_cosmos(&bind_spec, host_name, local_authoritative(), proportions)
                    .expect("Failed to bind multiplayer SanctuaryHost UDP socket");
                let code = host.code().encode();
                let ticket = host.ticket().encode();
                println!("=== LOW-CUBE CO-OP PORTAL OPEN ===");
                println!("Portal Code:    {code}");
                println!("Direct Address: {}", host.ticket().direct());
                println!("Invite Ticket:  {ticket}");
                Box::new(HostedPlanet::spawning(host))
            }
            Self::JoinRemote { target } => {
                let vault = ClientPassportBook::from_process().open_vault();
                let remote = RemotePlanet::connect_cosmos_with_vault(target, vault, proportions, 1.0)
                    .expect("Failed to connect multiplayer RemotePlanet UDP socket")
                    .spawning();
                println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
                Box::new(remote)
            }
        }
    }
}
