use world::SolarProportions;
use crate::command_flags::CommandFlags;
use crate::server_scale::{PlanetaryScale, ServerScale};
use crate::toml_document::TomlDocument;

/// Interface resolving configurable solar-system proportions without exposing mutable configuration bags.
pub trait CosmosScale: Send + Sync {
    fn proportions(&self) -> SolarProportions;
}

/// Polymorphic asteroid-belt density override avoiding `Option<f32>` fields inside configuration decorators.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BeltDensity {
    Inherited,
    Specified(f32),
}

impl BeltDensity {
    fn parse(raw: &str, source: &str) -> Self {
        let cleaned = raw.trim().trim_matches('"').trim_matches('\'').trim();
        let val = cleaned.parse::<f32>().unwrap_or_else(|_| {
            panic!("Invalid asteroid density '{raw}' in {source}: expected number >= 0")
        });
        assert!(
            val.is_finite() && val >= 0.0,
            "Invalid asteroid density '{raw}' in {source}: must be >= 0"
        );
        Self::Specified(val)
    }

    fn apply(self, current: SolarProportions) -> SolarProportions {
        match self {
            Self::Inherited => current,
            Self::Specified(density) => current.with_belt_density(density),
        }
    }
}

/// Origin `CosmosScale` deriving 1:1 true astronomical proportions from a `PlanetaryScale` (`size = 1..=32`).
pub struct MeasuredCosmos {
    scale: Box<dyn PlanetaryScale>,
}

impl MeasuredCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(scale: Box<dyn PlanetaryScale>) -> Self {
        Self { scale }
    }
}

impl CosmosScale for MeasuredCosmos {
    fn proportions(&self) -> SolarProportions {
        SolarProportions::true_scale(self.scale.blocks())
    }
}

/// TOML-driven `CosmosScale` decorator reading `[cosmos] asteroid_density` from `server.toml` or `client.toml`.
pub struct FileCosmos {
    origin: Box<dyn CosmosScale>,
    belt_density: BeltDensity,
}

impl FileCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn CosmosScale>, belt_density: BeltDensity) -> Self {
        Self { origin, belt_density }
    }

    pub fn from_path(path: &str, fallback: Box<dyn CosmosScale>) -> Self {
        let Some(doc) = TomlDocument::from_file(path) else {
            return Self::new(fallback, BeltDensity::Inherited);
        };
        Self::from_document(&doc, path, fallback)
    }

    pub fn from_document(doc: &TomlDocument, source: &str, fallback: Box<dyn CosmosScale>) -> Self {
        let prop = |key: &str| {
            doc.section_property("cosmos", key, false)
                .or_else(|| doc.section_property("world", key, false))
        };
        let belt = prop("asteroid_density")
            .or_else(|| prop("belt_density"))
            .map_or(BeltDensity::Inherited, |v| BeltDensity::parse(&v, source));
        Self::new(fallback, belt)
    }
}

impl CosmosScale for FileCosmos {
    fn proportions(&self) -> SolarProportions {
        self.belt_density.apply(self.origin.proportions())
    }
}

/// Environment-variable `CosmosScale` decorator reading `LOW_CUBE_ASTEROID_DENSITY`.
pub struct EnvironmentCosmos {
    origin: Box<dyn CosmosScale>,
    belt_density: BeltDensity,
}

impl EnvironmentCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn CosmosScale>, belt_density: BeltDensity) -> Self {
        Self { origin, belt_density }
    }

    pub fn process(fallback: Box<dyn CosmosScale>) -> Self {
        let belt = std::env::var("LOW_CUBE_ASTEROID_DENSITY")
            .ok()
            .map_or(BeltDensity::Inherited, |v| {
                BeltDensity::parse(&v, "LOW_CUBE_ASTEROID_DENSITY")
            });
        Self::new(fallback, belt)
    }
}

impl CosmosScale for EnvironmentCosmos {
    fn proportions(&self) -> SolarProportions {
        self.belt_density.apply(self.origin.proportions())
    }
}

/// CLI-argument `CosmosScale` decorator reading `--asteroid-density` (or `--belt-density`).
pub struct ArgumentCosmos {
    origin: Box<dyn CosmosScale>,
    belt_density: BeltDensity,
}

impl ArgumentCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn CosmosScale>, belt_density: BeltDensity) -> Self {
        Self { origin, belt_density }
    }

    pub fn process(fallback: Box<dyn CosmosScale>) -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::from_arguments(&args, fallback)
    }

    pub fn from_arguments(args: &[String], fallback: Box<dyn CosmosScale>) -> Self {
        let flags = CommandFlags::from_slice(args);
        let belt = flags
            .value_of("--asteroid-density")
            .or_else(|| flags.value_of("--belt-density"))
            .map_or(BeltDensity::Inherited, |v| {
                BeltDensity::parse(&v, "--asteroid-density")
            });
        Self::new(fallback, belt)
    }
}

impl CosmosScale for ArgumentCosmos {
    fn proportions(&self) -> SolarProportions {
        self.belt_density.apply(self.origin.proportions())
    }
}

/// `CosmosScale` decorator clamping `distance_scale` so the `3.20 AU` outer Asteroid Belt stays `< 1h` by Epstein-Drive.
pub struct EpsteinReachCosmos<C> {
    origin: C,
}

impl<C> EpsteinReachCosmos<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: C) -> Self {
        Self { origin }
    }
}

impl<C: CosmosScale> CosmosScale for EpsteinReachCosmos<C> {
    fn proportions(&self) -> SolarProportions {
        let raw = self.origin.proportions();
        let natural_au = (raw.earth_blocks().max(8) as f64) * 11_740.0;
        let max_dist_factor = (SolarProportions::EPSTEIN_HOUR_AU_CAP / natural_au) as f32;
        if raw.distance_scale() > max_dist_factor {
            raw.with_distance_scale(max_dist_factor)
        } else {
            raw
        }
    }
}

/// Full decorator chain (`EpsteinReach > CLI > Env > client.toml > server.toml > MeasuredCosmos`) resolving `SolarProportions`.
pub struct AssembledCosmos {
    origin: Box<dyn CosmosScale>,
}

impl AssembledCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn CosmosScale>) -> Self {
        Self { origin }
    }

    pub fn assembled() -> Self {
        let measured = Box::new(MeasuredCosmos::new(Box::new(ServerScale::assembled())));
        let server_toml = Box::new(FileCosmos::from_path("server.toml", measured));
        let client_toml = Box::new(FileCosmos::from_path("client.toml", server_toml));
        let env_layer = Box::new(EnvironmentCosmos::process(client_toml));
        let cli_layer = ArgumentCosmos::process(env_layer);
        Self::new(Box::new(EpsteinReachCosmos::new(cli_layer)))
    }
}

impl CosmosScale for AssembledCosmos {
    fn proportions(&self) -> SolarProportions {
        self.origin.proportions()
    }
}
