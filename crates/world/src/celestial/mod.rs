mod alien_worlds;
mod atlas;
mod belt;
mod binary;
mod catalog;
mod ephemeris;
mod gravity;
mod heliosphere;
mod kepler;
mod lunar;
mod orbital_fleet;
mod proportions;
mod spin;
mod translated;

use glam::Vec3;
use crate::{BeveledGravity, BiomedCubeWorld, ClimateZone, GravityField, PlanetaryWorld};

pub use self::alien_worlds::AlienWorld;
pub use self::atlas::{AtlasMode, CelestialAtlas};
pub use self::belt::{AsteroidBelt, BeltRock};
pub use self::binary::BinaryPlanet;
pub use self::catalog::{
    BodyId, BodyPhysique, OrbitalSpec, SolarCatalog, SurfaceNature, MAJOR_BODY_COUNT,
};
pub use self::ephemeris::{BodyKinematics, BodySnapshot, SolarEphemeris};
pub use self::gravity::{BinaryGravity, RadialWell};
pub use self::heliosphere::{ActiveVicinity, AwakenedBody, HeliosphereWorld, VicinityCache};
pub use self::kepler::{EllipticShape, KeplerAnomaly, OrbitElements, OrbitalLocus, OrbitalPlane};
pub use self::lunar::{LunarCraterRelief, LunarWorld};
pub use self::orbital_fleet::OrbitalFleetWorld;
pub use self::proportions::{BeltAndReach, BodyRatios, SolarProportions};
pub use self::spin::{ApexSpin, PlanetarySpin};
pub use self::translated::TranslatedWorld;

/// Cardinal cubic gravity field resolving the dominant climate-zone normal around an origin.
#[derive(Clone, Copy, Debug)]
pub struct ZoneGravity;

impl ZoneGravity {
    /// Primary constructor initializing the zero-state cubic normal field.
    pub const fn new() -> Self {
        Self
    }
}

impl GravityField for ZoneGravity {
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        ClimateZone::dominant(position).normal()
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}

type SystemGravity = BinaryGravity<
    RadialWell<BeveledGravity<ZoneGravity>>,
    RadialWell<BeveledGravity<ZoneGravity>>,
>;

/// Factory assembling the complete Earth-Moon `OrbitalFleetWorld<BinaryPlanet<...>>` decorator hierarchy.
pub struct CelestialSystem;

impl CelestialSystem {
    /// Orbital center of the 32-block Moon-Cube high above the northern Temperate horizon.
    pub const fn moon_center(home_blocks: u32) -> Vec3 {
        let half = (home_blocks as f32) * 0.5;
        Vec3::new(0.0, half + 256.0, -96.0)
    }

    fn compose_gravity(home_half: f32, margin: f32, moon_center: Vec3) -> SystemGravity {
        let home_well = RadialWell::homeworld(
            BeveledGravity::decorating(ZoneGravity::new(), home_half, margin),
            home_half,
        );
        let moon_well = RadialWell::lunar(
            BeveledGravity::decorating(ZoneGravity::new(), LunarWorld::HALF_SIZE, 1.0),
            moon_center,
            LunarWorld::HALF_SIZE,
        );
        BinaryGravity::new(home_well, moon_well)
    }

    /// Authoritative Earth-Moon system with living subterranean mineral veins on both worlds.
    pub fn authoritative(blocks: u32, margin: f32) -> Box<dyn PlanetaryWorld> {
        let home = BiomedCubeWorld::beveled(blocks, margin);
        let home_half = (blocks as f32) * 0.5;
        let moon_pos = Self::moon_center(blocks);
        let moon = TranslatedWorld::new(LunarWorld::authoritative(), moon_pos);
        let gravity = Self::compose_gravity(home_half, margin, moon_pos);
        Box::new(OrbitalFleetWorld::wrapping(BinaryPlanet::new(home, moon, gravity, home_half)))
    }

    /// Barren flora-free Earth-Moon system for fast headless physics and multiplayer tests (`< 3ms`).
    pub fn barren_authoritative(blocks: u32, margin: f32) -> Box<dyn PlanetaryWorld> {
        let home = BiomedCubeWorld::barren_beveled(blocks, margin);
        let home_half = (blocks as f32) * 0.5;
        let moon_pos = Self::moon_center(blocks);
        let moon = TranslatedWorld::new(LunarWorld::authoritative(), moon_pos);
        let gravity = Self::compose_gravity(home_half, margin, moon_pos);
        Box::new(OrbitalFleetWorld::wrapping(BinaryPlanet::new(home, moon, gravity, home_half)))
    }

    /// Zero-knowledge client Earth-Moon system without unexposed subterranean ores.
    pub fn hollow(blocks: u32, margin: f32) -> Box<dyn PlanetaryWorld> {
        let home = BiomedCubeWorld::hollow_beveled(blocks, margin);
        let home_half = (blocks as f32) * 0.5;
        let moon_pos = Self::moon_center(blocks);
        let moon = TranslatedWorld::new(LunarWorld::hollow(), moon_pos);
        let gravity = Self::compose_gravity(home_half, margin, moon_pos);
        Box::new(OrbitalFleetWorld::wrapping(BinaryPlanet::new(home, moon, gravity, home_half)))
    }

    /// Authoritative full Solar System (`HeliosphereWorld`) scaled to `proportions`.
    pub fn heliosphere(proportions: SolarProportions, margin: f32) -> Box<dyn PlanetaryWorld> {
        let earth = BiomedCubeWorld::beveled(proportions.earth_blocks(), margin);
        let eph = SolarEphemeris::new(proportions, 0.0);
        Box::new(OrbitalFleetWorld::wrapping(HeliosphereWorld::assembled(earth, eph, true)))
    }

    /// Flora-free authoritative full Solar System for fast physics, slingshot, and map tests.
    pub fn barren_heliosphere(proportions: SolarProportions, margin: f32) -> Box<dyn PlanetaryWorld> {
        let earth = BiomedCubeWorld::barren_beveled(proportions.earth_blocks(), margin);
        let eph = SolarEphemeris::new(proportions, 0.0);
        Box::new(OrbitalFleetWorld::wrapping(HeliosphereWorld::assembled(earth, eph, true)))
    }

    /// Zero-knowledge client full Solar System without unexposed subterranean ores.
    pub fn hollow_heliosphere(proportions: SolarProportions, margin: f32) -> Box<dyn PlanetaryWorld> {
        let earth = BiomedCubeWorld::hollow_beveled(proportions.earth_blocks(), margin);
        let eph = SolarEphemeris::new(proportions, 0.0);
        Box::new(OrbitalFleetWorld::wrapping(HeliosphereWorld::assembled(earth, eph, false)))
    }
}
