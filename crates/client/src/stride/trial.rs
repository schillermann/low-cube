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
        let over_pit = (self.stance.origin - origin_surf).dot(self.stance.up) > 0.02;
        let falling_into_pit = self.vertical.dot(self.stance.up) <= 0.0
            && (self.stance.origin - surface_pos).dot(self.stance.up) < if over_pit { 0.52 } else { -1e-4 }
            && !SurfaceStep::new(origin_surf, surface_pos, self.stance.up).walkable_with(self.stance.limit);
        let center_ok = if falling_into_pit {
            false
        } else if !clearance.cliff_impact(self.stance.limit) {
            true
        } else {
            SurfaceStep::new(origin_surf, surface_pos, self.stance.up).walkable_with(self.stance.limit)
        };
        center_ok
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
