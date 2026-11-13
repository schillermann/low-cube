use std::time::Duration;
use sanctuary::{HostedPlanet, JoinTarget, RemoteLink, RemotePlanet, SanctuaryHost};
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::command_flags::CommandFlags;
use crate::menu::PortalInvite;

/// Outcome of initiating an in-game P2P portal join.
pub enum DialOutcome {
    Connected(Box<dyn PlanetaryWorld>),
    Punching(String),
    Failed,
}

/// Non-blocking NAT hole-punching state keeping the guest's STUN-mapped UDP socket alive.
pub enum PortalDial {
    Idle,
    Punching {
        link: RemoteLink,
        elapsed: f32,
    },
}

impl PortalDial {
    pub fn begin(target: JoinTarget) -> (Self, DialOutcome) {
        let Ok(mut link) = RemoteLink::bind(target) else {
            return (Self::Idle, DialOutcome::Failed);
        };
        if let Some((blocks, tron)) = link.try_welcome_within(Duration::from_millis(240)) {
            println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
            let remote = RemotePlanet::from_link(link, blocks, tron, 1.0);
            return (Self::Idle, DialOutcome::Connected(Box::new(remote)));
        }
        if let Some(punch) = link.punch_code() {
            let reply_code = punch.encode();
            println!("=== NAT LOCKED - TELL HOST PUNCH CODE: {reply_code} ===");
            return (Self::Punching { link, elapsed: 0.0 }, DialOutcome::Punching(reply_code));
        }
        if let Some((blocks, tron)) = link.try_welcome_within(Duration::from_millis(210)) {
            println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
            let remote = RemotePlanet::from_link(link, blocks, tron, 1.0);
            return (Self::Idle, DialOutcome::Connected(Box::new(remote)));
        }
        (Self::Idle, DialOutcome::Failed)
    }

    pub fn cancel(&mut self) {
        *self = Self::Idle;
    }

    pub fn step(&mut self, dt: f32) -> Option<Box<dyn PlanetaryWorld>> {
        let ready_cfg = match *self {
            Self::Idle => return None,
            Self::Punching { ref mut link, ref mut elapsed, .. } => {
                *elapsed += dt;
                link.step_handshake(dt)
            }
        };
        if let Some((blocks, tron)) = ready_cfg {
            if let Self::Punching { link, .. } = std::mem::replace(self, Self::Idle) {
                println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
                return Some(Box::new(RemotePlanet::from_link(link, blocks, tron, 1.0)));
            }
        }
        if let Self::Punching { elapsed, .. } = *self {
            if elapsed > 180.0 {
                *self = Self::Idle;
            }
        }
        None
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
                Box::new(HostedPlanet::new(host))
            }
            Self::JoinRemote { target } => {
                let remote = RemotePlanet::connect(target, blocks, tron_active, 1.0)
                    .expect("Failed to connect multiplayer RemotePlanet UDP socket");
                println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
                Box::new(remote)
            }
        }
    }

    /// Wraps an already-running solitary `PlanetaryWorld` inside `HostedPlanet` on demand from the Pause Menu.
    pub fn open_live_portal(
        world: Box<dyn PlanetaryWorld>,
        blocks: u32,
        tron_active: bool,
    ) -> (Box<dyn PlanetaryWorld>, PortalInvite) {
        if let Some(existing_ticket) = world.portal_ticket() {
            let existing_code = world.portal_phrase().unwrap_or_default();
            println!("=== LOW-CUBE CO-OP PORTAL ===");
            if !existing_code.is_empty() {
                println!("Portal Code:    {existing_code}");
            }
            println!("Invite Ticket:  {existing_ticket}");
            return (world, PortalInvite::hosted(existing_code, existing_ticket));
        }
        let host = SanctuaryHost::bind("0.0.0.0:0", world, blocks, tron_active)
            .expect("Failed to bind live co-op portal socket");
        let code = host.code().encode();
        let ticket = host.ticket().encode();
        println!("=== LOW-CUBE CO-OP PORTAL OPEN ===");
        println!("Portal Code:    {code}");
        println!("Direct Address: {}", host.ticket().direct());
        println!("Invite Ticket:  {ticket}");
        (Box::new(HostedPlanet::new(host)), PortalInvite::hosted(code, ticket))
    }
}
