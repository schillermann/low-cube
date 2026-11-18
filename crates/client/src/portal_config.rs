use std::time::Duration;
use sanctuary::{ConduitPlanet, HostedPlanet, JoinTarget, RemoteLink, RemotePlanet, SanctuaryHost};
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::command_flags::CommandFlags;
use crate::menu::PortalInvite;

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
    #[cfg(test)]
    pub fn from_link(link: RemoteLink) -> (Self, String) {
        let reply_code = link
            .punch_code()
            .expect("wormhole link must carry punch code")
            .encode();
        (Self::Punching { link, elapsed: 0.0 }, reply_code)
    }

    pub fn begin(target: JoinTarget) -> (Self, DialOutcome) {
        let Ok(mut link) = RemoteLink::bind(target) else {
            return (Self::Idle, DialOutcome::Failed);
        };
        if let Some((blocks, tron)) = link.try_welcome_within(Duration::from_millis(240)) {
            println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
            let remote = RemotePlanet::from_link(link, blocks, tron, 1.0);
            return (Self::Idle, DialOutcome::Connected(Box::new(ConduitPlanet::spawning(Box::new(remote)))));
        }
        if let Some(punch) = link.punch_code() {
            let reply_code = punch.encode();
            println!("=== CO-OP SIGNAL OPEN - SEND REPLY CODE TO HOST: {reply_code} ===");
            return (Self::Punching { link, elapsed: 0.0 }, DialOutcome::Punching(reply_code));
        }
        if let Some((blocks, tron)) = link.try_welcome_within(Duration::from_millis(210)) {
            println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
            let remote = RemotePlanet::from_link(link, blocks, tron, 1.0);
            return (Self::Idle, DialOutcome::Connected(Box::new(ConduitPlanet::spawning(Box::new(remote)))));
        }
        (Self::Idle, DialOutcome::Failed)
    }

    pub fn cancel(&mut self) {
        *self = Self::Idle;
    }

    pub fn step(&mut self, dt: f32) -> DialStep {
        let ready_cfg = match *self {
            Self::Idle => return DialStep::Idle,
            Self::Punching { ref mut link, ref mut elapsed } => {
                *elapsed += dt;
                link.step_handshake(dt)
            }
        };
        if let Some((blocks, tron)) = ready_cfg {
            if let Self::Punching { link, .. } = std::mem::replace(self, Self::Idle) {
                println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
                let remote = RemotePlanet::from_link(link, blocks, tron, 1.0);
                return DialStep::Connected(Box::new(ConduitPlanet::spawning(Box::new(remote))));
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
    blocks: u32,
    tron_active: bool,
}

impl LivePortal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(blocks: u32, tron_active: bool) -> Self {
        Self { blocks, tron_active }
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
        let host = SanctuaryHost::bind("0.0.0.0:0", world, self.blocks, self.tron_active)
            .expect("Failed to bind live co-op portal socket");
        let code = host.code().encode();
        let ticket = host.ticket().encode();
        println!("=== LOW-CUBE CO-OP PORTAL OPEN ===");
        println!("Portal Code:    {code}");
        println!("Direct Address: {}", host.ticket().direct());
        println!("Invite Ticket:  {ticket}");
        let hosted = Box::new(HostedPlanet::new(host));
        (Box::new(ConduitPlanet::spawning(hosted)), PortalInvite::hosted(code, ticket))
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

    pub fn assemble_planet(&self, blocks: u32, tron_active: bool) -> Box<dyn PlanetaryWorld> {
        let local_authoritative = || -> Box<dyn PlanetaryWorld> {
            if tron_active {
                Box::new(BiomedCubeWorld::beveled_tron(blocks, 1.0))
            } else {
                Box::new(BiomedCubeWorld::beveled(blocks, 1.0))
            }
        };
        match *self {
            Self::Solitary => local_authoritative(),
            Self::HostSanctuary { port } => {
                let bind_spec = format!("0.0.0.0:{port}");
                let host = SanctuaryHost::bind(&bind_spec, local_authoritative(), blocks, tron_active)
                    .expect("Failed to bind multiplayer SanctuaryHost UDP socket");
                let code = host.code().encode();
                let ticket = host.ticket().encode();
                println!("=== LOW-CUBE CO-OP PORTAL OPEN ===");
                println!("Portal Code:    {code}");
                println!("Direct Address: {}", host.ticket().direct());
                println!("Invite Ticket:  {ticket}");
                Box::new(ConduitPlanet::spawning(Box::new(HostedPlanet::new(host))))
            }
            Self::JoinRemote { target } => {
                let remote = RemotePlanet::connect(target, blocks, tron_active, 1.0)
                    .expect("Failed to connect multiplayer RemotePlanet UDP socket");
                println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
                Box::new(ConduitPlanet::spawning(Box::new(remote)))
            }
        }
    }
}
