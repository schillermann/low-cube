use world::{BiomedCubeWorld, PlanetaryWorld, VolumePlanet};

/// Polymorphic planetary world assembly mechanism ("Tell, Don't Ask").
pub trait WorldAssembly {
    /// Assembles a concrete planetary world of the requested edge block count.
    fn assemble_world(&self, blocks: u32) -> Box<dyn PlanetaryWorld>;
}

/// Assembles a multi-biome planar face planetary world.
pub struct BiomedAssembly;

impl WorldAssembly for BiomedAssembly {
    fn assemble_world(&self, blocks: u32) -> Box<dyn PlanetaryWorld> {
        Box::new(BiomedCubeWorld::of_size(blocks))
    }
}

/// Assembles a 3D volume-partitioned voxel planet with greedy meshing and hollow core.
pub struct VolumeAssembly;

impl WorldAssembly for VolumeAssembly {
    fn assemble_world(&self, blocks: u32) -> Box<dyn PlanetaryWorld> {
        Box::new(VolumePlanet::of_size(blocks))
    }
}

/// Command-line argument driven planetary world assembly selector.
pub struct ArgumentWorld {
    arguments: Vec<String>,
    fallback: Box<dyn WorldAssembly>,
}

impl ArgumentWorld {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(arguments: Vec<String>, fallback: Box<dyn WorldAssembly>) -> Self {
        Self { arguments, fallback }
    }

    /// Secondary constructor capturing process environment arguments.
    pub fn process(fallback: Box<dyn WorldAssembly>) -> Self {
        Self::new(std::env::args().collect(), fallback)
    }

    fn matches_flag(&self, flag: &str, target: &str) -> bool {
        self.arguments.windows(2).any(|w| w[0] == flag && w[1] == target)
            || self.arguments.iter().any(|arg| arg == &format!("{}={}", flag, target))
    }
}

impl WorldAssembly for ArgumentWorld {
    fn assemble_world(&self, blocks: u32) -> Box<dyn PlanetaryWorld> {
        if self.matches_flag("--world", "biome") {
            BiomedAssembly.assemble_world(blocks)
        } else {
            self.fallback.assemble_world(blocks)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_assembly_selection() {
        let fallback = Box::new(VolumeAssembly);
        let world_arg = ArgumentWorld::new(vec!["low-cube".into(), "--world".into(), "biome".into()], fallback);
        let planet = world_arg.assemble_world(32);
        assert_eq!(planet.pull_direction_at(glam::Vec3::new(0.0, 20.0, 0.0)), glam::Vec3::NEG_Y);

        let fallback_default = Box::new(VolumeAssembly);
        let default_arg = ArgumentWorld::new(vec!["low-cube".into()], fallback_default);
        let default_planet = default_arg.assemble_world(32);
        assert_eq!(default_planet.pull_direction_at(glam::Vec3::new(0.0, 20.0, 0.0)), glam::Vec3::NEG_Y);
    }
}
