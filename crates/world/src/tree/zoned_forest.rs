use crate::{ClimateZone, Face, SurfaceFrame};
use super::arboreal::Arboreal;
use super::arctic_pine::ArcticPine;
use super::desert_succulent::DesertSucculent;
use super::floor_litter::FloorLitter;
use super::full_cube_ground::FullCubeGround;
use super::jungle_emergent::JungleEmergent;
use super::meridian_ground::MeridianGround;
use super::planting_ground::PlantingGround;
use super::solid_arboreal::SolidArboreal;
use super::spawn_ground::SpawnSanctuaryGround;
use super::species::TreeSpecies;
use super::standing::StandingTree;
use super::steppe_acacia::SteppeAcacia;
use super::surface_ground::SurfaceGround;
use super::temperate_oak::TemperateOak;
use super::timber_yield::SurfaceFind;
use super::volcanic_spire::VolcanicSpire;
use super::woodland_ground::WoodlandGround;

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

    /// Populates native trees and gatherable deadwood/flint litter across a planetary face.
    pub fn populate_face(&self, origin: &dyn Face, frame: SurfaceFrame, half_size: f32) -> Vec<Box<dyn Arboreal>> {
        let mut trees = Vec::new();
        let surface_ground = Box::new(SurfaceGround::new(origin, &frame, half_size));
        let full_cube_ground = Box::new(FullCubeGround::new(surface_ground, origin, &frame, half_size));
        let woodland_ground = Box::new(WoodlandGround::in_zone(full_cube_ground, self.zone));
        let meridian_ground = Box::new(MeridianGround::new(woodland_ground, 1.2));
        let ground = SpawnSanctuaryGround::new(meridian_ground, self.zone, 4.5);

        let border_margin = 4.0;
        let min_coord = (-(half_size - border_margin) * 2.0).floor() as i32;
        let max_coord = ((half_size - border_margin) * 2.0).floor() as i32;
        let step = if half_size >= 48.0 { 8 } else { 6 };

        let mut cell_u = min_coord;
        while cell_u <= max_coord {
            let mut cell_v = min_coord;
            while cell_v <= max_coord {
                let hash = cell_u.wrapping_mul(73856093) ^ cell_v.wrapping_mul(19349663);
                let tree_u = (cell_u as f32) * 0.5 + 0.25;
                let tree_v = (cell_v as f32) * 0.5 + 0.25;
                let elevation = ground.ground_elevation(cell_u, cell_v);
                if ground.receptive(cell_u, cell_v, hash) {
                    trees.push(self.plant(frame, tree_u, tree_v, elevation));
                    let (du, dv) = match hash.unsigned_abs() & 3 {
                        0 => (2, 0),
                        1 => (-2, 0),
                        2 => (0, 2),
                        _ => (0, -2),
                    };
                    let (bu, bv) = (cell_u + du, cell_v + dv);
                    let bough_elev = ground.ground_elevation(bu, bv);
                    let yaw = ((hash.unsigned_abs() % 628) as f32) * 0.01;
                    trees.push(Box::new(FloorLitter::new(
                        frame,
                        (bu as f32) * 0.5 + 0.25,
                        (bv as f32) * 0.5 + 0.25,
                        bough_elev,
                        SurfaceFind::DeadwoodBough,
                        yaw,
                    )));
                } else if tree_u.hypot(tree_v) >= 2.5 {
                    let near_rock = elevation >= 0.40
                        || (ground.ground_elevation(cell_u, cell_v + 2) - elevation).abs() >= 0.20
                        || (ground.ground_elevation(cell_u, cell_v - 2) - elevation).abs() >= 0.20
                        || (ground.ground_elevation(cell_u + 2, cell_v) - elevation).abs() >= 0.20
                        || (ground.ground_elevation(cell_u - 2, cell_v) - elevation).abs() >= 0.20;
                    if near_rock && (hash.unsigned_abs() % 100) < 48 {
                        let yaw = ((hash.unsigned_abs() % 628) as f32) * 0.01;
                        trees.push(Box::new(FloorLitter::new(frame, tree_u, tree_v, elevation, SurfaceFind::FlintStone, yaw)));
                    }
                }
                cell_v += step;
            }
            cell_u += step;
        }
        trees
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
