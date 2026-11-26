use glam::Vec3;
use engine::GraphicPipeline;
use world::{ApexSpin, BodyId, CelestialAtlas, PlanetarySpin};
use super::catalog::{AirCatalog, SolarAirCatalog};
use super::{AtmosphericEnvelope, AtmosphericShell};

/// Focused contract synchronizing projected planetary ephemeris state into a `GraphicPipeline`.
pub trait EphemerisSync {
    fn sync_pipeline(&self, pipeline: &dyn GraphicPipeline);
}

/// Focused contract projecting a `CelestialAtlas` around a viewer position into `FirmamentPlanets`.
pub trait SkyProjection {
    fn project_from(&self, viewer: Vec3) -> FirmamentPlanets;
}

/// Real-time planet/solar directions `[MERCURY..NEPTUNE, SOL]`, eclipse shadow, and `AtmosphericShell`.
#[derive(Clone, Copy, Debug)]
pub struct FirmamentPlanets {
    planet_cubes: [[f32; 4]; 8],
    occluder: [f32; 4],
    atmosphere: AtmosphericShell,
}

impl FirmamentPlanets {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        planet_cubes: [[f32; 4]; 8],
        occluder: [f32; 4],
        atmosphere: AtmosphericShell,
    ) -> Self {
        Self { planet_cubes, occluder, atmosphere }
    }

    pub const fn absent() -> Self {
        Self::new([[0.0; 4]; 8], [0.0; 4], AtmosphericShell::vacuum())
    }

    #[cfg(test)]
    pub fn transfer<R>(self, sink: impl FnOnce(&[[f32; 4]; 8], [f32; 4]) -> R) -> R {
        sink(&self.planet_cubes, self.occluder)
    }

    #[cfg(test)]
    pub fn sky_illumination(self, sun_dir: Vec3) -> (Vec3, glam::Vec4, f32) {
        self.atmosphere.sky_illumination(sun_dir)
    }
}

impl EphemerisSync for FirmamentPlanets {
    fn sync_pipeline(&self, pipeline: &dyn GraphicPipeline) {
        self.atmosphere.transfer_shell_quad(|quad| {
            pipeline.sync_ephemeris(&self.planet_cubes, self.occluder, quad);
        });
    }
}

/// Ephemeris sky projector resolving planetary cubes, solar eclipse shadow, and nearest `AtmosphericShell`.
#[derive(Clone, Copy, Debug)]
pub struct AtlasSkyProjection<C> {
    atlas: CelestialAtlas,
    catalog: C,
}

impl<C> AtlasSkyProjection<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(atlas: CelestialAtlas, catalog: C) -> Self {
        Self { atlas, catalog }
    }
}

impl AtlasSkyProjection<SolarAirCatalog> {
    /// Secondary constructor pairing `atlas` with `SolarAirCatalog`.
    pub const fn solar(atlas: CelestialAtlas) -> Self {
        Self::new(atlas, SolarAirCatalog::new())
    }
}

impl<C: AirCatalog> SkyProjection for AtlasSkyProjection<C> {
    fn project_from(&self, viewer: Vec3) -> FirmamentPlanets {
        let Some(eph) = self.atlas.ephemeris() else {
            return FirmamentPlanets::absent();
        };
        let viewer_d = viewer.as_dvec3();
        let snaps = eph.snapshot_all(0.0);
        let apparent_sun =
            ApexSpin::from_ephemeris(eph).apparent_sun_center(snaps[0].center(), viewer_d);
        let to_sun = apparent_sun - viewer_d;
        let sun_dist = to_sun.length().max(1.0);
        let sun_dir = (to_sun / sun_dist).as_vec3();
        let sun_half_ang = ((snaps[0].half_size() / sun_dist) as f32).clamp(0.0008, 0.55);

        let order = [
            BodyId::MERCURY,
            BodyId::VENUS,
            BodyId::MARS,
            BodyId::JUPITER,
            BodyId::SATURN,
            BodyId::URANUS,
            BodyId::NEPTUNE,
        ];
        let mut cubes = [[0.0_f32; 4]; 8];
        for (slot, target_id) in order.into_iter().enumerate() {
            if let Some(s) = snaps.iter().find(|b| b.id() == target_id) {
                let rel = s.center() - viewer_d;
                let dist = rel.length().max(1.0);
                let dir = (rel / dist).as_vec3();
                cubes[slot] = [dir.x, dir.y, dir.z, ((s.half_size() / dist) as f32).clamp(0.00002, 0.45)];
            }
        }
        cubes[7] = [sun_dir.x, sun_dir.y, sun_dir.z, sun_half_ang];

        let mut best_occ = (-2.0_f32, [0.0_f32; 4]);
        for s in snaps.iter().skip(1) {
            let rel = s.center() - viewer_d;
            let dist = rel.length();
            if dist <= s.half_size() * 1.05 || dist >= sun_dist * 0.98 {
                continue;
            }
            let dir = (rel / dist).as_vec3();
            let align = dir.dot(sun_dir);
            if align > best_occ.0 {
                best_occ = (align, [dir.x, dir.y, dir.z, ((s.half_size() / dist) as f32).clamp(0.0005, 0.55)]);
            }
        }
        FirmamentPlanets::new(cubes, best_occ.1, self.catalog.nearest_shell(&snaps, viewer_d))
    }
}
