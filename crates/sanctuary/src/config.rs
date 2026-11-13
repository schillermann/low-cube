use std::env;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::SanctuaryHost;

/// Encapsulated `server.toml` text resolving dedicated sanctuary properties without external crates.
#[derive(Clone, Debug)]
pub struct SanctuaryToml {
    content: String,
}

impl SanctuaryToml {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(content: String) -> Self {
        Self { content }
    }

    /// Secondary constructor loading configuration text from disk when present.
    pub fn from_file(path: &str) -> Self {
        Self::new(fs::read_to_string(path).unwrap_or_default())
    }

    pub fn section_property(&self, section: &str, key: &str, top_level_allowed: bool) -> Option<String> {
        let mut active = top_level_allowed;
        for line in self.content.lines().map(str::trim) {
            if line.starts_with('[') && line.ends_with(']') {
                let header = line[1..line.len() - 1].trim();
                active = header.eq_ignore_ascii_case(section);
                continue;
            }
            if active && !line.is_empty() && !line.starts_with('#') {
                if let Some((k, raw_val)) = line.split_once('=') {
                    if k.trim().eq_ignore_ascii_case(key) {
                        let uncommented = raw_val.split('#').next().unwrap_or("").trim();
                        let cleaned = uncommented.trim_matches('"').trim_matches('\'').trim();
                        return Some(cleaned.to_string());
                    }
                }
            }
        }
        None
    }
}

/// Encapsulated UDP binding port for a dedicated P2P `SanctuaryHost`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SanctuaryEndpoint {
    port: u16,
}

impl SanctuaryEndpoint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    fn serve(self, blocks_per_edge: u32) -> std::io::Result<()> {
        let bind_addr = format!("0.0.0.0:{}", self.port);
        let bevel = ((blocks_per_edge as f32) * 0.5 * 0.125).clamp(0.5, 1.0);
        let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(blocks_per_edge, bevel));
        let mut host = SanctuaryHost::bind(&bind_addr, world, blocks_per_edge, false)?;
        let bound = host.local_addr()?;
        let ticket = host.ticket().encode();
        println!("=== LOW-CUBE DEDICATED SERVER ===");
        println!("Listening UDP:  {bound} ({blocks_per_edge} blocks/edge)");
        println!("Portal Code:    {}", host.code().encode());
        println!("Direct Address: {}", host.ticket().direct());
        println!("Invite Ticket:  {ticket}");
        println!("Players join in-game via Esc -> JOIN CO-OP / SERVER");
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

/// Assembled dedicated server configuration combining `server.toml`, environment variables, and CLI flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SanctuaryPlan {
    endpoint: SanctuaryEndpoint,
    blocks_per_edge: u32,
}

impl SanctuaryPlan {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(endpoint: SanctuaryEndpoint, blocks_per_edge: u32) -> Self {
        Self { endpoint, blocks_per_edge }
    }

    pub const fn endpoint(&self) -> SanctuaryEndpoint {
        self.endpoint
    }

    pub const fn blocks_per_edge(&self) -> u32 {
        self.blocks_per_edge
    }

    /// Secondary constructor assembling configuration in priority order: CLI > Env > `server.toml` > Baseline.
    pub fn assembled(args: &[String], toml: &SanctuaryToml) -> Self {
        let mut port = 27777u16;
        let mut blocks = 128u32;

        if let Some(raw_size) = toml
            .section_property("world", "size", true)
            .or_else(|| toml.section_property("world", "scale", true))
        {
            let factor = raw_size.parse::<i64>().unwrap_or_else(|_| {
                panic!("Invalid world-cube size '{raw_size}' in server.toml: expected integer >= 1")
            });
            assert!(factor >= 1, "Invalid world-cube size '{raw_size}' in server.toml: minimum is 1");
            blocks = 128 * (factor as u32);
        }
        if let Some(raw_port) = toml.section_property("server", "port", true) {
            port = raw_port.parse::<u16>().unwrap_or_else(|_| {
                panic!("Invalid server port '{raw_port}' in server.toml: expected 1..=65535")
            });
        }

        if let Ok(env_size) = env::var("LOW_CUBE_SIZE").or_else(|_| env::var("LOW_CUBE_SCALE")) {
            let factor = env_size.trim().parse::<i64>().unwrap_or_else(|_| {
                panic!("Invalid LOW_CUBE_SIZE '{env_size}': expected integer >= 1")
            });
            assert!(factor >= 1, "Invalid LOW_CUBE_SIZE '{env_size}': minimum is 1");
            blocks = 128 * (factor as u32);
        }
        if let Ok(env_port) = env::var("LOW_CUBE_PORT") {
            if let Ok(parsed) = env_port.trim().parse::<u16>() {
                port = parsed;
            }
        }

        let mut idx = 1;
        while idx < args.len() {
            let arg = args[idx].as_str();
            if let Some(val) = arg.strip_prefix("--size=").or_else(|| arg.strip_prefix("--scale=")) {
                let factor = val.parse::<i64>().expect("Invalid --size integer");
                assert!(factor >= 1, "--size must be >= 1");
                blocks = 128 * (factor as u32);
                idx += 1;
            } else if matches!(arg, "--size" | "--scale") && idx + 1 < args.len() {
                let factor = args[idx + 1].parse::<i64>().expect("Invalid --size integer");
                assert!(factor >= 1, "--size must be >= 1");
                blocks = 128 * (factor as u32);
                idx += 2;
            } else if arg == "--blocks" && idx + 1 < args.len() {
                if let Ok(b) = args[idx + 1].parse::<u32>() {
                    blocks = b.clamp(16, 1024);
                }
                idx += 2;
            } else if arg == "--port" && idx + 1 < args.len() {
                if let Ok(p) = args[idx + 1].parse::<u16>() {
                    port = p;
                }
                idx += 2;
            } else {
                idx += 1;
            }
        }

        Self::new(SanctuaryEndpoint::new(port), blocks)
    }

    pub fn execute(self) -> std::io::Result<()> {
        self.endpoint.serve(self.blocks_per_edge)
    }
}
