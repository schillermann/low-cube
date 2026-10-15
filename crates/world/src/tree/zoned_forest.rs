use crate::{ClimateZone, SurfaceFrame};
use super::arboreal::Arboreal;
use super::arctic_pine::ArcticPine;
use super::desert_succulent::DesertSucculent;
use super::jungle_emergent::JungleEmergent;
use super::solid_arboreal::SolidArboreal;
use super::species::TreeSpecies;
use super::standing::StandingTree;
use super::steppe_acacia::SteppeAcacia;
use super::temperate_oak::TemperateOak;
use super::volcanic_spire::VolcanicSpire;

/// Biome-adapted forest canopy tailored to a specific climate zone.
pub struct ZonedForest {
    zone: ClimateZone,
}

impl ZonedForest {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(zone: ClimateZone) -> Self {
        Self { zone }
    }

    /// Plants a native standing tree specimen within this climate zone.
    pub fn plant(&self, frame: SurfaceFrame, u: f32, v: f32, elevation: f32) -> Box<dyn Arboreal> {
        let species = self.native_species();
        let tree = Box::new(StandingTree::of_species(species, frame, u, v, elevation));
        Box::new(SolidArboreal::new(tree, u, v, 0.65, 3.0))
    }

    /// Assembles the native tree species instance for this climate zone.
    pub fn native_species(&self) -> Box<dyn TreeSpecies> {
        match self.zone {
            ClimateZone::Temperate => Box::new(TemperateOak::new()),
            ClimateZone::Arctic => Box::new(ArcticPine::new()),
            ClimateZone::Jungle => Box::new(JungleEmergent::new()),
            ClimateZone::Desert => Box::new(DesertSucculent::new()),
            ClimateZone::Steppe => Box::new(SteppeAcacia::new()),
            ClimateZone::Volcanic => Box::new(VolcanicSpire::new()),
        }
    }
}
