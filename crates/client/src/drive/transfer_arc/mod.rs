mod brachistochrone;
mod telemetry;
mod verlet;

use glam::Vec3;
use world::{BodyId, CelestialAtlas, PlanetaryWorld};
use super::space_step::MAX_COSMIC_SPEED;
pub use self::brachistochrone::BrachistochroneArc;
pub use self::telemetry::{CourseMode, CourseProfile, FlipBurnReadout, RouteTelemetry};
pub use self::verlet::SymplecticOrbit;

pub const ARC_NODES: usize = 33;

/// Gravity-integrated multi-body ballistic or Brachistochrone transfer trajectory (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct TransferArc {
    nodes: [Vec3; ARC_NODES],
    target_normal: Vec3,
    corridor_radius: f32,
    profile: CourseProfile,
}

impl TransferArc {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_profile(
        nodes: [Vec3; ARC_NODES],
        target_normal: Vec3,
        corridor_radius: f32,
        profile: CourseProfile,
    ) -> Self {
        Self { nodes, target_normal, corridor_radius, profile }
    }

    /// Secondary constructor for ballistic trajectories without explicit arrival timing.
    pub const fn new(nodes: [Vec3; ARC_NODES], target_normal: Vec3, corridor_radius: f32) -> Self {
        Self::with_profile(nodes, target_normal, corridor_radius, CourseProfile::Ballistic { arrival_secs: 0.0 })
    }

    fn clamp_cosmic(v: Vec3) -> Vec3 {
        if v.length_squared() > MAX_COSMIC_SPEED * MAX_COSMIC_SPEED {
            v.normalize_or_zero() * MAX_COSMIC_SPEED
        } else {
            v
        }
    }

    fn simulate_local(origin: Vec3, launch_vel: Vec3, duration: f32, world: &dyn PlanetaryWorld) -> [Vec3; ARC_NODES] {
        let dt = (duration / ((ARC_NODES - 1) as f32)).max(1e-3);
        let mut out = [origin; ARC_NODES];
        let (mut p, mut v) = (origin, Self::clamp_cosmic(launch_vel));
        for slot in out.iter_mut().skip(1) {
            let dv = world.apply_pull(p, dt);
            let next_v = Self::clamp_cosmic(v + dv);
            p += (v + next_v) * (0.5 * dt);
            v = next_v;
            *slot = p;
        }
        out
    }

