use std::fmt::Write as _;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(test)]
use glam::Vec3;
#[cfg(test)]
use world::{BiomedQuadSphere, PlanetaryWorld};
use world::ClimateZone;
#[cfg(test)]
use crate::avatar::{AvatarMotor as _, AvatarPresence as _, PlanetaryAvatar};
#[cfg(test)]
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::toml_document::TomlDocument;
#[cfg(test)]
use super::PresenceProbe;
use super::{FlightEvent, FlightTape, IncidentMedium, LiveTape, SilentTape, TickTrace};

/// In-memory TOML printed-media target for `FlightTape` dossiers ("Tell, Don't Ask").
pub struct MemoryDossier {
    header: String,
    boundaries: String,
    events: String,
    ticks: String,
}

impl MemoryDossier {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(header: String, boundaries: String, events: String, ticks: String) -> Self {
        Self { header, boundaries, events, ticks }
    }

    pub fn empty() -> Self {
        Self::new(String::new(), String::new(), String::new(), String::new())
    }

    pub fn render_toml(&self) -> String {
        let mut out = String::with_capacity(
            self.header.len() + self.boundaries.len() + self.events.len() + self.ticks.len() + 64,
        );
        out.push_str(&self.header);
        out.push_str(&self.boundaries);
        if !self.events.is_empty() {
            out.push_str("\n[events]\nentries = [\n");
            out.push_str(&self.events);
            out.push_str("]\n");
        }
        out.push_str("\n[trace]\nticks = [\n");
        out.push_str(&self.ticks);
        out.push_str("]\n");
        out
    }
}

impl IncidentMedium for MemoryDossier {
    fn record_header(&mut self, world_blocks: u32, tick_count: usize, anomaly_count: u8) {
        let _ = writeln!(
            self.header,
            "[incident]\nversion = \"{}\"\nworld_blocks = {}\nrecorded_ticks = {}\nanomalies = {}",
            env!("CARGO_PKG_VERSION"),
            world_blocks,
            tick_count,
            anomaly_count
        );
    }

    fn record_boundary(&mut self, section: &str, trace: TickTrace, zone: ClimateZone) {
        trace.transfer(|pos, view, dt, up, flags, intent| {
            let _ = writeln!(
                self.boundaries,
                "\n[{section}]\nzone = \"{zone:?}\"\nposition = [{:.4}, {:.4}, {:.4}]\nview = [{:.4}, {:.4}, {:.4}]\nsurface_up = [{:.4}, {:.4}, {:.4}]\ndt = {:.6}\nflags = {flags}\nintent = [{:.3}, {:.3}]",
                pos.x, pos.y, pos.z, view.x, view.y, view.z, up.x, up.y, up.z, dt, intent.0, intent.1
            );
        });
    }

    fn record_event(&mut self, tick_offset: usize, event: FlightEvent) {
        let _ = writeln!(self.events, "  \"{tick_offset}:{}\",", event.summary());
    }

    fn record_tick(&mut self, _index: usize, trace: TickTrace) {
        trace.transfer(|pos, view, dt, up, flags, intent| {
            let _ = writeln!(
                self.ticks,
                "  \"{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.6},{:.4},{:.4},{:.4},{flags},{:.3},{:.3}\",",
                pos.x, pos.y, pos.z, view.x, view.y, view.z, dt, up.x, up.y, up.z, intent.0, intent.1
            );
        });
    }
}

/// Disk-backed TOML incident report medium writing into `reports/incident_<millis>.toml`.
pub struct DiskDossier {
    folder: String,
    memory: MemoryDossier,
}

impl DiskDossier {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(folder: String, memory: MemoryDossier) -> Self {
        Self { folder, memory }
    }

    pub fn in_folder(folder: &str) -> Self {
        Self::new(folder.to_string(), MemoryDossier::empty())
    }

    pub fn commit_to_disk(&self) -> String {
        fs::create_dir_all(&self.folder).expect("Failed to create incident reports directory");
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System clock before UNIX_EPOCH")
            .as_millis();
        let path = format!("{}/incident_{millis}.toml", self.folder);
        fs::write(&path, self.memory.render_toml()).expect("Failed to write incident TOML dossier");
        path
    }
}

impl IncidentMedium for DiskDossier {
    fn record_header(&mut self, world_blocks: u32, tick_count: usize, anomaly_count: u8) {
        self.memory.record_header(world_blocks, tick_count, anomaly_count);
    }

    fn record_boundary(&mut self, section: &str, trace: TickTrace, zone: ClimateZone) {
        self.memory.record_boundary(section, trace, zone);
    }

    fn record_event(&mut self, tick_offset: usize, event: FlightEvent) {
        self.memory.record_event(tick_offset, event);
    }

