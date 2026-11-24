use glam::Vec3;
use world::{CelestialAtlas, PlanetaryWorld};
use super::TransferArc;

/// Polymorphic spacecraft navigation state avoiding `Option<TransferArc>`.
#[derive(Clone, Copy, Debug)]
pub enum NavigationCourse {
    Uncharted,
    Charted(TransferArc),
}

impl NavigationCourse {
    pub fn sync_ephemeris(&mut self, atlas: CelestialAtlas) {
        if let Self::Charted(ref mut arc) = self {
            *arc = arc.synced_with_atlas(atlas);
        }
    }

    pub fn shift_frame(&mut self, delta: Vec3) {
        if let Self::Charted(ref mut arc) = self {
            *arc = arc.shifted_by(delta);
        }
    }

    pub fn mark_along(
        &mut self,
        (boresight, gimbal): ((Vec3, Vec3), (Vec3, Vec3)),
        (ship_origin, velocity): (Vec3, Vec3),
        world: &dyn PlanetaryWorld,
    ) -> bool {
        let atlas = world.celestial_atlas();
        let hit = atlas
            .lock_sight_surface(boresight.0, boresight.1, world)
            .or_else(|| atlas.lock_sight_surface(gimbal.0, gimbal.1, world));
        if let Some((hit_pos, hit_norm)) = hit {
            if let Self::Charted(prev) = self {
                if !prev.retargets(hit_pos) {
                    *self = Self::Uncharted;
                    return true;
                }
            }
            *self = Self::Charted(TransferArc::charted(ship_origin, velocity, hit_pos, hit_norm, world));
            true
        } else {
            *self = Self::Uncharted;
            true
        }
    }
}
