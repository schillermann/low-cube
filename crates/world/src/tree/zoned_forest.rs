use glam::Vec3;
use crate::{ClimateZone, Face, SurfaceFrame, WorldSeed};
use super::arboreal::Arboreal;
use super::arctic_pine::ArcticPine;
use super::desert_succulent::DesertSucculent;
use super::full_cube_ground::FullCubeGround;
use super::jungle_emergent::JungleEmergent;
use super::meridian_ground::MeridianGround;
use super::planting_ground::PlantingGround;
use super::seeded_ground::SeededGround;
use super::solid_arboreal::SolidArboreal;
use super::spawn_ground::SpawnSanctuaryGround;
use super::species::TreeSpecies;
use super::standing::StandingTree;
use super::steppe_acacia::SteppeAcacia;
use super::surface_ground::SurfaceGround;
use super::temperate_oak::TemperateOak;
use super::volcanic_spire::VolcanicSpire;
use super::woodland_ground::WoodlandGround;

/// Biome-adapted forest canopy tailored to a specific climate zone.
pub struct ZonedForest {
    zone: ClimateZone,
    seed: WorldSeed,
}

impl ZonedForest {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn seeded(zone: ClimateZone, seed: WorldSeed) -> Self {
        Self { zone, seed }
    }

    /// Secondary constructor initializing the canonical origin-seeded forest canopy.
    pub const fn new(zone: ClimateZone) -> Self {
        Self::seeded(zone, WorldSeed::origin())
    }

    /// Plants a native standing tree specimen within this climate zone.
    pub fn plant(&self, frame: SurfaceFrame, u: f32, v: f32, elevation: f32) -> Box<dyn Arboreal> {
        let species = self.native_species();
        let tree = Box::new(StandingTree::of_species(species, frame, u, v, elevation));
        Box::new(SolidArboreal::rooted(tree, u, v, elevation, 0.65, 3.0))
    }

    fn species_drag(&self, dist: f32, relative_h: f32) -> f32 {
        match self.zone {
            ClimateZone::Temperate => TemperateOak::new().canopy_drag(dist, relative_h),
            ClimateZone::Arctic => ArcticPine::new().canopy_drag(dist, relative_h),
            ClimateZone::Jungle => JungleEmergent::new().canopy_drag(dist, relative_h),
            ClimateZone::Desert => DesertSucculent::new().canopy_drag(dist, relative_h),
            ClimateZone::Steppe => SteppeAcacia::new().canopy_drag(dist, relative_h),
            ClimateZone::Volcanic => VolcanicSpire::new().canopy_drag(dist, relative_h),
        }
    }

    fn sector_bounds(half_size: f32, (gsu, gsv): (i32, i32)) -> Option<(i32, i32, i32, i32, i32)> {
        let min_coord = (-(half_size - 4.0) * 2.0).floor() as i32;
        let max_coord = ((half_size - 4.0) * 2.0).floor() as i32;
        let step = if half_size >= 48.0 { 8 } else { 6 };
        let (u0, u1) = (gsu * 8, gsu * 8 + 7);
        let (v0, v1) = (gsv * 8, gsv * 8 + 7);
        if u1 < min_coord || u0 > max_coord || v1 < min_coord || v0 > max_coord {
            return None;
        }
        let start_u = min_coord + ((u0 - min_coord).max(0) + step - 1) / step * step;
        let start_v = min_coord + ((v0 - min_coord).max(0) + step - 1) / step * step;
        Some((start_u, u1.min(max_coord), start_v, v1.min(max_coord), step))
    }

    /// Zero-allocation stack evaluation of untouched trunk collision inside `8×8` sector `(gsu, gsv)`.
    pub fn constrain_untouched_sector(
        &self,
        mut pos: Vec3,
        (u, v, distance): (f32, f32, f32),
        origin: &dyn Face,
        frame: &SurfaceFrame,
        half_size: f32,
        key: (i32, i32),
    ) -> Vec3 {
        let Some((start_u, end_u, start_v, end_v, step)) = Self::sector_bounds(half_size, key) else {
            return pos;
        };
        let surface_ground = SurfaceGround::new(origin, frame, half_size);
        let full_cube_ground = FullCubeGround::new(surface_ground, origin, frame, half_size);
        let woodland_ground = WoodlandGround::in_zone(full_cube_ground, self.zone);
        let seeded_ground = SeededGround::new(woodland_ground, self.seed);
        let meridian_ground = MeridianGround::new(seeded_ground, 1.2);
        let ground = SpawnSanctuaryGround::new(meridian_ground, self.zone, 4.5);

        let mut cell_u = start_u;
        while cell_u <= end_u {
            let tree_u = (cell_u as f32) * 0.5 + 0.25;
            if (u - tree_u).abs() <= 0.65 {
                let mut cell_v = start_v;
                while cell_v <= end_v {
                    let tree_v = (cell_v as f32) * 0.5 + 0.25;
                    if (u - tree_u).hypot(v - tree_v) <= 0.65 {
                        let hash = cell_u.wrapping_mul(73856093) ^ cell_v.wrapping_mul(19349663);
                        if ground.receptive(cell_u, cell_v, hash) {
                            let base_elev = ground.ground_elevation(cell_u, cell_v);
                            let cur_elev = frame.elevation_of(pos);
                            let support = cur_elev - distance;
                            if support >= base_elev - 0.45 && support <= base_elev + 3.0 {
                                pos = frame.point_at(u, v, cur_elev + 3.0);
                            }
                        }
                    }
                    cell_v += step;
                }
            }
            cell_u += step;
        }
        pos
    }

