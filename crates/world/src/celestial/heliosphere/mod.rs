mod earthworks;
mod facets;
mod vicinity;

use std::cell::RefCell;
use glam::Vec3;
use crate::{CompanionPortal, CubePlanet, GravityField, PlanetaryWorld, TerrainMesh};
use super::catalog::SurfaceNature;
use super::ephemeris::SolarEphemeris;
pub use self::vicinity::{ActiveVicinity, AwakenedBody, VicinityCache};

/// Scaled solar-system world uniting Earth (`0..6`), on-demand off-world `VicinityCache` (`6..12`), `NBodyGravity`, and `AsteroidBelt`.
pub struct HeliosphereWorld {
    pub(super) earth: CubePlanet,
    pub(super) ephemeris: SolarEphemeris,
    pub(super) vicinity: RefCell<VicinityCache>,
}

impl HeliosphereWorld {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        earth: CubePlanet,
        ephemeris: SolarEphemeris,
        vicinity: RefCell<VicinityCache>,
    ) -> Self {
        Self { earth, ephemeris, vicinity }
    }

    pub fn assembled(earth: CubePlanet, ephemeris: SolarEphemeris, authoritative: bool) -> Self {
        Self::new(earth, ephemeris, RefCell::new(VicinityCache::empty(authoritative)))
    }

    pub(super) fn near_earth(&self, pos: Vec3) -> bool {
        pos.abs().max_element() <= self.ephemeris.proportions().earth_half() + 64.0
    }

    pub(super) fn map_earth_slice(idx: usize) -> usize {
        if idx < 6 { idx } else { idx + 6 }
    }

    pub(super) fn map_vicinity_slice(idx: usize) -> usize {
        if idx < 6 { idx + 6 } else { idx + 12 }
    }
}

impl GravityField for HeliosphereWorld {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().active_body() {
            if body.near(position, 48.0) {
                return body.planet().pull_direction_at(position);
            }
        }
        -self.ephemeris.dominant_up_at(position.as_dvec3())
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().active_body() {
            if body.near(position, 48.0) {
                return body.planet().surface_up_at(position);
            }
        }
        self.ephemeris.dominant_up_at(position.as_dvec3())
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().active_body() {
            if body.near(position, 32.0) {
                let g = self
                    .ephemeris
                    .snapshot_all(0.0)
                    .into_iter()
                    .find(|s| s.id() == body.body_id())
                    .map_or(1.62, |s| s.surface_g());
                return body.planet().pull_direction_at(position) * (g * dt);
            }
        }
        (self.ephemeris.gravity_accel_at(position.as_dvec3(), 0.0) * f64::from(dt)).as_vec3()
    }
}

impl CompanionPortal for HeliosphereWorld {}

impl PlanetaryWorld for HeliosphereWorld {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        if self.near_earth(position) {
            return self.earth.constrain_to_surface(position, player_height);
        }
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(position, self.ephemeris) {
            return body.planet().constrain_to_surface(position, player_height);
        }
        position
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        if self.near_earth(position) {
            return self.earth.canopy_drag(position);
        }
        let pd = position.as_dvec3();
        for snap in self.ephemeris.snapshot_all(0.0) {
            if matches!(snap.nature(), SurfaceNature::GasGiant { .. }) {
                let cheb = (pd - snap.center()).abs().max_element();
                let h = snap.half_size();
                if cheb <= h * 1.15 {
                    let depth = ((h * 1.15 - cheb) / (h * 0.45)).clamp(0.0, 4.0);
                    return (depth * 1.8) as f32;
                }
            }
        }
        0.0
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.earth.placed_camera_tripod().or_else(|| {
            self.vicinity.borrow().active_body().and_then(|b| b.planet().placed_camera_tripod())
        })
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let locked = self.vicinity.borrow().active_body().is_some();
        if !locked {
            self.ephemeris = self.ephemeris.advanced(f64::from(dt));
        }
        let a = self.earth.step_surface(dt);
        let b = self
            .vicinity
            .borrow_mut()
            .active_body_mut()
            .is_some_and(|w| w.planet_mut().step_surface(dt));
        a || b || self.pending_slices()
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        if self.near_earth(position) {
            return self.earth.lethal_at(position);
        }
        if let Some(body) = self.vicinity.borrow().active_body() {
            if body.near(position, 8.0) {
                return body.planet().lethal_at(position);
            }
        }
        let pd = position.as_dvec3();
        for snap in self.ephemeris.snapshot_all(0.0) {
            let cheb = (pd - snap.center()).abs().max_element();
            match snap.nature() {
                SurfaceNature::SolarStar if cheb <= snap.half_size() * 1.25 => return true,
                SurfaceNature::GasGiant { .. } if cheb <= snap.half_size() * 0.55 => return true,
                _ => {}
            }
        }
        false
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        if self.near_earth(position) {
            return self.earth.confine_beneath_crust(position);
        }
        if let Some(body) = self.vicinity.borrow().active_body() {
            if body.near(position, 36.0) {
                return body.planet().confine_beneath_crust(position);
            }
        }
        position
    }

    fn local_body_point(&self, position: Vec3) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().active_body() {
            if body.near(position, 48.0) {
                return body.planet().local_body_point(position);
            }
        }
        position
    }

    fn celestial_atlas(&self) -> crate::CelestialAtlas {
        crate::CelestialAtlas::heliosphere(self.ephemeris)
    }
}
