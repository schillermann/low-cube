use world::SolarProportions;
use crate::command_flags::CommandFlags;
use crate::server_scale::{PlanetaryScale, ServerScale};
use crate::toml_document::TomlDocument;

/// Interface resolving configurable solar-system proportions without exposing mutable configuration bags.
pub trait CosmosScale: Send + Sync {
    fn proportions(&self) -> SolarProportions;
}

/// Polymorphic override state avoiding `Option<f32>` fields inside configuration decorators.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScaleOverride {
    Inherited,
    Specified(f32),
}

impl ScaleOverride {
    fn parse_positive(raw: &str, source: &str) -> Self {
        let cleaned = raw.trim().trim_matches('"').trim_matches('\'').trim();
        let val = cleaned.parse::<f32>().unwrap_or_else(|_| {
            panic!("Invalid cosmos scale '{raw}' in {source}: expected positive number")
        });
        assert!(val.is_finite() && val > 0.0, "Invalid cosmos scale '{raw}' in {source}: must be > 0");
        Self::Specified(val)
    }

    fn parse_non_negative(raw: &str, source: &str) -> Self {
        let cleaned = raw.trim().trim_matches('"').trim_matches('\'').trim();
        let val = cleaned.parse::<f32>().unwrap_or_else(|_| {
            panic!("Invalid asteroid density '{raw}' in {source}: expected number >= 0")
        });
        assert!(val.is_finite() && val >= 0.0, "Invalid asteroid density '{raw}' in {source}: must be >= 0");
        Self::Specified(val)
    }

    fn apply_with(self, current: SolarProportions, f: fn(SolarProportions, f32) -> SolarProportions) -> SolarProportions {
        match self {
            Self::Inherited => current,
            Self::Specified(factor) => f(current, factor),
        }
    }

    const fn or_fallback(self, fallback: Self) -> Self {
        match self {
            Self::Inherited => fallback,
            Self::Specified(_) => self,
        }
    }
}

/// Four-factor encapsulated override bundle (`<= 4` fields) applied by file, environment, or CLI decorators.
#[derive(Clone, Copy, Debug)]
pub struct CosmosOverrides {
    body_scale: ScaleOverride,
    sun_scale: ScaleOverride,
    distance_scale: ScaleOverride,
    belt_density: ScaleOverride,
}

impl CosmosOverrides {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        body_scale: ScaleOverride,
        sun_scale: ScaleOverride,
        distance_scale: ScaleOverride,
        belt_density: ScaleOverride,
    ) -> Self {
        Self { body_scale, sun_scale, distance_scale, belt_density }
    }

    pub const fn inherited() -> Self {
        Self::new(
            ScaleOverride::Inherited,
            ScaleOverride::Inherited,
            ScaleOverride::Inherited,
            ScaleOverride::Inherited,
        )
    }

    fn apply(self, base: SolarProportions) -> SolarProportions {
        let s1 = self.body_scale.apply_with(base, SolarProportions::with_body_scale);
        let s2 = self.sun_scale.apply_with(s1, SolarProportions::with_sun_scale);
        let s3 = self.distance_scale.apply_with(s2, SolarProportions::with_distance_scale);
        self.belt_density.apply_with(s3, SolarProportions::with_belt_density)
    }
}

/// Origin `CosmosScale` deriving 1:1 true astronomical proportions from a `PlanetaryScale`.
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

/// TOML-driven `CosmosScale` decorator reading `[cosmos]` from `server.toml` or `client.toml`.
pub struct FileCosmos {
    origin: Box<dyn CosmosScale>,
    overrides: CosmosOverrides,
}

