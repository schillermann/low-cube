mod awakened;
mod cosmic_sky;
mod earthworks;
mod facets;
mod scanner;
mod vault;
mod vicinity;

use std::cell::RefCell;
use glam::Vec3;
use crate::{CompanionPortal, GravityField, PlanetaryWorld, QuadSphere, TerrainMesh};
use super::catalog::SurfaceNature;
use super::ephemeris::SolarEphemeris;
pub use self::awakened::{ActiveVicinity, AwakenedBody};
pub use self::vault::OFFWORLD_IDLE_TTL_SECS;
pub use self::vicinity::VicinityCache;

/// Scaled solar-system world uniting `QuadSphere` Earth (`0..6`), dual-slot `VicinityCache`, `NBodyGravity`, and `AsteroidBelt`.
pub struct HeliosphereWorld {
    pub(super) earth: QuadSphere,
    pub(super) ephemeris: SolarEphemeris,
    pub(super) vicinity: RefCell<VicinityCache>,
}

impl HeliosphereWorld {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        earth: QuadSphere,
        ephemeris: SolarEphemeris,
        vicinity: RefCell<VicinityCache>,
    ) -> Self {
        Self { earth, ephemeris, vicinity }
    }

    pub fn assembled(earth: QuadSphere, ephemeris: SolarEphemeris, authoritative: bool) -> Self {
        Self::new(earth, ephemeris, RefCell::new(VicinityCache::empty(authoritative)))
    }

    pub fn warm_offworld_count(&self) -> usize {
        self.vicinity.borrow().warm_body_count()
    }

    pub fn cold_chronicle_count(&self, id: crate::BodyId) -> usize {
        self.vicinity.borrow().cold_chronicle_count(id)
    }

    pub(super) fn near_earth(&self, pos: Vec3) -> bool {
        if !self.ephemeris.frame().earth_centered() {
            return false;
        }
        let half = self.ephemeris.proportions().earth_half();
        let band = (half * 0.45).clamp(64.0, 160.0);
        pos.length() <= half + band
    }

    pub(super) const fn map_earth_slice(idx: usize) -> usize {
        if idx < 6 { idx } else { idx + 6 }
    }

    pub(super) const fn map_slot_slice(slot: usize, face_idx: usize) -> Option<usize> {
        if face_idx >= 6 {
            None
        } else if slot == 0 {
            Some(face_idx + 6)
        } else {
            Some(face_idx + 18)
        }
    }
}

impl GravityField for HeliosphereWorld {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().body_near(position, 48.0) {
            return body.planet().pull_direction_at(position);
        }
        -self.ephemeris.dominant_up_at(position.as_dvec3())
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().body_near(position, 48.0) {
            return body.planet().surface_up_at(position);
        }
        self.ephemeris.dominant_up_at(position.as_dvec3())
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().body_near(position, 32.0) {
            let g = self
                .ephemeris
                .snapshot_all(0.0)
                .into_iter()
                .find(|s| s.id() == body.body_id())
                .map_or(1.62, |s| s.surface_g());
            return body.planet().pull_direction_at(position) * (g * dt);
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

    fn surface_contact(&self, probe: Vec3, clearance: f32) -> crate::SurfaceContact {
        if self.near_earth(probe) {
            return self.earth.surface_contact(probe, clearance);
        }
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(probe, self.ephemeris) {
            let contact = body.planet().surface_contact(probe, clearance);
            if contact.landable() {
                return contact;
            }
        }
        let drag = self.canopy_drag(probe);
        if drag > 0.0 {
            crate::SurfaceContact::GasEnvelope { drag }
        } else {
            crate::SurfaceContact::Vacuum
        }
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        if self.near_earth(position) {
            return self.earth.canopy_drag(position);
        }
        let pd = position.as_dvec3();
        for snap in self.ephemeris.snapshot_all(0.0) {
            if matches!(snap.nature(), SurfaceNature::GasGiant { .. }) {
                let dist = (pd - snap.center()).length();
                let h = snap.half_size();
                if dist <= h * 1.15 {
                    let depth = ((h * 1.15 - dist) / (h * 0.45)).clamp(0.0, 4.0);
                    return (depth * 1.8) as f32;
                }
            }
        }
        0.0
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.earth.placed_camera_tripod().or_else(|| {
            let vic = self.vicinity.borrow();
            [0usize, 1]
                .into_iter()
                .filter_map(|i| vic.slot_body(i))
                .find_map(|b| b.planet().placed_camera_tripod())
        })
    }

    fn collect_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        self.earth.collect_medium_sites(medium, out);
    }

