use glam::{Quat, Vec3};
use world::{PilotOrder, PlanetaryWorld, VehicleAssembly};
use super::clearance::HullClearance;

pub(crate) const MAX_COSMIC_SPEED: f32 = 45.0;
pub(crate) const MAX_FUSION_SPEED: f32 = 240_000.0;

/// Immutable 6-DOF spacecraft state step returned by the `SpaceStep` decorator chain.
#[derive(Clone, Copy, Debug)]
pub(super) struct SpaceState {
    origin: Vec3,
    velocity: Vec3,
    angular_velocity: Vec3,
    rotation: Quat,
}

impl SpaceState {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        origin: Vec3,
        velocity: Vec3,
        angular_velocity: Vec3,
        rotation: Quat,
    ) -> Self {
        Self { origin, velocity, angular_velocity, rotation }
    }

    pub(super) fn bounded_at(
        origin: Vec3,
        raw_velocity: Vec3,
        angular_velocity: Vec3,
        rotation: Quat,
        speed_cap: f32,
    ) -> Self {
        let velocity = if raw_velocity.length_squared() > speed_cap * speed_cap {
            raw_velocity.normalize() * speed_cap
        } else {
            raw_velocity
        };
        Self::new(origin, velocity, angular_velocity, rotation)
    }

    pub(super) fn transfer<R>(self, f: impl FnOnce(Vec3, Vec3, Vec3, Quat) -> R) -> R {
        f(self.origin, self.velocity, self.angular_velocity, self.rotation)
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
    angular_velocity: Vec3,
}

impl InertialDrift {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: Vec3, velocity: Vec3, angular_velocity: Vec3) -> Self {
        Self { origin, velocity, angular_velocity }
    }
}

impl SpaceStep for InertialDrift {
    fn resolve(&self, dt: f32) -> SpaceState {
        SpaceState::new(
            self.origin + self.velocity * dt,
            self.velocity,
            self.angular_velocity,
            Quat::IDENTITY,
        )
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
        SpaceState::bounded_at(s.origin + dv * (0.5 * dt), s.velocity + dv, s.angular_velocity, s.rotation, MAX_FUSION_SPEED)
    }
}

/// `SpaceStep` decorator firing directional `RocketNozzle` and `RcsManifold` reaction thrust (`<= 4` fields).
pub(super) struct NozzleThrustStep<'a, S> {
    origin: S,
    assembly: &'a VehicleAssembly,
    order: PilotOrder,
    world: &'a dyn PlanetaryWorld,
}

impl<'a, S> NozzleThrustStep<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        origin: S,
        assembly: &'a VehicleAssembly,
        order: PilotOrder,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self { origin, assembly, order, world }
    }
}

impl<S: SpaceStep> SpaceStep for NozzleThrustStep<'_, S> {
    fn resolve(&self, dt: f32) -> SpaceState {
        let s = self.origin.resolve(dt);
        let impulse = self.assembly.sample_impulse(&self.order);
        let fusion = self.assembly.fusion_equipped();
        let chem_cap = self
            .world
            .celestial_atlas()
            .ephemeris()
            .map_or(MAX_COSMIC_SPEED, |e| e.proportions().chemical_speed_cap());
        let cap = if fusion { MAX_FUSION_SPEED } else { chem_cap };
        let raw_dv = if fusion {
            impulse.keel_fusion_velocity_delta(self.assembly.aerospace_framed(), dt)
        } else {
            impulse.velocity_delta(dt)
        };
        let speed = s.velocity.length();
        let dv = if speed > 1e-4 {
            let vel_dir = s.velocity / speed;
            let fwd_proj = raw_dv.dot(vel_dir);
            if fwd_proj > 0.0 {
                let ratio = (speed / cap).clamp(0.0, 1.0);
                let r2 = ratio * ratio;
                let taper = (1.0 - r2 * r2).max(0.0);
                (raw_dv - vel_dir * fwd_proj) + vel_dir * (fwd_proj * taper)
            } else {
                raw_dv
            }
        } else {
            raw_dv
        };
        SpaceState::bounded_at(s.origin + dv * (0.5 * dt), s.velocity + dv, s.angular_velocity, s.rotation, cap)
    }
}

/// `SpaceStep` decorator integrating physical `r × F` lever moments and `FlightConsole` guidance.
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
        let (next_omega, step_rot) = impulse.integrate_attitude(
            s.angular_velocity,
            self.order.guidance(),
            dt,
        );
        SpaceState::new(s.origin, s.velocity, next_omega, (step_rot * s.rotation).normalize())
    }
}

/// `SpaceStep` decorator arresting descent on planetary or lunar terrain and settling landing skids upright.
pub(super) struct TouchdownStep<'a, S> {
    origin: S,
    assembly: &'a VehicleAssembly,
    world: &'a dyn PlanetaryWorld,
}

impl<'a, S> TouchdownStep<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: S, assembly: &'a VehicleAssembly, world: &'a dyn PlanetaryWorld) -> Self {
        Self { origin, assembly, world }
    }
}

impl<S: SpaceStep> SpaceStep for TouchdownStep<'_, S> {
    fn resolve(&self, dt: f32) -> SpaceState {
        let s = self.origin.resolve(dt);
        if self.world.apply_pull(s.origin, 1.0).length_squared() <= 0.04 {
            return s;
        }
        let probe = HullClearance::new(self.assembly, s.origin, s.velocity, s.rotation);
        let Some((ground, surf_up, alt, v_norm)) = probe.crust_contact(self.world) else {
            return s;
        };
        let rotated_up = (s.rotation * self.assembly.normal()).normalize_or_zero();
        let clearance = probe.span_projection(surf_up, rotated_up);
        if alt > 0.005 || (alt >= 0.0 && v_norm > 0.0) {
            return s;
        }
        if v_norm > 0.0 {
            return SpaceState::new(
                ground + surf_up * (clearance + 0.006),
                s.velocity,
                s.angular_velocity,
                s.rotation,
            );
        }
        let next_v = (s.velocity - surf_up * v_norm.min(0.0)) * (1.0 - (5.5 * dt).clamp(0.0, 1.0));
        let pad_omega = surf_up * s.angular_velocity.dot(surf_up) * (1.0 - (0.6 * dt).clamp(0.0, 1.0));
        let dot = rotated_up.dot(surf_up).clamp(-1.0, 1.0);
        let max_turn = (7.5 * dt).clamp(0.0, 0.35);
        let settle_arc = if dot >= 0.9999 {
            Quat::IDENTITY
        } else if dot < -0.995 {
            let pivot = if surf_up.x.abs() < 0.9 { Vec3::X } else { Vec3::Z };
            Quat::from_axis_angle(rotated_up.cross(pivot).normalize_or_zero(), max_turn)
        } else {
            let full = Quat::from_rotation_arc(rotated_up, surf_up);
            Quat::IDENTITY.slerp(full, (max_turn / dot.acos().max(1e-4)).clamp(0.0, 1.0))
        };
        let next_rot = (settle_arc * s.rotation).normalize();
        let next_up = (next_rot * self.assembly.normal()).normalize_or_zero();
        let next_clear = probe.span_projection(surf_up, next_up);
        SpaceState::new(ground + surf_up * next_clear, next_v, pad_omega, next_rot)
    }
}