impl FileCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn CosmosScale>, overrides: CosmosOverrides) -> Self {
        Self { origin, overrides }
    }

    pub fn from_path(path: &str, fallback: Box<dyn CosmosScale>) -> Self {
        let Some(doc) = TomlDocument::from_file(path) else {
            return Self::new(fallback, CosmosOverrides::inherited());
        };
        Self::from_document(&doc, path, fallback)
    }

    pub fn from_document(doc: &TomlDocument, source: &str, fallback: Box<dyn CosmosScale>) -> Self {
        let prop = |key: &str| doc.section_property("cosmos", key, false).or_else(|| doc.section_property("world", key, false));
        let sys = doc
            .section_property("cosmos", "scale", false)
            .or_else(|| prop("cosmos_scale"))
            .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_positive(&v, source));
        let body = prop("body_scale")
            .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_positive(&v, source))
            .or_fallback(sys);
        let sun = prop("sun_scale")
            .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_positive(&v, source))
            .or_fallback(sys);
        let dist = prop("distance_scale")
            .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_positive(&v, source))
            .or_fallback(sys);
        let belt = prop("asteroid_density")
            .or_else(|| prop("belt_density"))
            .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_non_negative(&v, source));
        Self::new(fallback, CosmosOverrides::new(body, sun, dist, belt))
    }
}

impl CosmosScale for FileCosmos {
    fn proportions(&self) -> SolarProportions {
        self.overrides.apply(self.origin.proportions())
    }
}

/// Environment-variable `CosmosScale` decorator reading `LOW_CUBE_*_SCALE` and `LOW_CUBE_ASTEROID_DENSITY`.
pub struct EnvironmentCosmos {
    origin: Box<dyn CosmosScale>,
    overrides: CosmosOverrides,
}

impl EnvironmentCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn CosmosScale>, overrides: CosmosOverrides) -> Self {
        Self { origin, overrides }
    }

    pub fn process(fallback: Box<dyn CosmosScale>) -> Self {
        let pos = |var: &str| {
            std::env::var(var)
                .ok()
                .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_positive(&v, var))
        };
        let belt = std::env::var("LOW_CUBE_ASTEROID_DENSITY")
            .ok()
            .map_or(ScaleOverride::Inherited, |v| {
                ScaleOverride::parse_non_negative(&v, "LOW_CUBE_ASTEROID_DENSITY")
            });
        let sys = pos("LOW_CUBE_COSMOS_SCALE");
        Self::new(
            fallback,
            CosmosOverrides::new(
                pos("LOW_CUBE_BODY_SCALE").or_fallback(sys),
                pos("LOW_CUBE_SUN_SCALE").or_fallback(sys),
                pos("LOW_CUBE_DISTANCE_SCALE").or_fallback(sys),
                belt,
            ),
        )
    }
}

impl CosmosScale for EnvironmentCosmos {
    fn proportions(&self) -> SolarProportions {
        self.overrides.apply(self.origin.proportions())
    }
}

/// CLI-argument `CosmosScale` decorator reading `--body-scale`, `--sun-scale`, `--distance-scale`, and `--asteroid-density`.
pub struct ArgumentCosmos {
    origin: Box<dyn CosmosScale>,
    overrides: CosmosOverrides,
}

impl ArgumentCosmos {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn CosmosScale>, overrides: CosmosOverrides) -> Self {
        Self { origin, overrides }
    }

    pub fn process(fallback: Box<dyn CosmosScale>) -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::from_arguments(&args, fallback)
    }

    pub fn from_arguments(args: &[String], fallback: Box<dyn CosmosScale>) -> Self {
        let flags = CommandFlags::from_slice(args);
        let pos = |flag: &str| {
            flags
                .value_of(flag)
                .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_positive(&v, flag))
        };
        let belt = flags
            .value_of("--asteroid-density")
            .or_else(|| flags.value_of("--belt-density"))
            .map_or(ScaleOverride::Inherited, |v| ScaleOverride::parse_non_negative(&v, "--asteroid-density"));
        let sys = pos("--cosmos-scale");
        Self::new(
            fallback,
            CosmosOverrides::new(
                pos("--body-scale").or_fallback(sys),
                pos("--sun-scale").or_fallback(sys),
                pos("--distance-scale").or_fallback(sys),
                belt,
            ),
        )
    }
}

impl CosmosScale for ArgumentCosmos {
    fn proportions(&self) -> SolarProportions {
        self.overrides.apply(self.origin.proportions())
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
