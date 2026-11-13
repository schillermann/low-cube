use std::net::ToSocketAddrs;
use sanctuary::{BeaconRoute, HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::command_flags::CommandFlags;

/// Polymorphic session topology unifying solitary play, living-room co-op hosting, and remote internet join.
#[derive(Clone, Debug)]
pub enum PortalConfig {
    Solitary,
    HostSanctuary {
        port: u16,
        beacon: BeaconRoute,
    },
    JoinRemote {
        target: JoinTarget,
    },
}

impl PortalConfig {
    fn resolve_beacon(flags: &CommandFlags) -> BeaconRoute {
        let raw = flags
            .value_of("--relay")
            .or_else(|| std::env::var("LOWCUBE_RELAY").ok());
        if let Some(spec) = raw {
            if let Ok(mut addrs) = spec.trim().to_socket_addrs() {
                if let Some(addr) = addrs.next() {
                    return BeaconRoute::Assisted(addr);
                }
            }
        }
        BeaconRoute::Unassisted
    }

    pub fn from_process() -> Self {
        let flags = CommandFlags::process();
        let beacon = Self::resolve_beacon(&flags);
        if let Some(join_spec) = flags.value_of("--join").or_else(|| std::env::var("LOWCUBE_JOIN").ok()) {
            let target = JoinTarget::parse(&join_spec)
                .unwrap_or_else(|| panic!("Invalid --join portal ticket or host:port `{join_spec}`"));
            return Self::JoinRemote { target };
        }
        if flags.contains_switch("--host") || flags.value_of("--port").is_some() {
            let port = flags
                .value_of("--port")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);
            return Self::HostSanctuary { port, beacon };
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
            Self::HostSanctuary { port, beacon } => {
                let bind_spec = format!("0.0.0.0:{port}");
                let host = SanctuaryHost::bind(&bind_spec, local_authoritative(), blocks, tron_active, beacon)
                    .expect("Failed to bind multiplayer SanctuaryHost UDP socket");
                let ticket = host.ticket().encode();
                println!("=== LOW-CUBE CO-OP PORTAL OPEN ===");
                println!("Direct Address: {}", host.ticket().direct());
                println!("Invite Ticket:  {ticket}");
                Box::new(HostedPlanet::new(host))
            }
            Self::JoinRemote { target } => {
                let remote = RemotePlanet::connect(target, blocks, tron_active, 1.0)
                    .expect("Failed to open multiplayer RemotePlanet UDP socket");
                println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
                Box::new(remote)
            }
        }
    }

    /// Wraps an already-running solitary `PlanetaryWorld` inside `HostedPlanet` on demand from the Pause Menu.
    pub fn open_live_portal(world: Box<dyn PlanetaryWorld>, blocks: u32, tron_active: bool) -> (Box<dyn PlanetaryWorld>, String) {
        if let Some(existing) = world.portal_ticket() {
            println!("=== LOW-CUBE CO-OP PORTAL TICKET ===\n{existing}");
            return (world, existing);
        }
        let beacon = Self::resolve_beacon(&CommandFlags::process());
        let host = SanctuaryHost::bind("0.0.0.0:0", world, blocks, tron_active, beacon)
            .expect("Failed to bind live co-op portal socket");
        let ticket = host.ticket().encode();
        println!("=== LOW-CUBE CO-OP PORTAL OPEN ===");
        println!("Direct Address: {}", host.ticket().direct());
        println!("Invite Ticket:  {ticket}");
        (Box::new(HostedPlanet::new(host)), ticket)
    }

    /// Connects in-game from the Pause Menu to a friend's invite ticket or dedicated server address.
    pub fn join_live_portal(
        target: JoinTarget,
        blocks: u32,
        tron_active: bool,
    ) -> std::io::Result<Box<dyn PlanetaryWorld>> {
        let remote = RemotePlanet::connect(target, blocks, tron_active, 1.0)?;
        println!("=== CONNECTED TO REMOTE WORLD-CUBE ===");
        Ok(Box::new(remote))
    }
}
