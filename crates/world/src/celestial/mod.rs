mod alien_worlds;
mod atlas;
mod belt;
mod binary;
mod brad64;
mod bubble;
mod catalog;
mod cosmic_frame;
mod ephemeris;
mod gravity;
mod heliosphere;
mod kepler;
mod lunar;
mod orbital_fleet;
mod proportions;
mod sky_signal;
mod spin;
mod translated;

use glam::Vec3;
use crate::{BiomedQuadSphere, GravityField, PlanetaryWorld};

pub use self::alien_worlds::AlienWorld;
pub use self::atlas::{AtlasMode, CelestialAtlas};
pub use self::belt::{AsteroidBelt, BeltRock};
pub use self::binary::BinaryPlanet;
pub use self::brad64::Brad64;
pub use self::bubble::{CosmicFrame, FrameHandoff, GravityBubble, DEEP_SPACE_SECTOR_SPAN};
pub use self::catalog::{
    BodyId, BodyPhysique, OrbitalSpec, SolarCatalog, SurfaceNature, MAJOR_BODY_COUNT,
};
pub use self::ephemeris::{BodyKinematics, BodySnapshot, SolarEphemeris};
pub use self::gravity::{BinaryGravity, RadialWell};
pub use self::heliosphere::{
    ActiveVicinity, AwakenedBody, HeliosphereWorld, VicinityCache, OFFWORLD_IDLE_TTL_SECS,
};
pub use self::kepler::{EllipticShape, KeplerAnomaly, OrbitElements, OrbitalLocus, OrbitalPlane};
pub use self::lunar::{LunarCraterRelief, LunarWorld};
pub use self::orbital_fleet::OrbitalFleetWorld;
pub use self::proportions::{BeltAndReach, SolarProportions};
pub use self::sky_signal::{SignalKind, SkySignal};
pub use self::spin::{ApexSpin, PlanetarySpin};
pub use self::translated::TranslatedWorld;

/// Spherical radial gravity field resolving $C^\infty$ outward surface-up vectors around a `QuadSphere` origin.
#[derive(Clone, Copy, Debug)]
pub struct ZoneGravity;

impl ZoneGravity {
    /// Primary constructor initializing the zero-state spherical radial normal field.
    pub const fn new() -> Self {
        Self
    }
}

impl GravityField for ZoneGravity {
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        position.try_normalize().unwrap_or(Vec3::Y)
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}

type SystemGravity = BinaryGravity<RadialWell<ZoneGravity>, RadialWell<ZoneGravity>>;

/// Factory assembling the complete Earth-Moon `OrbitalFleetWorld<BinaryPlanet<...>>` decorator hierarchy.
pub struct CelestialSystem;

impl CelestialSystem {
    /// Orbital center of the 32-block Moon `QuadSphere` high above the northern Temperate horizon.
    pub const fn moon_center(home_blocks: u32) -> Vec3 {
        let half = (home_blocks as f32) * 0.5;
        Vec3::new(0.0, half + 256.0, -96.0)
    }

    fn compose_gravity(home_half: f32, moon_center: Vec3) -> SystemGravity {
        let home_well = RadialWell::homeworld(ZoneGravity::new(), home_half);
        let moon_well = RadialWell::lunar(ZoneGravity::new(), moon_center, LunarWorld::HALF_SIZE);
        BinaryGravity::new(home_well, moon_well)
    }

    /// Authoritative Earth-Moon system with living subterranean mineral veins on both `QuadSphere` worlds.
    pub fn authoritative(blocks: u32, _margin: f32) -> Box<dyn PlanetaryWorld> {
        let home = BiomedQuadSphere::of_size(blocks);
        let home_half = (blocks as f32) * 0.5;
        let moon_pos = Self::moon_center(blocks);
        let moon = TranslatedWorld::new(LunarWorld::authoritative(), moon_pos);
        let gravity = Self::compose_gravity(home_half, moon_pos);
        Box::new(OrbitalFleetWorld::wrapping(BinaryPlanet::new(home, moon, gravity, home_half)))
    }

    /// Barren flora-free Earth-Moon system for fast headless physics and multiplayer tests (`< 3ms`).
    pub fn barren_authoritative(blocks: u32, _margin: f32) -> Box<dyn PlanetaryWorld> {
        let home = BiomedQuadSphere::barren(blocks);
        let home_half = (blocks as f32) * 0.5;
        let moon_pos = Self::moon_center(blocks);
        let moon = TranslatedWorld::new(LunarWorld::authoritative(), moon_pos);
        let gravity = Self::compose_gravity(home_half, moon_pos);
        Box::new(OrbitalFleetWorld::wrapping(BinaryPlanet::new(home, moon, gravity, home_half)))
    }

    /// Zero-knowledge client Earth-Moon system without unexposed subterranean ores.
    pub fn hollow(blocks: u32, _margin: f32) -> Box<dyn PlanetaryWorld> {
        let home = BiomedQuadSphere::hollow_of_size(blocks);
        let home_half = (blocks as f32) * 0.5;
        let moon_pos = Self::moon_center(blocks);
        let moon = TranslatedWorld::new(LunarWorld::hollow(), moon_pos);
        let gravity = Self::compose_gravity(home_half, moon_pos);
        Box::new(OrbitalFleetWorld::wrapping(BinaryPlanet::new(home, moon, gravity, home_half)))
    }

    /// Authoritative full Solar System (`HeliosphereWorld`) scaled to `proportions`.
    pub fn heliosphere(proportions: SolarProportions, _margin: f32) -> Box<dyn PlanetaryWorld> {
        let earth = BiomedQuadSphere::seeded(proportions.earth_blocks(), proportions.seed());
        let eph = SolarEphemeris::new(proportions, 0.0);
        Box::new(OrbitalFleetWorld::wrapping(HeliosphereWorld::assembled(earth, eph, true)))
    }

    /// Flora-free authoritative full Solar System for fast physics, slingshot, and map tests.
    pub fn barren_heliosphere(proportions: SolarProportions, _margin: f32) -> Box<dyn PlanetaryWorld> {
        let earth = BiomedQuadSphere::barren_seeded(proportions.earth_blocks(), proportions.seed());
        let eph = SolarEphemeris::new(proportions, 0.0);
        Box::new(OrbitalFleetWorld::wrapping(HeliosphereWorld::assembled(earth, eph, true)))
    }

    /// Zero-knowledge client full Solar System without unexposed subterranean ores.
    pub fn hollow_heliosphere(proportions: SolarProportions, _margin: f32) -> Box<dyn PlanetaryWorld> {
        let earth = BiomedQuadSphere::hollow_seeded(proportions.earth_blocks(), proportions.seed());
        let eph = SolarEphemeris::new(proportions, 0.0);
        Box::new(OrbitalFleetWorld::wrapping(HeliosphereWorld::assembled(earth, eph, false)))
    }
}
