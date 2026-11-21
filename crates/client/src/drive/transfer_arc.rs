use glam::Vec3;
use world::{CelestialAtlas, PlanetaryWorld};
use super::space_step::MAX_COSMIC_SPEED;

pub const ARC_NODES: usize = 33;

/// Live course-alignment telemetry sampled from a spacecraft against a `TransferArc`.
#[derive(Clone, Copy, Debug)]
pub struct RouteTelemetry {
    anchor: Vec3,
    cross_track: f32,
    angle_deg: f32,
    aligned: bool,
}

impl RouteTelemetry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(anchor: Vec3, cross_track: f32, angle_deg: f32, aligned: bool) -> Self {
        Self { anchor, cross_track, angle_deg, aligned }
    }

    /// Transfers `(anchor_world, cross_track_meters, angle_deg, aligned)` without getters.
    pub fn transfer<R>(self, f: impl FnOnce(Vec3, f32, f32, bool) -> R) -> R {
        f(self.anchor, self.cross_track, self.angle_deg, self.aligned)
    }
}

/// Gravity-integrated two-body ballistic transfer trajectory from departure point to celestial target.
#[derive(Clone, Copy, Debug)]
pub struct TransferArc {
    nodes: [Vec3; ARC_NODES],
    target_normal: Vec3,
    corridor_radius: f32,
}

impl TransferArc {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(nodes: [Vec3; ARC_NODES], target_normal: Vec3, corridor_radius: f32) -> Self {
        Self { nodes, target_normal, corridor_radius }
    }

    fn clamp_cosmic(v: Vec3) -> Vec3 {
        if v.length_squared() > MAX_COSMIC_SPEED * MAX_COSMIC_SPEED {
            v.normalize_or_zero() * MAX_COSMIC_SPEED
        } else {
            v
        }
    }

    fn simulate_arc(
        origin: Vec3,
        launch_vel: Vec3,
        duration: f32,
        world: &dyn PlanetaryWorld,
    ) -> [Vec3; ARC_NODES] {
        let steps = (ARC_NODES - 1) as f32;
        let dt = (duration / steps).max(1e-3);
        let mut out = [origin; ARC_NODES];
        let mut p = origin;
        let mut v = Self::clamp_cosmic(launch_vel);
        for slot in out.iter_mut().skip(1) {
            let dv = world.apply_pull(p, dt);
            let next_v = Self::clamp_cosmic(v + dv);
            p += (v + next_v) * (0.5 * dt);
            v = next_v;
            *slot = p;
        }
        out
    }

    /// Integrates `PlanetaryWorld::apply_pull` (`BinaryGravity`) via symplectic shooting to connect `origin` to `target`.
    pub fn charted(
        origin: Vec3,
        velocity: Vec3,
        target: Vec3,
        target_normal: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Self {
        let chord = target - origin;
        let dist = chord.length();
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
        let mut nodes = Self::simulate_arc(origin, launch_vel, duration, world);
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
            nodes = Self::simulate_arc(origin, launch_vel, duration, world);
        }
        let residual = target - nodes[ARC_NODES - 1];
        let denom = (ARC_NODES - 1) as f32;
        for (idx, pt) in nodes.iter_mut().enumerate() {
            let s = (idx as f32) / denom;
            let blend = s * s * (3.0 - 2.0 * s);
            *pt += residual * blend;
        }
        nodes[0] = origin;
        nodes[ARC_NODES - 1] = target;
        Self::new(nodes, target_normal.normalize_or_zero(), corridor)
    }

    /// True when `candidate` marks a distinct surface location (`> 1.5m` from the active target).
    pub fn retargets(&self, candidate: Vec3) -> bool {
        (candidate - self.nodes[ARC_NODES - 1]).length_squared() > 2.25
    }

    /// Passes `(&nodes, target_normal)` to a rendering collaborator without exposing internal fields.
    pub fn feed_nodes(&self, f: impl FnOnce(&[Vec3; ARC_NODES], Vec3)) {
        f(&self.nodes, self.target_normal);
    }

    /// Projects `ship_pos` and `flight_dir` onto the trajectory polyline to evaluate course alignment.
    pub fn sample_telemetry(&self, ship_pos: Vec3, flight_dir: Vec3) -> RouteTelemetry {
        let mut best_anchor = self.nodes[0];
        let mut best_tangent = (self.nodes[1] - self.nodes[0]).normalize_or_zero();
        let mut best_dist_sq = f32::MAX;
        for i in 0..(ARC_NODES - 1) {
            let a = self.nodes[i];
            let b = self.nodes[i + 1];
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
    /// Marks a celestial surface target along `boresight` (or `gimbal` fallback) or clears an existing lock.
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

/// Immutable telemetry packet for the Nose-Camera 3D Steuerkonsolen-Orrery (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct OrrerySnapshot {
    atlas: CelestialAtlas,
    ship_pos: Vec3,
    ship_axes: (Vec3, Vec3),
    course: NavigationCourse,
}

impl OrrerySnapshot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        atlas: CelestialAtlas,
        ship_pos: Vec3,
        ship_axes: (Vec3, Vec3),
        course: NavigationCourse,
    ) -> Self {
        Self { atlas, ship_pos, ship_axes, course }
    }

    /// Transfers `(atlas, ship_pos, (nose_axis, flight_vector), course)` to a HUD view without getters.
    pub fn transfer<R>(
        self,
        f: impl FnOnce(CelestialAtlas, Vec3, (Vec3, Vec3), NavigationCourse) -> R,
    ) -> R {
        f(self.atlas, self.ship_pos, self.ship_axes, self.course)
    }
}

/// Encapsulated spacecraft celestial atlas and plotted `NavigationCourse` (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct HelmChart {
    atlas: CelestialAtlas,
    course: NavigationCourse,
}

impl HelmChart {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(atlas: CelestialAtlas, course: NavigationCourse) -> Self {
        Self { atlas, course }
    }

    pub const fn uncharted(atlas: CelestialAtlas) -> Self {
        Self::new(atlas, NavigationCourse::Uncharted)
    }

    pub fn sync_world(&mut self, world: &dyn PlanetaryWorld) {
        self.atlas = world.celestial_atlas();
    }

    pub fn mark_target(
        &mut self,
        rays: ((Vec3, Vec3), (Vec3, Vec3)),
        ship_state: (Vec3, Vec3),
        world: &dyn PlanetaryWorld,
    ) -> bool {
        self.atlas = world.celestial_atlas();
        self.course.mark_along(rays, ship_state, world)
    }

    pub fn snapshot(&self, ship_pos: Vec3, ship_nose: Vec3, velocity: Vec3) -> OrrerySnapshot {
        let nose = ship_nose.normalize_or_zero();
        let flight = if velocity.length_squared() > 2.25 { velocity.normalize_or_zero() } else { nose };
        OrrerySnapshot::new(self.atlas, ship_pos, (nose, flight), self.course)
    }
}