    /// Zero-allocation stack evaluation of untouched canopy drag inside `8×8` sector `(gsu, gsv)`.
    pub fn canopy_drag_untouched_sector(
        &self,
        (u, v, elev): (f32, f32, f32),
        mut peak: f32,
        origin: &dyn Face,
        frame: &SurfaceFrame,
        half_size: f32,
        key: (i32, i32),
    ) -> f32 {
        let Some((start_u, end_u, start_v, end_v, step)) = Self::sector_bounds(half_size, key) else {
            return peak;
        };
        let surface_ground = SurfaceGround::new(origin, frame, half_size);
        let full_cube_ground = FullCubeGround::new(surface_ground, origin, frame, half_size);
        let woodland_ground = WoodlandGround::in_zone(full_cube_ground, self.zone);
        let seeded_ground = SeededGround::new(woodland_ground, self.seed);
        let meridian_ground = MeridianGround::new(seeded_ground, 1.2);
        let ground = SpawnSanctuaryGround::new(meridian_ground, self.zone, 4.5);

        let mut cell_u = start_u;
        while cell_u <= end_u {
            let tree_u = (cell_u as f32) * 0.5 + 0.25;
            if (u - tree_u).abs() <= 4.0 {
                let mut cell_v = start_v;
                while cell_v <= end_v {
                    let tree_v = (cell_v as f32) * 0.5 + 0.5 * 0.5;
                    let dist = (u - tree_u).hypot(v - tree_v);
                    if dist <= 4.0 {
                        let hash = cell_u.wrapping_mul(73856093) ^ cell_v.wrapping_mul(19349663);
                        if ground.receptive(cell_u, cell_v, hash) {
                            let root_elev = ground.ground_elevation(cell_u, cell_v);
                            peak = peak.max(self.species_drag(dist, elev - root_elev));
                        }
                    }
                    cell_v += step;
                }
            }
            cell_u += step;
        }
        peak
    }

    /// Populates native standing trees within a single `8×8`-cell sector `(gsu, gsv)`.
    pub fn populate_sector(
        &self,
        origin: &dyn Face,
        frame: SurfaceFrame,
        half_size: f32,
        key: (i32, i32),
    ) -> Vec<Box<dyn Arboreal>> {
        let Some((start_u, end_u, start_v, end_v, step)) = Self::sector_bounds(half_size, key) else {
            return Vec::new();
        };
        let mut trees = Vec::new();
        let surface_ground = SurfaceGround::new(origin, &frame, half_size);
        let full_cube_ground = FullCubeGround::new(surface_ground, origin, &frame, half_size);
        let woodland_ground = WoodlandGround::in_zone(full_cube_ground, self.zone);
        let seeded_ground = SeededGround::new(woodland_ground, self.seed);
        let meridian_ground = MeridianGround::new(seeded_ground, 1.2);
        let ground = SpawnSanctuaryGround::new(meridian_ground, self.zone, 4.5);

        let mut cell_u = start_u;
        while cell_u <= end_u {
            let mut cell_v = start_v;
            while cell_v <= end_v {
                let raw_hash = cell_u.wrapping_mul(73856093) ^ cell_v.wrapping_mul(19349663);
                if ground.receptive(cell_u, cell_v, raw_hash) {
                    let tree_u = (cell_u as f32) * 0.5 + 0.25;
                    let tree_v = (cell_v as f32) * 0.5 + 0.25;
                    let elevation = ground.ground_elevation(cell_u, cell_v);
                    trees.push(self.plant(frame, tree_u, tree_v, elevation));
                }
                cell_v += step;
            }
            cell_u += step;
        }
        trees
    }

    /// Populates native trees and gatherable deadwood/flint litter across an entire compact planetary face.
    pub fn populate_face(&self, origin: &dyn Face, frame: SurfaceFrame, half_size: f32) -> Vec<Box<dyn Arboreal>> {
        let half_sec = ((half_size * 2.0).ceil() as i32).div_euclid(8) + 1;
        let mut trees = Vec::new();
        for gsu in -half_sec..=half_sec {
            for gsv in -half_sec..=half_sec {
                trees.extend(self.populate_sector(origin, frame, half_size, (gsu, gsv)));
            }
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
