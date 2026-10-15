use glam::Vec3;
use crate::noise::{CoherentNoise, SpatialFrequency};
use crate::topology::ClimateZone;
use super::planting_ground::PlantingGround;

/// Decorator modulating arboreal density across biomes (dense forest, solitary trees, open clearings).
pub struct WoodlandGround<'a> {
    origin: Box<dyn PlantingGround + 'a>,
    frequency: Box<dyn SpatialFrequency>,
    zone: ClimateZone,
}

impl<'a> WoodlandGround<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Box<dyn PlantingGround + 'a>,
        frequency: Box<dyn SpatialFrequency>,
        zone: ClimateZone,
    ) -> Self {
        Self {
            origin,
            frequency,
            zone,
        }
    }

    /// Secondary constructor initializing zone-calibrated macro frequency.
    pub fn in_zone(origin: Box<dyn PlantingGround + 'a>, zone: ClimateZone) -> Self {
        let seed = match zone {
            ClimateZone::Temperate => 701,
            ClimateZone::Volcanic => 702,
            ClimateZone::Desert => 703,
            ClimateZone::Arctic => 704,
            ClimateZone::Jungle => 705,
            ClimateZone::Steppe => 706,
        };
        let noise = Box::new(CoherentNoise::new(seed, 0.04));
        Self::new(origin, noise, zone)
    }
}

impl<'a> PlantingGround for WoodlandGround<'a> {
    fn receptive(&self, cell_u: i32, cell_v: i32, hash: i32) -> bool {
        if !self.origin.receptive(cell_u, cell_v, hash) {
            return false;
        }

        let u = (cell_u as f32) + 0.5;
        let v = (cell_v as f32) + 0.5;
        let sample = self.frequency.sample(Vec3::new(u, v, 0.0));
        let roll = (hash % 100).abs();

        match self.zone {
            ClimateZone::Jungle => {
                if sample > 0.0 {
                    roll < 65
                } else if sample > -0.3 {
                    roll < 20
                } else {
                    false
                }
            }
            ClimateZone::Desert => {
                if sample > 0.4 {
                    roll < 25
                } else if sample > 0.1 {
                    roll < 6
                } else {
                    false
                }
            }
            ClimateZone::Steppe => {
                if sample > 0.35 {
                    roll < 40
                } else if sample > 0.0 {
                    roll < 12
                } else {
                    false
                }
            }
            _ => {
                if sample > 0.25 {
                    roll < 60
                } else if sample > -0.15 {
                    roll < 14
                } else {
                    false
                }
            }
        }
    }

    fn ground_elevation(&self, cell_u: i32, cell_v: i32) -> f32 {
        self.origin.ground_elevation(cell_u, cell_v)
    }
}
