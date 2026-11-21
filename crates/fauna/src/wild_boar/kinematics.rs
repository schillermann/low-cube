use glam::Vec3;
use world::{ClimateZone, PlanetaryWorld};
use crate::roe_deer::TemperateGlade;

/// Physical terrain collaboration keeping wild boars grounded on Temperate soil and blocked by walls or cliffs.
pub(super) struct BoarTerrain<'a> {
    world: &'a dyn PlanetaryWorld,
}

impl<'a> BoarTerrain<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(world: &'a dyn PlanetaryWorld) -> Self {
        Self { world }
    }

    pub(super) fn planar_dist_sq(&self, a: Vec3, b: Vec3) -> f32 {
        if (a.y - b.y).abs() > 6.5 {
            1e9
        } else {
            (a.x - b.x) * (a.x - b.x) + (a.z - b.z) * (a.z - b.z)
        }
    }

    /// Reconciles `pos` beneath oak crowns (`+1.15m`) with live surface height, returning `None` if buried under structures.
    pub(super) fn reconcile_ground(&self, pos: Vec3, up: Vec3) -> Option<Vec3> {
        let surface = self.world.constrain_to_surface(pos + up * 1.15, 0.0);
        if (surface - pos).dot(up).abs() <= 1.25 && TemperateGlade::temperate_surface(surface, self.world) {
            Some(surface)
        } else {
            None
        }
    }

    /// Evaluates whether `listener` lies within `range` across unobstructed Temperate ground beneath tree canopies.
    pub(super) fn threat_reachable(&self, boar_pos: Vec3, listener: Vec3, range: f32) -> bool {
        if self.planar_dist_sq(boar_pos, listener) >= range * range {
            return false;
        }
        let probe_y = listener.y.clamp(boar_pos.y - 1.5, boar_pos.y + 1.35);
        let target_ground = self.world.constrain_to_surface(Vec3::new(listener.x, probe_y, listener.z), 0.0);
        let stance_height = listener.y - target_ground.y;
        if !(-0.85..=2.25).contains(&stance_height) || (target_ground.y - boar_pos.y).abs() > 2.6 {
            return false;
        }
        let mid = (boar_pos + target_ground) * 0.5;
        let avg_y = (boar_pos.y + target_ground.y) * 0.5;
        let flat = Vec3::new(target_ground.x - boar_pos.x, 0.0, target_ground.z - boar_pos.z).normalize_or_zero();
        let side = Vec3::new(-flat.z, 0.0, flat.x);
        [Vec3::ZERO, side * 0.65, -side * 0.65].into_iter().any(|off| {
            let g = self.world.constrain_to_surface(Vec3::new(mid.x + off.x, avg_y + 1.15, mid.z + off.z), 0.0);
            (g.y - avg_y).abs() <= 1.35
        })
    }

    /// Evaluates whether a charging boar at `boar_pos` physically reaches `listener` without walls, roofs, or shafts in between.
    pub(super) fn tusk_reaches(&self, boar_pos: Vec3, listener: Vec3) -> bool {
        if self.planar_dist_sq(boar_pos, listener) > 1.28 * 1.28 {
            return false;
        }
        let probe_y = listener.y.clamp(boar_pos.y - 1.0, boar_pos.y + 1.25);
        let target_ground = self.world.constrain_to_surface(Vec3::new(listener.x, probe_y, listener.z), 0.0);
        let stance_height = listener.y - target_ground.y;
        if (target_ground.y - boar_pos.y).abs() > 0.65 || !(-0.65..=1.65).contains(&stance_height) {
            return false;
        }
        let mid = (boar_pos + target_ground) * 0.5;
        let flat = Vec3::new(target_ground.x - boar_pos.x, 0.0, target_ground.z - boar_pos.z).normalize_or_zero();
        let side = Vec3::new(-flat.z, 0.0, flat.x);
        [Vec3::ZERO, side * 0.45, -side * 0.45].into_iter().any(|off| {
            let g = self.world.constrain_to_surface(Vec3::new(mid.x + off.x, boar_pos.y + 1.15, mid.z + off.z), 0.0);
            (g.y - boar_pos.y).abs() <= 0.65
        })
    }

    pub(super) fn advance_toward(&self, pos: Vec3, target: Vec3, speed: f32, dt: f32, charging: bool) -> (Vec3, Vec3, bool) {
        let flat = Vec3::new(target.x - pos.x, 0.0, target.z - pos.z);
        let dist = flat.length();
        if dist <= 0.30 {
            return (pos, Vec3::X, true);
        }
        let dir = flat / dist;
        let step = (speed * dt).min(dist);
        let max_climb = if charging { 1.65 } else { 1.25 };
        for steer in [0.0_f32, 0.68, -0.68] {
            let (s, c) = steer.sin_cos();
            let try_dir = Vec3::new(dir.x * c - dir.z * s, 0.0, dir.x * s + dir.z * c);
            let raw = pos + try_dir * step;
            let next = self.world.constrain_to_surface(Vec3::new(raw.x, pos.y + 1.35, raw.z), 0.0);
            let valid_zone = if charging {
                ClimateZone::dominant(next) == ClimateZone::Temperate && self.world.surface_up_at(next).y > 0.72
            } else {
                TemperateGlade::temperate_surface(next, self.world)
            };
            if (next.y - pos.y).abs() <= max_climb && valid_zone {
                return (next, try_dir, dist - step <= 0.30);
            }
            if !charging {
                break;
            }
        }
        (pos, dir, true)
    }
}
