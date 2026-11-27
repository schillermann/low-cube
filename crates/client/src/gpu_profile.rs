use engine::GpuPowerProfile;
use crate::toml_document::TomlDocument;

/// Focused contract resolving the active laptop/desktop `GpuPowerProfile` (`HighPerformance` vs. `LowPower`).
pub trait GpuChoice {
    fn select_profile(&self) -> GpuPowerProfile;
}

/// Baseline AC-powered profile selecting `DiscreteHighPerformance` (`MX150` / dedicated GPU, `0` fields).
#[derive(Clone, Copy, Debug)]
pub struct AcPowerChoice;

impl AcPowerChoice {
    /// Primary constructor doing nothing except creating the instance.
    pub const fn new() -> Self {
        Self
    }
}

impl GpuChoice for AcPowerChoice {
    fn select_profile(&self) -> GpuPowerProfile {
        GpuPowerProfile::DiscreteHighPerformance
    }
}

/// Yegor Decorator reading `[graphics] gpu = "low" | "high"` from a TOML file (`2` private fields).
pub struct FileGpuChoice<C> {
    path: &'static str,
    origin: C,
}

impl<C> FileGpuChoice<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(path: &'static str, origin: C) -> Self {
        Self { path, origin }
    }
}

impl<C: GpuChoice> GpuChoice for FileGpuChoice<C> {
    fn select_profile(&self) -> GpuPowerProfile {
        if let Some(doc) = TomlDocument::from_file(self.path) {
            if let Some(val) = doc.section_property("graphics", "gpu", true) {
                if let Some(profile) = GpuPowerProfile::from_token(&val) {
                    return profile;
                }
            }
        }
        self.origin.select_profile()
    }
}

/// Yegor Decorator overriding GPU selection via `LOW_CUBE_GPU=low|high` environment variable (`2` fields).
pub struct EnvironmentGpuChoice<C> {
    var_name: &'static str,
    origin: C,
}

impl<C> EnvironmentGpuChoice<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(var_name: &'static str, origin: C) -> Self {
        Self { var_name, origin }
    }
}

impl<C: GpuChoice> GpuChoice for EnvironmentGpuChoice<C> {
    fn select_profile(&self) -> GpuPowerProfile {
        if let Ok(val) = std::env::var(self.var_name) {
            if let Some(profile) = GpuPowerProfile::from_token(&val) {
                return profile;
            }
        }
        self.origin.select_profile()
    }
}

/// Yegor Decorator overriding GPU selection via `--gpu <high|low|igpu|mx150>` CLI flags (`2` private fields).
pub struct ArgumentGpuChoice<C> {
    args: Vec<String>,
    origin: C,
}

impl<C> ArgumentGpuChoice<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(args: Vec<String>, origin: C) -> Self {
        Self { args, origin }
    }

    pub fn from_process(origin: C) -> Self {
        Self::new(std::env::args().collect(), origin)
    }
}

impl<C: GpuChoice> GpuChoice for ArgumentGpuChoice<C> {
    fn select_profile(&self) -> GpuPowerProfile {
        let mut iter = self.args.iter();
        while let Some(arg) = iter.next() {
            if arg == "--gpu" {
                if let Some(val) = iter.next() {
                    if let Some(p) = GpuPowerProfile::from_token(val) {
                        return p;
                    }
                }
            } else if let Some(val) = arg.strip_prefix("--gpu=") {
                if let Some(p) = GpuPowerProfile::from_token(val) {
                    return p;
                }
            }
        }
        self.origin.select_profile()
    }
}

/// Assembled Optimus GPU selector decorator chain (`CLI -> ENV -> settings.toml -> HighPerformance`, `1` field).
pub struct ServerGpuChoice {
    chain: ArgumentGpuChoice<EnvironmentGpuChoice<FileGpuChoice<AcPowerChoice>>>,
}

impl ServerGpuChoice {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        chain: ArgumentGpuChoice<EnvironmentGpuChoice<FileGpuChoice<AcPowerChoice>>>,
    ) -> Self {
        Self { chain }
    }

    pub fn assembled() -> Self {
        Self::new(ArgumentGpuChoice::from_process(EnvironmentGpuChoice::new(
            "LOW_CUBE_GPU",
            FileGpuChoice::new("settings.toml", AcPowerChoice::new()),
        )))
    }
}

impl GpuChoice for ServerGpuChoice {
    fn select_profile(&self) -> GpuPowerProfile {
        self.chain.select_profile()
    }
}
