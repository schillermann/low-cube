use std::env;
use world::SolarProportions;
use super::SanctuaryToml;

/// Resolver applying `[cosmos]` (`server.toml`), `LOW_CUBE_*` environment, and CLI scale overrides.
pub(super) struct ServerCosmosScale<'a> {
    args: &'a [String],
    toml: &'a SanctuaryToml,
}

impl<'a> ServerCosmosScale<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(args: &'a [String], toml: &'a SanctuaryToml) -> Self {
        Self { args, toml }
    }

    fn parse_positive(raw: &str, min: f32, max: f32) -> Option<f32> {
        raw.trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= min)
            .map(|v| v.min(max))
    }

    fn apply_toml(&self, mut p: SolarProportions) -> SolarProportions {
        if let Some(v) = self
            .toml
            .section_property("cosmos", "scale", false)
            .or_else(|| self.toml.section_property("cosmos", "cosmos_scale", false))
            .and_then(|s| Self::parse_positive(&s, 0.0005, 8.0))
        {
            p = p.with_system_scale(v);
        }
        if let Some(v) = self
            .toml
            .section_property("cosmos", "distance_scale", false)
            .and_then(|s| Self::parse_positive(&s, 0.0005, 8.0))
        {
            p = p.with_distance_scale(v);
        }
        if let Some(v) = self
            .toml
            .section_property("cosmos", "body_scale", false)
            .and_then(|s| Self::parse_positive(&s, 0.02, 8.0))
        {
            p = p.with_body_scale(v);
        }
        if let Some(v) = self
            .toml
            .section_property("cosmos", "sun_scale", false)
            .and_then(|s| Self::parse_positive(&s, 0.01, 8.0))
        {
            p = p.with_sun_scale(v);
        }
        if let Some(v) = self
            .toml
            .section_property("cosmos", "asteroid_density", false)
            .or_else(|| self.toml.section_property("cosmos", "belt_density", false))
            .and_then(|s| Self::parse_positive(&s, 0.0, 8.0))
        {
            p = p.with_belt_density(v);
        }
        p
    }

    fn apply_env(mut p: SolarProportions) -> SolarProportions {
        if let Ok(s) = env::var("LOW_CUBE_COSMOS_SCALE") {
            if let Some(v) = Self::parse_positive(&s, 0.0005, 8.0) {
                p = p.with_system_scale(v);
            }
        }
        if let Ok(s) = env::var("LOW_CUBE_DISTANCE_SCALE") {
            if let Some(v) = Self::parse_positive(&s, 0.0005, 8.0) {
                p = p.with_distance_scale(v);
            }
        }
        if let Ok(s) = env::var("LOW_CUBE_BODY_SCALE") {
            if let Some(v) = Self::parse_positive(&s, 0.02, 8.0) {
                p = p.with_body_scale(v);
            }
        }
        if let Ok(s) = env::var("LOW_CUBE_SUN_SCALE") {
            if let Some(v) = Self::parse_positive(&s, 0.01, 8.0) {
                p = p.with_sun_scale(v);
            }
        }
        if let Ok(s) = env::var("LOW_CUBE_ASTEROID_DENSITY") {
            if let Some(v) = Self::parse_positive(&s, 0.0, 8.0) {
                p = p.with_belt_density(v);
            }
        }
        p
    }

    fn apply_cli(&self, mut p: SolarProportions) -> SolarProportions {
        let mut i = 1;
        while i < self.args.len() {
            let arg = self.args[i].as_str();
            if let Some(val) = arg.strip_prefix("--cosmos-scale=") {
                if let Some(v) = Self::parse_positive(val, 0.0005, 8.0) {
                    p = p.with_system_scale(v);
                }
                i += 1;
            } else if arg == "--cosmos-scale" && i + 1 < self.args.len() {
                if let Some(v) = Self::parse_positive(&self.args[i + 1], 0.0005, 8.0) {
                    p = p.with_system_scale(v);
                }
                i += 2;
            } else if let Some(val) = arg.strip_prefix("--distance-scale=") {
                if let Some(v) = Self::parse_positive(val, 0.0005, 8.0) {
                    p = p.with_distance_scale(v);
                }
                i += 1;
            } else if arg == "--distance-scale" && i + 1 < self.args.len() {
                if let Some(v) = Self::parse_positive(&self.args[i + 1], 0.0005, 8.0) {
                    p = p.with_distance_scale(v);
                }
                i += 2;
            } else if let Some(val) = arg.strip_prefix("--body-scale=") {
                if let Some(v) = Self::parse_positive(val, 0.02, 8.0) {
                    p = p.with_body_scale(v);
                }
                i += 1;
            } else if arg == "--body-scale" && i + 1 < self.args.len() {
                if let Some(v) = Self::parse_positive(&self.args[i + 1], 0.02, 8.0) {
                    p = p.with_body_scale(v);
                }
                i += 2;
            } else if let Some(val) = arg.strip_prefix("--sun-scale=") {
                if let Some(v) = Self::parse_positive(val, 0.01, 8.0) {
                    p = p.with_sun_scale(v);
                }
                i += 1;
            } else if arg == "--sun-scale" && i + 1 < self.args.len() {
                if let Some(v) = Self::parse_positive(&self.args[i + 1], 0.01, 8.0) {
                    p = p.with_sun_scale(v);
                }
                i += 2;
            } else if let Some(val) = arg
                .strip_prefix("--asteroid-density=")
                .or_else(|| arg.strip_prefix("--belt-density="))
            {
                if let Some(v) = Self::parse_positive(val, 0.0, 8.0) {
                    p = p.with_belt_density(v);
                }
                i += 1;
            } else if matches!(arg, "--asteroid-density" | "--belt-density") && i + 1 < self.args.len() {
                if let Some(v) = Self::parse_positive(&self.args[i + 1], 0.0, 8.0) {
                    p = p.with_belt_density(v);
                }
                i += 2;
            } else {
                i += 1;
            }
        }
        p
    }

    pub(super) fn resolve(&self, earth_blocks: u32) -> SolarProportions {
        let from_toml = self.apply_toml(SolarProportions::true_scale(earth_blocks));
        let from_env = Self::apply_env(from_toml);
        self.apply_cli(from_env)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SanctuaryPlan, SanctuaryToml};

    #[test]
    fn test_dedicated_server_cosmos_toml_and_cli_override_precedence() {
        let toml = SanctuaryToml::new(
            "[world]\nsize = 1\n[cosmos]\ndistance_scale = 0.02\nbody_scale = 1.5\nsun_scale = 0.8\nasteroid_density = 2.5\n"
                .to_string(),
        );
        let base_args = vec!["low-cube-server".to_string()];
        let from_toml = SanctuaryPlan::assembled(&base_args, &toml).proportions();
        assert!((from_toml.distance_scale() - 0.02).abs() < 1e-5);
        assert!((from_toml.body_scale() - 1.5).abs() < 1e-5);
        assert!((from_toml.sun_scale() - 0.8).abs() < 1e-5);
        assert!((from_toml.belt_density() - 2.5).abs() < 1e-5);

        let cli_args = vec![
            "low-cube-server".to_string(),
            "--distance-scale=0.005".to_string(),
            "--asteroid-density".to_string(),
            "0.4".to_string(),
        ];
        let overridden = SanctuaryPlan::assembled(&cli_args, &toml).proportions();
        assert!((overridden.distance_scale() - 0.005).abs() < 1e-5);
        assert!((overridden.body_scale() - 1.5).abs() < 1e-5);
        assert!((overridden.belt_density() - 0.4).abs() < 1e-5);
    }
}
