use glam::{Quat, Vec3};
use world::{PilotOrder, PlanetaryWorld, VehicleAssembly};
use super::track::FaceContact;

const MAX_COSMIC_SPEED: f32 = 52.0;

/// Immutable 6-DOF spacecraft state step returned by the `SpaceStep` decorator chain.
#[derive(Clone, Copy, Debug)]
pub(super) struct SpaceState {
    origin: Vec3,
    velocity: Vec3,
    rotation: Quat,
}

impl SpaceState {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: Vec3, velocity: Vec3, rotation: Quat) -> Self {
        Self { origin, velocity, rotation }
    }

    pub(super) fn transfer<R>(self, f: impl FnOnce(Vec3, Vec3, Quat) -> R) -> R {
        f(self.origin, self.velocity, self.rotation)
    }
}

/// Narrow 1-method contract for stack-allocated spacecraft trajectory decorators.
pub(super) trait SpaceStep {
    fn resolve(&self, dt: f32) -> SpaceState;
}

/// Base `SpaceStep` executing Newton's first law inertial drift in vacuum without drag.
pub(super) struct InertialDrift {
    origin: Vec3,
    velocity: Vec3,
}

impl InertialDrift {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: Vec3, velocity: Vec3) -> Self {
        Self { origin, velocity }
    }
}

impl SpaceStep for InertialDrift {
    fn resolve(&self, dt: f32) -> SpaceState {
        SpaceState::new(self.origin + self.velocity * dt, self.velocity, Quat::IDENTITY)
    }
}

/// `SpaceStep` decorator applying the continuous planetary/lunar `GravityField::apply_pull` impulse.
pub(super) struct GravitationStep<'a, S> {
    origin: S,
    world: &'a dyn PlanetaryWorld,
}

impl<'a, S> GravitationStep<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: S, world: &'a dyn PlanetaryWorld) -> Self {
        Self { origin, world }
    }
}

impl<S: SpaceStep> SpaceStep for GravitationStep<'_, S> {
    fn resolve(&self, dt: f32) -> SpaceState {
        let s = self.origin.resolve(dt);
        let dv = self.world.apply_pull(s.origin, dt);
        SpaceState::new(s.origin + dv * (0.5 * dt), s.velocity + dv, s.rotation)
    }
}

/// `SpaceStep` decorator firing directional `RocketNozzle` reaction thrust across all ship parts.
pub(super) struct NozzleThrustStep<'a, S> {
    origin: S,
    assembly: &'a VehicleAssembly,
    order: PilotOrder,
}

impl<'a, S> NozzleThrustStep<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: S, assembly: &'a VehicleAssembly, order: PilotOrder) -> Self {
        Self { origin, assembly, order }
    }
}

impl<S: SpaceStep> SpaceStep for NozzleThrustStep<'_, S> {
    fn resolve(&self, dt: f32) -> SpaceState {
        let s = self.origin.resolve(dt);
        let impulse = self.assembly.sample_impulse(&self.order);
        let dv = impulse.velocity_delta(dt);
        let raw_v = s.velocity + dv;
        let clamped_v = if raw_v.length_squared() > MAX_COSMIC_SPEED * MAX_COSMIC_SPEED {
            raw_v.normalize() * MAX_COSMIC_SPEED
        } else {
            raw_v
        };
        SpaceState::new(s.origin + dv * (0.5 * dt), clamped_v, s.rotation)
    }
}

/// `SpaceStep` decorator resolving `FlightConsole` gyro stabilization against off-center nozzle torque.
pub(super) struct GyroStep<'a, S> {
    origin: S,
    assembly: &'a VehicleAssembly,
    order: PilotOrder,
}

impl<'a, S> GyroStep<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: S, assembly: &'a VehicleAssembly, order: PilotOrder) -> Self {
        Self { origin, assembly, order }
    }
}

impl<S: SpaceStep> SpaceStep for GyroStep<'_, S> {
    fn resolve(&self, dt: f32) -> SpaceState {
        let s = self.origin.resolve(dt);
        let impulse = self.assembly.sample_impulse(&self.order);
        let up = self.assembly.normal().normalize_or_zero();
        let raw_fwd = self.assembly.heading();
        let fwd = (raw_fwd - up * raw_fwd.dot(up)).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        let step_rot = impulse.attitude_step(&self.order, right, up, fwd, dt);
        SpaceState::new(s.origin, s.velocity, (step_rot * s.rotation).normalize())
    }
}

/// `SpaceStep` decorator arresting descent on planetary or lunar terrain and settling landing skids upright.
pub(super) struct TouchdownStep<'a, S> {
    origin: S,
    chassis_up: Vec3,
    world: &'a dyn PlanetaryWorld,
}

impl<'a, S> TouchdownStep<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: S, chassis_up: Vec3, world: &'a dyn PlanetaryWorld) -> Self {
        Self { origin, chassis_up, world }
    }
}

impl<S: SpaceStep> SpaceStep for TouchdownStep<'_, S> {
    fn resolve(&self, dt: f32) -> SpaceState {
        let s = self.origin.resolve(dt);
        if self.world.apply_pull(s.origin, 1.0).length_squared() <= 0.04 {
            return s;
        }
        let surf_up = self.world.surface_up_at(s.origin);
        let ground = FaceContact::new(s.origin).ground(0.0, self.world);
        let alt = (s.origin - ground).dot(surf_up);
        let v_norm = s.velocity.dot(surf_up);
        if alt > 0.005 || (alt >= 0.0 && v_norm > 0.0) {
            return s;
        }
        if v_norm > 0.0 {
            return SpaceState::new(ground + surf_up * 0.006, s.velocity, s.rotation);
        }
        let next_v = (s.velocity - surf_up * v_norm.min(0.0)) * (1.0 - (5.5 * dt).clamp(0.0, 1.0));
        let rotated_up = (s.rotation * self.chassis_up).normalize_or_zero();
        let settle_arc = if rotated_up.dot(surf_up) > 0.25 && rotated_up.dot(surf_up) < 0.9999 {
            Quat::IDENTITY.slerp(Quat::from_rotation_arc(rotated_up, surf_up), (8.0 * dt).clamp(0.0, 1.0))
        } else {
            Quat::IDENTITY
        };
        SpaceState::new(ground, next_v, (settle_arc * s.rotation).normalize())
    }
}
