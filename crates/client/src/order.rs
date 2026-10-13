use glam::Vec3;
use world::PlanetaryWorld;
use crate::controls::LocomotionTarget;
use crate::flight::GravitationalFlight;
use crate::gait::{LocomotionGait, SneakGait, SprintGait, WalkGait};
use crate::lens::PerspectiveLens;
use crate::phase::{FlightContext, FlightProgress};
use crate::posture::AvatarPosture;
use crate::threshold::StepThreshold;

/// Polymorphic intention to initiate vertical leap flight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LeapIntent {
    Resting,
    Active,
}

impl LeapIntent {
    pub fn apply(&self, flight: &mut dyn GravitationalFlight) {
        match self {
            Self::Resting => {}
            Self::Active => flight.jump(),
        }
    }
}

/// Active locomotion gait mode selecting between walking, sneaking, and sprinting.
#[derive(Clone, Copy, Debug)]
pub enum ActiveGait {
    Walk(WalkGait),
    Sneak(SneakGait),
    Sprint(SprintGait),
}

impl LocomotionGait for ActiveGait {
    fn pace_distance(&self, distance: f32) -> f32 {
        match self {
            Self::Walk(g) => g.pace_distance(distance),
            Self::Sneak(g) => g.pace_distance(distance),
            Self::Sprint(g) => g.pace_distance(distance),
        }
    }

    fn shape_posture(&self, posture: AvatarPosture, dt: f32) -> AvatarPosture {
        match self {
            Self::Walk(g) => g.shape_posture(posture, dt),
            Self::Sneak(g) => g.shape_posture(posture, dt),
            Self::Sprint(g) => g.shape_posture(posture, dt),
        }
    }

    fn shape_lens(&self, lens: PerspectiveLens, dt: f32) -> PerspectiveLens {
        match self {
            Self::Walk(g) => g.shape_lens(lens, dt),
            Self::Sneak(g) => g.shape_lens(lens, dt),
            Self::Sprint(g) => g.shape_lens(lens, dt),
        }
    }

    fn resolve_precipice(
        &self,
        ctx: &FlightContext,
        threshold: StepThreshold,
        horizontal: Vec3,
    ) -> FlightProgress {
        match self {
            Self::Walk(g) => g.resolve_precipice(ctx, threshold, horizontal),
            Self::Sneak(g) => g.resolve_precipice(ctx, threshold, horizontal),
            Self::Sprint(g) => g.resolve_precipice(ctx, threshold, horizontal),
        }
    }
}

/// Encapsulated navigational order commanded by input controls for a single simulation step.
pub struct LocomotionOrder {
    direction: Vec3,
    leap: LeapIntent,
    gait: ActiveGait,
}

impl LocomotionOrder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(direction: Vec3, leap: LeapIntent, gait: ActiveGait) -> Self {
        Self {
            direction,
            leap,
            gait,
        }
    }

    /// Secondary constructor initializing with idle state and walking gait.
    pub fn idle() -> Self {
        Self::new(Vec3::ZERO, LeapIntent::Resting, ActiveGait::Walk(WalkGait::upright()))
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
        flight.advance(dt, position, stride, &self.gait, surface_up, world)
    }
}

impl LocomotionTarget for LocomotionOrder {
    fn travel(&mut self, direction: Vec3) {
        self.direction = direction;
    }

    fn jump(&mut self) {
        self.leap = LeapIntent::Active;
    }

    fn sneak(&mut self) {
        self.gait = ActiveGait::Sneak(SneakGait::crouching());
    }

    fn sprint(&mut self) {
        self.gait = ActiveGait::Sprint(SprintGait::swift());
    }
}
