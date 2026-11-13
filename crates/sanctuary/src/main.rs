use std::env;
use std::net::ToSocketAddrs;
use std::thread;
use std::time::{Duration, Instant};
use sanctuary::{BeaconRoute, RelayBeacon, SanctuaryHost};
use world::{BiomedCubeWorld, PlanetaryWorld};

/// Command-line configuration for the dedicated headless World-Cube sanctuary or NAT relay beacon.
struct SanctuaryArguments {
    bind_addr: String,
    blocks_per_edge: u32,
    relay_only: bool,
    beacon: BeaconRoute,
}

impl SanctuaryArguments {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(bind_addr: String, blocks_per_edge: u32, relay_only: bool, beacon: BeaconRoute) -> Self {
        Self { bind_addr, blocks_per_edge, relay_only, beacon }
    }

    fn from_cli(args: &[String]) -> Self {
        let mut port = 27777u16;
        let mut blocks = 64u32;
        let mut relay_only = false;
        let mut beacon = BeaconRoute::Unassisted;
        let mut idx = 1;
        while idx < args.len() {
            match args[idx].as_str() {
                "--port" if idx + 1 < args.len() => {
                    if let Ok(p) = args[idx + 1].parse::<u16>() {
                        port = p;
                    }
                    idx += 2;
                }
                "--blocks" if idx + 1 < args.len() => {
                    if let Ok(b) = args[idx + 1].parse::<u32>() {
                        blocks = b.clamp(16, 256);
                    }
                    idx += 2;
                }
                "--relay" if idx + 1 < args.len() => {
                    if let Ok(mut addrs) = args[idx + 1].to_socket_addrs() {
                        if let Some(addr) = addrs.next() {
                            beacon = BeaconRoute::Assisted(addr);
                        }
                    }
                    idx += 2;
                }
                "--relay-only" => {
                    relay_only = true;
                    idx += 1;
                }
                _ => {
                    idx += 1;
                }
            }
        }
        Self::new(format!("0.0.0.0:{port}"), blocks, relay_only, beacon)
    }

    fn execute(self) -> std::io::Result<()> {
        if self.relay_only {
            let mut relay = RelayBeacon::bind(&self.bind_addr)?;
            let bound = relay.local_addr()?;
            println!("=== WORLD-CUBE RELAY BEACON ===");
            println!("Listening on UDP {bound} (end-to-end encrypted NAT rendezvous & fallback)");
            let mut last = Instant::now();
            loop {
                let now = Instant::now();
                let dt = (now - last).as_secs_f32().min(0.1);
                last = now;
                relay.step(dt);
                thread::sleep(Duration::from_millis(10));
            }
        } else {
            let bevel = ((self.blocks_per_edge as f32) * 0.5 * 0.125).clamp(0.5, 1.5);
            let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(self.blocks_per_edge, bevel));
            let mut host = SanctuaryHost::bind(&self.bind_addr, world, self.blocks_per_edge, false, self.beacon)?;
            let bound = host.local_addr()?;
            let ticket = host.ticket().encode();
            println!("=== WORLD-CUBE DEDICATED SANCTUARY ===");
            println!("Bound UDP:     {bound} ({} blocks/edge)", self.blocks_per_edge);
            println!("Direct Join:   --join {}", host.ticket().direct());
            println!("Portal Ticket: --join {ticket}");
            let mut last = Instant::now();
            loop {
                let now = Instant::now();
                let dt = (now - last).as_secs_f32().min(0.1);
                last = now;
                let _ = host.tick(dt);
                thread::sleep(Duration::from_millis(15));
            }
        }
    }
}

fn main() {
    let raw_args: Vec<String> = env::args().collect();
    if let Err(err) = SanctuaryArguments::from_cli(&raw_args).execute() {
        panic!("Sanctuary startup failed: {err}");
    }
}
