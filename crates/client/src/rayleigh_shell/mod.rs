#[cfg(test)]
mod optics;
#[cfg(test)]
pub use optics::{CubicRayleighShell, CubicSolarBody};

use glam::{Vec3, Vec4};
use engine::GraphicPipeline;
use world::{ApexSpin, BodyId, BodySnapshot, CelestialAtlas, ClimateZone, PlanetarySpin};

/// Camera-relative planetary atmosphere envelope (`Earth`, `Mars`, `Venus`, `Titan`; airless = `0.0`).
#[derive(Clone, Copy, Debug)]
pub struct AtmosphericShell {
    rel_cam_and_half: [f32; 4],
    tint_and_thick: [f32; 4],
}

impl AtmosphericShell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(rel_cam_and_half: [f32; 4], tint_and_thick: [f32; 4]) -> Self {
        Self { rel_cam_and_half, tint_and_thick }
    }

    pub const fn vacuum() -> Self {
        Self::new([0.0; 4], [0.0; 4])
    }

    pub const fn terrestrial_at(cam_pos: Vec3) -> Self {
        Self::new([cam_pos.x, cam_pos.y, cam_pos.z, 16.0], [0.21, 0.44, 0.80, 12.0])
    }

    fn body_atmosphere(id: BodyId, half: f32) -> Option<([f32; 3], f32)> {
        if id == BodyId::EARTH {
            Some(([0.18, 0.56, 1.00], (half * 0.30).max(10.0)))
        } else if id == BodyId::MARS {
            Some(([0.86, 0.48, 0.28], (half * 0.22).max(6.0)))
        } else if id == BodyId::VENUS {
            Some(([0.95, 0.78, 0.42], (half * 0.38).max(12.0)))
        } else if id == BodyId::TITAN {
            Some(([0.90, 0.58, 0.20], (half * 0.36).max(8.0)))
        } else {
            None
        }
    }

    pub fn nearest(snaps: &[BodySnapshot], viewer: glam::DVec3) -> Self {
        let mut closest_any: Option<(f64, BodyId, f64)> = None;
        for s in snaps.iter().skip(1) {
            let gap = (viewer - s.center()).abs().max_element() - s.half_size();
            if closest_any.map_or(true, |(b, _, _)| gap < b) {
                closest_any = Some((gap, s.id(), s.half_size()));
            }
        }
        if let Some((gap, id, half)) = closest_any {
            if Self::body_atmosphere(id, half as f32).is_none() && gap <= half * 4.0 {
                return Self::vacuum();
            }
        }
        let mut best: Option<(f64, Self)> = None;
        for s in snaps {
            let h = s.half_size() as f32;
            let Some((tint, thick)) = Self::body_atmosphere(s.id(), h) else {
                continue;
            };
            let rel_cam = viewer - s.center();
            let gap = rel_cam.abs().max_element() - s.half_size();
            if gap <= f64::from(h * 12.0 + thick * 4.0) && best.map_or(true, |(b, _)| gap < b) {
                let rc = rel_cam.as_vec3();
                best = Some((
                    gap,
                    Self::new([rc.x, rc.y, rc.z, h], [tint[0], tint[1], tint[2], thick]),
                ));
            }
        }
        best.map_or_else(Self::vacuum, |(_, shell)| shell)
    }

    /// Evaluates `(local_up, zenith_clear_color, night_factor)` for `sun_dir` at mean 1 AU solar flux.
    pub fn sky_illumination(self, sun_dir: Vec3) -> (Vec3, Vec4, f32) {
        self.sky_illumination_with_flux(sun_dir, 0.00465)
    }

    /// Evaluates `(local_up, zenith_clear_color, night_factor)` scaled by elliptical solar angular half-size.
    pub fn sky_illumination_with_flux(self, sun_dir: Vec3, solar_half_ang: f32) -> (Vec3, Vec4, f32) {
        let rel_body = Vec3::new(self.rel_cam_and_half[0], self.rel_cam_and_half[1], self.rel_cam_and_half[2]);
        let body_up = ClimateZone::dominant(rel_body).normal();
        let space_black = Vec4::new(0.008, 0.012, 0.025, 1.0);
        let thick = self.tint_and_thick[3];
        if thick <= 0.01 || (rel_body.abs().max_element() - self.rel_cam_and_half[3]) > thick * 1.2 {
            return (body_up, space_black, 1.0);
        }
        let flux = (solar_half_ang / 0.00465).clamp(0.80, 1.30);
        let sun_h = sun_dir.dot(body_up);
        let tint = Vec3::new(self.tint_and_thick[0], self.tint_and_thick[1], self.tint_and_thick[2]);
        let day_col = (tint * (1.15 * flux)).clamp(Vec3::ZERO, Vec3::ONE).extend(1.0);
        let sunset_gold = Vec4::new((0.85 * flux).min(1.0), (0.38 * flux).min(1.0), 0.12, 1.0);
        let clear = if sun_h > 0.15 {
            day_col
        } else if sun_h >= 0.0 {
            sunset_gold.lerp(day_col, (sun_h / 0.15).clamp(0.0, 1.0))
        } else if sun_h >= -0.15 {
            space_black.lerp(sunset_gold, ((sun_h + 0.15) / 0.15).clamp(0.0, 1.0))
        } else {
            space_black
        };
        (body_up, clear, (-sun_h / 0.15).clamp(0.0, 1.0))
    }

    pub const fn uniforms(self) -> ([f32; 4], [f32; 4]) {
        (self.rel_cam_and_half, self.tint_and_thick)
    }
}

/// Real-time planet/solar directions `[MERCURY..NEPTUNE, SOL]`, eclipse occluder, and `AtmosphericShell`.
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

    pub fn from_atlas(atlas: CelestialAtlas, viewer: Vec3) -> Self {
        let Some(eph) = atlas.ephemeris() else {
            return Self::absent();
        };
        let viewer_d = viewer.as_dvec3();
        let snaps = eph.snapshot_all(0.0);
        let apparent_sun = ApexSpin::from_ephemeris(eph).apparent_sun_center(snaps[0].center(), viewer_d);
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
        Self::new(cubes, best_occ.1, AtmosphericShell::nearest(&snaps, viewer_d))
    }

    pub fn sync_pipeline(&self, pipeline: &dyn GraphicPipeline) {
        pipeline.sync_ephemeris(&self.planet_cubes, self.occluder, self.atmosphere.uniforms());
    }

    #[cfg(test)]
    pub fn transfer<R>(self, f: impl FnOnce(&[[f32; 4]; 8], [f32; 4]) -> R) -> R {
        f(&self.planet_cubes, self.occluder)
    }

    #[cfg(test)]
    pub const fn atmosphere_uniforms(self) -> ([f32; 4], [f32; 4]) {
        self.atmosphere.uniforms()
    }
}
