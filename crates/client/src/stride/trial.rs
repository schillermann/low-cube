use glam::Vec3;
use world::PlanetaryWorld;
use crate::envelope::{BodyEnvelope, PhysicalEnvelope};
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

/// Polymorphic trial step candidate evaluated against planetary terrain.
pub trait StrideTrial {
    fn passable(&self) -> bool;
    fn destination(&self) -> Vec3;
}

/// Evaluated ground trial step outcome.
pub struct WalkTrial<'a> {
    origin: Vec3,
    step: Vec3,
    up: Vec3,
    eye: f32,
    envelope: BodyEnvelope,
    limit: StepThreshold,
    seam_allowed: bool,
    world: &'a dyn PlanetaryWorld,
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
            origin,
            step,
            up,
            eye,
            envelope,
            limit,
            seam_allowed,
            world,
        }
    }
}

impl<'a> StrideTrial for WalkTrial<'a> {
    fn passable(&self) -> bool {
        if self.step.length_squared() <= 1e-6 {
            return false;
        }
        let candidate = self.origin + self.step;
        let surface_pos = self.envelope.constrain_with_height(candidate, self.eye, self.world);
        let clearance = SurfaceClearance::between(self.origin, candidate, surface_pos, self.up, self.world);
        clearance.passable_with(self.seam_allowed)
            && SurfaceStep::new(self.origin, surface_pos, self.up).walkable_with(self.limit)
            && self.envelope.unobstructed_ground(
                candidate,
                surface_pos,
                self.step,
                self.up,
                self.eye,
                self.limit,
                self.world,
            )
    }

    fn destination(&self) -> Vec3 {
        if self.step.length_squared() <= 1e-6 {
            return self.origin;
        }
        self.envelope.constrain_with_height(self.origin + self.step, self.eye, self.world)
    }
}

/// Evaluated mid-air trial step outcome.
pub struct AerialTrial<'a> {
    origin: Vec3,
    tangent: Vec3,
    vertical: Vec3,
    up: Vec3,
    eye: f32,
    envelope: BodyEnvelope,
    limit: StepThreshold,
    world: &'a dyn PlanetaryWorld,
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
            origin,
            tangent,
            vertical,
            up,
            eye,
            envelope,
            limit,
            world,
        }
    }
}

impl<'a> StrideTrial for AerialTrial<'a> {
    fn passable(&self) -> bool {
        if self.tangent.length_squared() <= 1e-6 {
            return false;
        }
        let candidate = self.origin + self.tangent + self.vertical;
        let surface_pos = self.envelope.constrain_with_height(candidate, self.eye, self.world);
        let clearance = SurfaceClearance::between(self.origin, candidate, surface_pos, self.up, self.world);
        let origin_surf = self.envelope.constrain_with_height(self.origin, self.eye, self.world);
        let over_pit = (self.origin - origin_surf).dot(self.up) > 0.02;
        let falling_into_pit = self.vertical.dot(self.up) <= 0.0
            && (self.origin - surface_pos).dot(self.up) < if over_pit { 0.52 } else { -1e-4 }
            && !SurfaceStep::new(origin_surf, surface_pos, self.up).walkable_with(self.limit);
        let center_ok = if falling_into_pit {
            false
        } else if !clearance.cliff_impact(self.limit) {
            true
        } else {
            SurfaceStep::new(origin_surf, surface_pos, self.up).walkable_with(self.limit)
        };
        center_ok
            && self.envelope.unobstructed_aerial(
                self.origin,
                candidate,
                self.tangent,
                self.up,
                self.eye,
                self.limit,
                self.world,
            )
    }

    fn destination(&self) -> Vec3 {
        self.origin + self.tangent + self.vertical
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
