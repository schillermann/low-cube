use crate::command_flags::CommandFlags;
use crate::toml_document::TomlDocument;

/// Interface representing the configurable planetary world-cube scale (minimum factor 1 = 128 blocks).
pub trait PlanetaryScale: Send + Sync {
    fn multiple(&self) -> u32;
    fn blocks(&self) -> u32 { 128 * self.multiple() }
}

/// Baseline pocket-moonlet planetary scale (factor 1 = 128 blocks per cube edge).
pub struct MoonletScale;

impl MoonletScale {
    /// Primary constructor initializing the factor-1 world-cube scale.
    pub fn new() -> Self { Self }
}

impl PlanetaryScale for MoonletScale {
    fn multiple(&self) -> u32 { 1 }
}

/// Validated planetary world-cube scale multiplying the 128-block moonlet baseline by an integer >= 1.
pub struct ScaledCube {
    multiple: u32,
}

impl ScaledCube {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(multiple: u32) -> Self { Self { multiple } }

    /// Secondary constructor parsing and fail-fast validating an integer scale factor >= 1.
    pub fn parse(raw: &str, source: &str) -> Self {
        let unquoted = raw.trim().trim_matches('"').trim_matches('\'').trim();
        let parsed = unquoted.parse::<i64>().unwrap_or_else(|_| {
            panic!("Invalid world-cube size '{raw}' in {source}: expected an integer >= 1.")
        });
        if parsed < 1 {
            panic!("Invalid world-cube size '{raw}' in {source}: minimum allowed setting is 1 (128 blocks).");
        }
        Self::new(parsed as u32)
    }
}

impl PlanetaryScale for ScaledCube {
    fn multiple(&self) -> u32 { self.multiple }
}

/// Configuration file-driven planetary scale decorator reading `[world] size` from `server.toml`.
pub struct FileScale {
    scale: Box<dyn PlanetaryScale>,
}

impl FileScale {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(scale: Box<dyn PlanetaryScale>) -> Self { Self { scale } }

    /// Secondary constructor targeting the `server.toml` configuration file.
    pub fn server(fallback: Box<dyn PlanetaryScale>) -> Self { Self::from_path("server.toml", fallback) }

    /// Secondary constructor loading world-cube size from an explicit configuration file path.
    pub fn from_path(path: &str, fallback: Box<dyn PlanetaryScale>) -> Self {
        let Some(doc) = TomlDocument::from_file(path) else { return Self::new(fallback) };
        let raw = doc
            .section_property("world", "size", true)
            .or_else(|| doc.section_property("world", "scale", true));
        if let Some(val) = raw {
            Self::new(Box::new(ScaledCube::parse(&val, path)))
        } else {
            Self::new(fallback)
        }
    }
}

impl PlanetaryScale for FileScale {
    fn multiple(&self) -> u32 { self.scale.multiple() }
}

/// Process environment variable-driven planetary scale decorator reading `LOW_CUBE_SIZE`.
pub struct EnvironmentScale {
    scale: Box<dyn PlanetaryScale>,
}

impl EnvironmentScale {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(scale: Box<dyn PlanetaryScale>) -> Self { Self { scale } }

    /// Secondary constructor reading `LOW_CUBE_SIZE` or `LOW_CUBE_SCALE` from the process environment.
    pub fn process(fallback: Box<dyn PlanetaryScale>) -> Self {
        if let Ok(raw) = std::env::var("LOW_CUBE_SIZE").or_else(|_| std::env::var("LOW_CUBE_SCALE")) {
            Self::new(Box::new(ScaledCube::parse(&raw, "LOW_CUBE_SIZE")))
        } else {
            Self::new(fallback)
        }
    }
}

impl PlanetaryScale for EnvironmentScale {
    fn multiple(&self) -> u32 { self.scale.multiple() }
}

/// Command-line argument-driven planetary scale decorator reading `--size` or `--scale`.
pub struct ArgumentScale {
    scale: Box<dyn PlanetaryScale>,
}

impl ArgumentScale {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(scale: Box<dyn PlanetaryScale>) -> Self { Self { scale } }

    /// Secondary constructor capturing arguments from the active process environment.
    pub fn process(fallback: Box<dyn PlanetaryScale>) -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::from_arguments(&args, fallback)
    }

    /// Secondary constructor parsing an explicit arguments slice.
    pub fn from_arguments(args: &[String], fallback: Box<dyn PlanetaryScale>) -> Self {
        let flags = CommandFlags::from_slice(args);
        if let Some(raw) = flags.value_of("--size").or_else(|| flags.value_of("--scale")) {
            Self::new(Box::new(ScaledCube::parse(&raw, "--size")))
        } else {
            Self::new(fallback)
        }
    }
}

impl PlanetaryScale for ArgumentScale {
    fn multiple(&self) -> u32 { self.scale.multiple() }
}

/// Server launch planetary scale decorator coordinating CLI arguments, environment variables, `server.toml`, and factor-1 baseline.
pub struct ServerScale {
    scale: Box<dyn PlanetaryScale>,
}

impl ServerScale {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(scale: Box<dyn PlanetaryScale>) -> Self { Self { scale } }

    /// Secondary constructor assembling the full decorator chain in priority order.
    pub fn assembled() -> Self {
        Self::new(Box::new(ArgumentScale::process(Box::new(
            EnvironmentScale::process(Box::new(FileScale::server(Box::new(MoonletScale::new())))),
        ))))
    }
}

impl PlanetaryScale for ServerScale {
    fn multiple(&self) -> u32 { self.scale.multiple() }
}
