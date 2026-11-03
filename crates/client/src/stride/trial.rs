use glam::Vec3;
use world::PlanetaryWorld;
use crate::envelope::{BodyEnvelope, PhysicalEnvelope};
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

/// Polymorphic trial step candidate evaluated against planetary terrain.
pub trait StrideTrial {
    fn passable(&self) -> bool;
    fn destination(&self) -> Vec3;
}

/// Encapsulated posture frame of an avatar evaluating a stride trial.
#[derive(Clone, Copy)]
struct StrideStance {
    origin: Vec3,
    up: Vec3,
    eye: f32,
    limit: StepThreshold,
}

impl StrideStance {
    const OVERHEAD_PIN_TOLERANCE: f32 = 0.04;
    const DEEP_SHAFT_CLEARANCE: f32 = 2.5;
    const PIT_HOVER_TOLERANCE: f32 = 0.02;
    const PIT_RIM_CLEARANCE: f32 = 0.52;
    const LEVEL_CONTACT_EPSILON: f32 = -1e-4;

    fn pinned_under_ceiling(&self, origin_surf: Vec3, vertical: Vec3) -> bool {
        (origin_surf - self.origin).dot(self.up) > Self::OVERHEAD_PIN_TOLERANCE
            && vertical.dot(self.up) > 0.0
    }

    fn permits_aerial_step(
        &self,
        origin_surf: Vec3,
        surface_pos: Vec3,
        vertical: Vec3,
        clearance: SurfaceClearance,
    ) -> bool {
        let altitude_above_floor = (self.origin - origin_surf).dot(self.up);
        let in_deep_shaft = altitude_above_floor >= Self::DEEP_SHAFT_CLEARANCE;
        let over_pit = altitude_above_floor > Self::PIT_HOVER_TOLERANCE && !in_deep_shaft;
        let step_ref = if in_deep_shaft { self.origin } else { origin_surf };
        let drop_limit = if over_pit {
            Self::PIT_RIM_CLEARANCE
        } else {
            Self::LEVEL_CONTACT_EPSILON
        };
        let falling_into_pit = vertical.dot(self.up) <= 0.0
            && (self.origin - surface_pos).dot(self.up) < drop_limit
            && !SurfaceStep::new(step_ref, surface_pos, self.up).walkable_with(self.limit);
        if falling_into_pit {
            false
        } else if !clearance.cliff_impact(self.limit) {
            true
        } else {
            SurfaceStep::new(step_ref, surface_pos, self.up).walkable_with(self.limit)
        }
    }
}

/// Encapsulated physical probe against planetary terrain.
#[derive(Clone, Copy)]
struct StrideProbe<'a> {
    envelope: BodyEnvelope,
    world: &'a dyn PlanetaryWorld,
}

/// Evaluated ground trial step outcome.
pub struct WalkTrial<'a> {
    step: Vec3,
    seam_allowed: bool,
    stance: StrideStance,
    probe: StrideProbe<'a>,
}

impl<'a> WalkTrial<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: Vec3,
        step: Vec3,
        up: Vec3,
        eye: f32,
        envelope: BodyEnvelope,
        limit: StepThreshold,
        seam_allowed: bool,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self {
            step,
            seam_allowed,
            stance: StrideStance { origin, up, eye, limit },
            probe: StrideProbe { envelope, world },
        }
    }
}

impl<'a> StrideTrial for WalkTrial<'a> {
    fn passable(&self) -> bool {
        if self.step.length_squared() <= 1e-6 {
            return false;
        }
        let candidate = self.stance.origin + self.step;
        let surface_pos = self.probe.envelope.constrain_with_height(candidate, self.stance.eye, self.probe.world);
        let clearance = SurfaceClearance::between(self.stance.origin, candidate, surface_pos, self.stance.up, self.probe.world);
        clearance.passable_with(self.seam_allowed)
            && SurfaceStep::new(self.stance.origin, surface_pos, self.stance.up).walkable_with(self.stance.limit)
            && self.probe.envelope.unobstructed_ground(
                candidate,
                surface_pos,
                self.step,
                self.stance.up,
                self.stance.eye,
                self.stance.limit,
                self.probe.world,
            )
    }

    fn destination(&self) -> Vec3 {
        if self.step.length_squared() <= 1e-6 {
            return self.stance.origin;
        }
        self.probe.envelope.constrain_with_height(self.stance.origin + self.step, self.stance.eye, self.probe.world)
    }
}

/// Evaluated mid-air trial step outcome.
pub struct AerialTrial<'a> {
    tangent: Vec3,
    vertical: Vec3,
    stance: StrideStance,
    probe: StrideProbe<'a>,
}

impl<'a> AerialTrial<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: Vec3,
        tangent: Vec3,
        vertical: Vec3,
        up: Vec3,
        eye: f32,
        envelope: BodyEnvelope,
        limit: StepThreshold,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self {
            tangent,
            vertical,
            stance: StrideStance { origin, up, eye, limit },
            probe: StrideProbe { envelope, world },
        }
    }
}

impl<'a> StrideTrial for AerialTrial<'a> {
    fn passable(&self) -> bool {
        if self.tangent.length_squared() <= 1e-6 {
            return false;
        }
        let candidate = self.stance.origin + self.tangent + self.vertical;
        let surface_pos = self.probe.envelope.constrain_with_height(candidate, self.stance.eye, self.probe.world);
        let clearance = SurfaceClearance::between(self.stance.origin, candidate, surface_pos, self.stance.up, self.probe.world);
        let origin_surf = self.probe.envelope.constrain_with_height(self.stance.origin, self.stance.eye, self.probe.world);
        if self.stance.pinned_under_ceiling(origin_surf, self.vertical) {
            return false;
        }
        self.stance.permits_aerial_step(origin_surf, surface_pos, self.vertical, clearance)
            && self.probe.envelope.unobstructed_aerial(
                self.stance.origin,
                candidate,
                self.tangent,
                self.stance.up,
                self.stance.eye,
                self.stance.limit,
                self.probe.world,
            )
    }

    fn destination(&self) -> Vec3 {
        self.stance.origin + self.tangent + self.vertical
    }
}

/// Interface for cascaded stride resolution across planetary terrain.
pub trait StrideResolution {
    fn resolve(&self) -> Vec3;
}

/// Fallback terminal resolution yielding resting position without displacement.
pub struct RestingStride {
    position: Vec3,
}

impl RestingStride {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}

impl StrideResolution for RestingStride {
    fn resolve(&self) -> Vec3 {
        self.position
    }
}

/// Decorator attempting a trial step and delegating to fallback resolution when obstructed.
pub struct DeflectedStride<'a> {
    trial: &'a dyn StrideTrial,
    fallback: &'a dyn StrideResolution,
}

impl<'a> DeflectedStride<'a> {
    pub fn new(trial: &'a dyn StrideTrial, fallback: &'a dyn StrideResolution) -> Self {
        Self { trial, fallback }
    }
}

impl<'a> StrideResolution for DeflectedStride<'a> {
    fn resolve(&self) -> Vec3 {
        if self.trial.passable() {
            self.trial.destination()
        } else {
            self.fallback.resolve()
        }
    }
}