    fn record_tick(&mut self, index: usize, trace: TickTrace) {
        self.memory.record_tick(index, trace);
    }
}

/// Persistent profile storing the player's menu ON/OFF choice for the Flight Blackbox.
pub struct FileBlackBox {
    path: String,
}

impl FileBlackBox {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(path: String) -> Self {
        Self { path }
    }

    pub fn profile() -> Self {
        Self::new("blackbox.toml".to_string())
    }

    pub fn load_tape(&self) -> Box<dyn FlightTape> {
        let enabled = TomlDocument::from_file(&self.path)
            .and_then(|doc| doc.section_property("blackbox", "active", true))
            .map(|val| val.trim() != "false")
            .unwrap_or(true);
        if enabled {
            Box::new(LiveTape::fresh())
        } else {
            Box::new(SilentTape::new())
        }
    }

    pub fn save_active(&self, active: bool) {
        let content = format!("[blackbox]\nactive = {active}\n");
        let _ = fs::write(&self.path, content);
    }
}

#[cfg(test)]
struct ReplayControls {
    world_dir: Vec3,
    flags: u8,
}

#[cfg(test)]
impl KeyboardControls for ReplayControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}

    fn drive_locomotion(&self, _forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        if self.world_dir.length_squared() > 1e-6 {
            target.travel(self.world_dir);
        }
        if (self.flags & 1) != 0 {
            target.jump();
        }
        if (self.flags & 2) != 0 {
            target.sprint();
        }
        if (self.flags & 4) != 0 {
            target.sneak();
        }
    }
}

/// Deterministic incident replay parsed from a TOML bug-report dossier ("Every Bug Becomes an Automated Test").
#[cfg(test)]
pub struct IncidentReplay {
    world_blocks: u32,
    frames: Vec<TickTrace>,
}

#[cfg(test)]
impl IncidentReplay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(world_blocks: u32, frames: Vec<TickTrace>) -> Self {
        Self { world_blocks, frames }
    }

    pub fn from_toml(toml_text: &str) -> Self {
        let mut blocks = 128u32;
        let mut frames = Vec::new();
        let mut in_ticks = false;
        for raw_line in toml_text.lines() {
            let line = raw_line.trim();
            if let Some(rest) = line.strip_prefix("world_blocks =") {
                blocks = rest.trim().parse::<u32>().expect("Invalid world_blocks in dossier");
            } else if line == "ticks = [" {
                in_ticks = true;
            } else if in_ticks && line == "]" {
                in_ticks = false;
            } else if in_ticks {
                let inner = line.trim_end_matches(',').trim().trim_matches('"');
                let parts: Vec<f32> = inner
                    .split(',')
                    .map(|s| s.trim().parse::<f32>().expect("Invalid float in dossier tick"))
                    .collect();
                assert_eq!(parts.len(), 13, "Corrupt tick entry in incident dossier: {inner}");
                frames.push(TickTrace::new(
                    Vec3::new(parts[0], parts[1], parts[2]),
                    (Vec3::new(parts[3], parts[4], parts[5]), parts[6]),
                    (Vec3::new(parts[7], parts[8], parts[9]), parts[10] as u8),
                    (parts[11], parts[12]),
                ));
            }
        }
        assert!(!frames.is_empty(), "Incident dossier contains zero replayable ticks");
        Self::new(blocks, frames)
    }

    pub fn tick_count(&self) -> usize {
        self.frames.len()
    }

    pub fn replay_and_verify(&self) -> Vec3 {
        let world = BiomedQuadSphere::barren(self.world_blocks);
        self.replay_on(&world)
    }

    pub fn replay_on(&self, world: &dyn PlanetaryWorld) -> Vec3 {
        let start = self.frames[0].transfer(|pos, _, _, _, _, _| pos);
        let mut avatar = PlanetaryAvatar::at(start);
        let half = (self.world_blocks as f32) * 0.5;
        let mut probe = PresenceProbe::at_origin();
        for frame in &self.frames {
            frame.transfer(|_, view, dt, up, flags, (fwd_i, str_i)| {
                let flat_fwd = (view - up * view.dot(up)).normalize_or_zero();
                let right = flat_fwd.cross(up).normalize_or_zero();
                let world_dir = (flat_fwd * fwd_i + right * str_i).normalize_or_zero();
                let ctrl = ReplayControls { world_dir, flags };
                avatar.step_motion(dt, &ctrl, world);
                avatar.report_presence(&mut probe);
                let pos = probe.position();
                assert!(pos.is_finite(), "Replayed avatar position became non-finite: {pos:?}");
                assert!(
                    pos.abs().max_element() >= half - 0.15,
                    "Replayed avatar penetrated solid planetary surface at {pos:?} (half={half})"
                );
            });
        }
        probe.position()
    }
}
