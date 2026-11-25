use std::env;
use world::SolarProportions;
use super::SanctuaryToml;

/// Resolver coupling `SolarProportions` directly to `earth_blocks` (`[world] size = 1..=32`) plus optional `asteroid_density`.
pub(super) struct ServerCosmosScale<'a> {
    args: &'a [String],
    toml: &'a SanctuaryToml,
}

impl<'a> ServerCosmosScale<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(args: &'a [String], toml: &'a SanctuaryToml) -> Self {
        Self { args, toml }
    }

    fn parse_density(raw: &str) -> Option<f32> {
        raw.trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .map(|v| v.min(8.0))
    }

    fn apply_toml(&self, p: SolarProportions) -> SolarProportions {
        self.toml
            .section_property("cosmos", "asteroid_density", false)
            .or_else(|| self.toml.section_property("cosmos", "belt_density", false))
            .and_then(|s| Self::parse_density(&s))
            .map_or(p, |v| p.with_belt_density(v))
    }

    fn apply_env(p: SolarProportions) -> SolarProportions {
        env::var("LOW_CUBE_ASTEROID_DENSITY")
            .ok()
            .and_then(|s| Self::parse_density(&s))
            .map_or(p, |v| p.with_belt_density(v))
    }

    fn apply_cli(&self, mut p: SolarProportions) -> SolarProportions {
        let mut i = 1;
        while i < self.args.len() {
            let arg = self.args[i].as_str();
            if let Some(val) = arg
                .strip_prefix("--asteroid-density=")
                .or_else(|| arg.strip_prefix("--belt-density="))
            {
                if let Some(v) = Self::parse_density(val) {
                    p = p.with_belt_density(v);
                }
                i += 1;
            } else if matches!(arg, "--asteroid-density" | "--belt-density") && i + 1 < self.args.len() {
                if let Some(v) = Self::parse_density(&self.args[i + 1]) {
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
    fn test_dedicated_server_cosmos_couples_to_world_size_and_cli_belt_density() {
        let toml = SanctuaryToml::new(
            "[world]\nsize = 8\n[cosmos]\nasteroid_density = 2.5\n".to_string(),
        );
        let base_args = vec!["low-cube-server".to_string()];
        let from_toml = SanctuaryPlan::assembled(&base_args, &toml).proportions();
        assert_eq!(from_toml.earth_blocks(), 1024);
        assert!((from_toml.distance_scale() - 1.0).abs() < 1e-5);
        assert!((from_toml.belt_density() - 2.5).abs() < 1e-5);

        let cli_args = vec![
            "low-cube-server".to_string(),
            "--size=4".to_string(),
            "--asteroid-density".to_string(),
            "0.4".to_string(),
        ];
        let overridden = SanctuaryPlan::assembled(&cli_args, &toml).proportions();
        assert_eq!(overridden.earth_blocks(), 512);
        assert!((overridden.belt_density() - 0.4).abs() < 1e-5);
    }
}