    fn collect_offworld_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(crate::BodyId, Vec3, Vec3)>) {
        self.vicinity
            .borrow()
            .collect_offworld_medium_sites(medium, self.ephemeris.proportions(), out);
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        self.step_active_zones(dt, 0b0011_1111, &[])
    }

    fn step_active_zones(
        &mut self,
        dt: f32,
        earth_zone_mask: u8,
        active_bodies: &[crate::BodyId],
    ) -> bool {
        self.ephemeris = self.ephemeris.advanced(f64::from(dt));
        self.vicinity.borrow_mut().recenter_active(self.ephemeris);
        let a = if self.ephemeris.frame().earth_centered() || earth_zone_mask != 0 {
            self.earth.step_active_zones(dt, earth_zone_mask, active_bodies)
        } else {
            false
        };
        let b = self.vicinity.borrow_mut().step_active_bodies(dt, active_bodies);
        a || b || self.pending_slices()
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        if self.near_earth(position) {
            return self.earth.lethal_at(position);
        }
        if let Some(body) = self.vicinity.borrow().body_near(position, 8.0) {
            return body.planet().lethal_at(position);
        }
        let pd = position.as_dvec3();
        for snap in self.ephemeris.snapshot_all(0.0) {
            let dist = (pd - snap.center()).length();
            match snap.nature() {
                SurfaceNature::SolarStar if dist <= snap.half_size() * 1.25 => return true,
                SurfaceNature::GasGiant { .. } if dist <= snap.half_size() * 0.55 => return true,
                _ => {}
            }
        }
        false
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        if self.near_earth(position) {
            return self.earth.confine_beneath_crust(position);
        }
        if let Some(body) = self.vicinity.borrow().body_near(position, 36.0) {
            return body.planet().confine_beneath_crust(position);
        }
        position
    }

    fn local_body_point(&self, position: Vec3) -> Vec3 {
        if let Some(body) = self.vicinity.borrow().body_near(position, 48.0) {
            return body.planet().local_body_point(position);
        }
        position
    }

    fn celestial_atlas(&self) -> crate::CelestialAtlas {
        crate::CelestialAtlas::heliosphere(self.ephemeris)
    }

    fn cosmic_frame(&self) -> crate::CosmicFrame {
        self.ephemeris.frame()
    }

    fn align_epoch(&mut self, epoch_secs: f64) {
        self.ephemeris = self.ephemeris.at_epoch(epoch_secs);
        self.vicinity.borrow_mut().recenter_active(self.ephemeris);
    }

    fn anchor_to_frame(&mut self, frame: crate::CosmicFrame, local_pos: Vec3) {
        self.ephemeris = self.ephemeris.reanchored(frame);
        let mut vic = self.vicinity.borrow_mut();
        vic.recenter_active(self.ephemeris);
        let _ = vic.ensure_near(local_pos, self.ephemeris);
        vic.recenter_active(self.ephemeris);
    }

    fn reconcile_cosmic_frame(&mut self, local_pos: Vec3, local_vel: Vec3) -> Option<crate::FrameHandoff> {
        let handoff = super::bubble::GravityBubble::from_ephemeris(self.ephemeris)
            .evaluate_handoff(self.ephemeris, local_pos, local_vel)?;
        self.ephemeris = self.ephemeris.reanchored(handoff.next_frame());
        {
            let mut vic = self.vicinity.borrow_mut();
            vic.recenter_active(self.ephemeris);
            let _ = vic.ensure_near(handoff.next_pos(), self.ephemeris);
            vic.recenter_active(self.ephemeris);
        }
        Some(handoff)
    }
}