    /// Integrates `PlanetaryWorld` gravity via symplectic Velocity-Verlet shooting to connect `origin` to `target`.
    pub fn charted(
        origin: Vec3,
        velocity: Vec3,
        target: Vec3,
        target_normal: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Self {
        let chord = target - origin;
        let dist = chord.length();
        if let Some(eph) = world.celestial_atlas().ephemeris() {
            if dist > eph.proportions().earth_half() * 4.5 {
                return SymplecticOrbit::new(eph, 6).point_intercept(
                    origin.as_dvec3(),
                    velocity.as_dvec3(),
                    target.as_dvec3(),
                    target_normal,
                );
            }
        }
        let corridor = (dist * 0.055).clamp(2.5, 16.0);
        if dist < 0.5 {
            let mut linear = [origin; ARC_NODES];
            linear[ARC_NODES - 1] = target;
            return Self::new(linear, target_normal.normalize_or_zero(), corridor);
        }
        let up_0 = world.surface_up_at(origin).normalize_or_zero();
        let g0 = world.apply_pull(origin, 1.0).length();
        let mut duration = (dist / 31.0).clamp(1.2, 26.0);
        let loft = up_0 * (g0 * 0.65) + target_normal.normalize_or_zero() * 1.8;
        let seed_vel = if velocity.length_squared() > 4.0 && velocity.dot(chord) > 0.0 {
            (chord / duration) * 0.78 + velocity * 0.22 + loft * 0.5
        } else {
            chord / duration + loft
        };
        let mut launch_vel = Self::clamp_cosmic(seed_vel);
        let mut nodes = Self::simulate_local(origin, launch_vel, duration, world);
        for _ in 0..8 {
            let miss = target - nodes[ARC_NODES - 1];
            if miss.length_squared() <= 0.04 {
                break;
            }
            let raw_next = launch_vel + (miss / duration) * 0.85;
            if raw_next.length() > MAX_COSMIC_SPEED * 0.97 && duration < 42.0 {
                duration *= 1.18;
            }
            launch_vel = Self::clamp_cosmic(raw_next);
            nodes = Self::simulate_local(origin, launch_vel, duration, world);
        }
        let residual = target - nodes[ARC_NODES - 1];
        let denom = (ARC_NODES - 1) as f32;
        for (idx, pt) in nodes.iter_mut().enumerate() {
            let s = (idx as f32) / denom;
            *pt += residual * (s * s * (3.0 - 2.0 * s));
        }
        nodes[0] = origin;
        nodes[ARC_NODES - 1] = target;
        Self::with_profile(nodes, target_normal.normalize_or_zero(), corridor, CourseProfile::Ballistic { arrival_secs: duration })
    }

    /// Computes a lead-compensated multi-body Keplerian or Brachistochrone intercept to `body`.
    pub fn intercept_body(
        origin: Vec3,
        velocity: Vec3,
        body: BodyId,
        mode: CourseMode,
        burn_accel: f32,
        atlas: CelestialAtlas,
    ) -> Option<Self> {
        let eph = atlas.ephemeris()?;
        match mode {
            CourseMode::Ballistic => SymplecticOrbit::new(eph, 8).intercept_body(origin.as_dvec3(), velocity.as_dvec3(), body, atlas),
            CourseMode::Brachistochrone => {
                BrachistochroneArc::new(f64::from(burn_accel.max(8.0))).intercept_body(origin.as_dvec3(), velocity.as_dvec3(), body, atlas)
            }
        }
    }

    /// Computes the live unpowered curved Keplerian / slingshot trajectory over `horizon_secs`.
    pub fn coasting_preview(origin: Vec3, velocity: Vec3, horizon_secs: f64, atlas: CelestialAtlas) -> [Vec3; ARC_NODES] {
        if let Some(eph) = atlas.ephemeris() {
            SymplecticOrbit::new(eph, 8).coasting_arc(origin.as_dvec3(), velocity.as_dvec3(), horizon_secs)
        } else {
            std::array::from_fn(|i| origin + velocity * ((horizon_secs as f32) * (i as f32) / ((ARC_NODES - 1) as f32)))
        }
    }

    pub const fn flip_marker(&self) -> Option<Vec3> {
        match self.profile {
            CourseProfile::Ballistic { .. } => None,
            CourseProfile::Brachistochrone { flip_point, .. } => Some(flip_point),
        }
    }

    pub fn sample_flip_burn(&self, ship_pos: Vec3, velocity: Vec3, nose_axis: Vec3) -> FlipBurnReadout {
        let CourseProfile::Brachistochrone { flip_point, burn_accel, .. } = self.profile else {
            return FlipBurnReadout::Inactive;
        };
        let (origin, dest) = (self.nodes[0], self.nodes[ARC_NODES - 1]);
        let chord = dest - origin;
        let span = chord.length().max(1.0);
        let axis = chord / span;
        let progress = ((ship_pos - origin).dot(axis) / span).clamp(0.0, 1.0);
        let to_dest = dest - ship_pos;
        let rem_dist = to_dest.length();
        if rem_dist <= 8.0 {
            return FlipBurnReadout::Inactive;
        }
        let close_dir = to_dest / rem_dist;
        let v_close = velocity.dot(close_dir).max(0.0);
        let a = burn_accel.max(5.0);
        let stop_dist = (v_close * v_close) / (2.0 * a);
        if progress < 0.48 && stop_dist < rem_dist * 0.92 {
            let d_flip = (flip_point - ship_pos).dot(axis).max(0.0);
            let t_flip = ((v_close * v_close + 2.0 * a * d_flip).sqrt() - v_close) / a;
            FlipBurnReadout::TimeToFlip(t_flip.max(0.1))
        } else if velocity.length_squared() > 16.0 && nose_axis.normalize_or_zero().dot(velocity.normalize_or_zero()) > -0.55 {
            FlipBurnReadout::FlipNow
        } else {
            let eta = (v_close / a).max((2.0 * rem_dist / a).sqrt() * 0.35);
            FlipBurnReadout::BrakeBurn(eta)
        }
    }

    pub fn retargets(&self, candidate: Vec3) -> bool {
        (candidate - self.nodes[ARC_NODES - 1]).length_squared() > 2.25
    }

    pub fn feed_nodes(&self, f: impl FnOnce(&[Vec3; ARC_NODES], Vec3)) {
        f(&self.nodes, self.target_normal);
    }

    pub fn sample_range<R>(&self, ship_pos: Vec3, velocity: Vec3, f: impl FnOnce(Vec3, f32, f32) -> R) -> R {
        let target = self.nodes[ARC_NODES - 1];
        let away = ship_pos - target;
        let dist = away.length();
        let rate = if dist > 1e-4 { velocity.dot(away / dist) } else { 0.0 };
        f(target, dist, rate)
    }

    pub fn sample_telemetry(&self, ship_pos: Vec3, flight_dir: Vec3) -> RouteTelemetry {
        let mut best_anchor = self.nodes[0];
        let mut best_tangent = (self.nodes[1] - self.nodes[0]).normalize_or_zero();
        let mut best_dist_sq = f32::MAX;
        for i in 0..(ARC_NODES - 1) {
            let (a, b) = (self.nodes[i], self.nodes[i + 1]);
            let ab = b - a;
            let len_sq = ab.length_squared();
            let t = if len_sq > 1e-6 { ((ship_pos - a).dot(ab) / len_sq).clamp(0.0, 1.0) } else { 0.0 };
            let closest = a + ab * t;
            let d_sq = (ship_pos - closest).length_squared();
            if d_sq < best_dist_sq {
                best_dist_sq = d_sq;
                best_anchor = closest;
                if len_sq > 1e-6 {
                    best_tangent = ab / len_sq.sqrt();
                }
            }
        }
        let cross_track = best_dist_sq.sqrt();
        let f_dir = flight_dir.normalize_or_zero();
        let angle_deg = if f_dir.length_squared() > 0.25 && best_tangent.length_squared() > 0.25 {
            f_dir.dot(best_tangent).clamp(-1.0, 1.0).acos().to_degrees()
        } else {
            0.0
        };
        let near_destination = (ship_pos - self.nodes[ARC_NODES - 1]).length_squared() <= 9.0;
        let aligned = near_destination || (cross_track <= self.corridor_radius && angle_deg <= 22.0);
        RouteTelemetry::new(best_anchor, cross_track, angle_deg, aligned)
    }
}

/// Polymorphic spacecraft navigation state avoiding `Option<TransferArc>`.
#[derive(Clone, Copy, Debug)]
pub enum NavigationCourse {
    Uncharted,
    Charted(TransferArc),
}

impl NavigationCourse {
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
