use glam::Vec3;
use world::PlanetaryWorld;
use crate::controls::LocomotionTarget;
use crate::flight::GravitationalFlight;
use crate::gait::{LocomotionGait, SneakGait, SprintGait, WalkGait};
use crate::lens::PerspectiveLens;
use crate::posture::AvatarPosture;

/// Polymorphic intention to initiate vertical leap flight.
pub trait LeapIntent: Send + Sync {
    fn apply(&self, flight: &mut dyn GravitationalFlight);
}

/// Inactive leap state maintaining current flight phase.
pub struct RestingLeap;

impl LeapIntent for RestingLeap {
    fn apply(&self, _flight: &mut dyn GravitationalFlight) {}
}

/// Active leap command triggering instantaneous upward leap impulse.
pub struct ActiveLeap;

impl LeapIntent for ActiveLeap {
    fn apply(&self, flight: &mut dyn GravitationalFlight) {
        flight.jump();
    }
}

/// Encapsulated navigational order commanded by input controls for a single simulation step.
pub struct LocomotionOrder {
    direction: Vec3,
    leap: Box<dyn LeapIntent>,
    gait: Box<dyn LocomotionGait>,
}

impl LocomotionOrder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(direction: Vec3, leap: Box<dyn LeapIntent>, gait: Box<dyn LocomotionGait>) -> Self {
        Self {
            direction,
            leap,
            gait,
        }
    }

    /// Secondary constructor initializing with idle state and walking gait.
    pub fn idle() -> Self {
        Self::new(Vec3::ZERO, Box::new(RestingLeap), Box::new(WalkGait::upright()))
    }

    /// Computes the horizontal stride aligned with a surface normal for a given distance scaled by gait pace.
    pub fn surface_stride(&self, surface_up: Vec3, distance: f32) -> Vec3 {
        let tangent = self.direction - surface_up * self.direction.dot(surface_up);
        if tangent.length_squared() > 1e-4 {
            let effective = self.gait.pace_distance(distance);
            tangent.normalize() * effective
        } else {
            Vec3::ZERO
        }
    }

    /// Dispatches leap command to flight entity.
    pub fn dispatch_jump(&self, flight: &mut dyn GravitationalFlight) {
        self.leap.apply(flight);
    }

    /// Advances avatar posture towards this order's active gait across elapsed time.
    pub fn advance_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        self.gait.shape_posture(posture, dt)
    }

    /// Advances projection lens towards this order's active gait across elapsed time.
    pub fn advance_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        self.gait.shape_lens(lens, dt)
    }

    /// Advances gravitational flight using commanded stride and active gait.
    pub fn advance_flight(
        &self,
        flight: &mut dyn GravitationalFlight,
        dt: f32,
        position: Vec3,
        surface_up: Vec3,
        base_speed: f32,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        let stride = self.surface_stride(surface_up, base_speed * dt);
        flight.advance(dt, position, stride, self.gait.as_ref(), surface_up, world)
    }
}

impl LocomotionTarget for LocomotionOrder {
    fn travel(&mut self, direction: Vec3) {
        self.direction = direction;
    }

    fn jump(&mut self) {
        self.leap = Box::new(ActiveLeap);
    }

    fn sneak(&mut self) {
        self.gait = Box::new(SneakGait::crouching());
    }

    fn sprint(&mut self) {
        self.gait = Box::new(SprintGait::swift());
    }
}
