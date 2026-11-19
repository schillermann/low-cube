use glam::Vec3;
use world::PlanetaryWorld;
use crate::envelope::BodyEnvelope;
use crate::flight::GravitationalFlight;
use crate::gait::LocomotionGait;

const EVA_SWIM_SPEED: f32 = 4.8;
const EVA_DRAG: f32 = 1.6;
const MAX_EVA_DRIFT: f32 = 12.0;

/// `GravitationalFlight` decorator enabling weightless 3D EVA spacewalks where `world.apply_pull` vanishes (`0.0g`).
pub struct ZeroGFlight<F> {
    origin: F,
    drift: Vec3,
    ascend_queued: bool,
}

impl<F> ZeroGFlight<F> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: F, drift: Vec3, ascend_queued: bool) -> Self {
        Self { origin, drift, ascend_queued }
    }

    pub const fn wrapping(origin: F) -> Self {
        Self::new(origin, Vec3::ZERO, false)
    }
}

impl<F: GravitationalFlight> GravitationalFlight for ZeroGFlight<F> {
    fn jump(&mut self) {
        self.ascend_queued = true;
        self.origin.jump();
    }

    fn ground(&mut self) {
        self.drift = Vec3::ZERO;
        self.ascend_queued = false;
        self.origin.ground();
    }

    fn apply_envelope(&mut self, envelope: BodyEnvelope) {
        self.origin.apply_envelope(envelope);
    }

    fn advance(
        &mut self,
        dt: f32,
        position: Vec3,
        stride: Vec3,
        gait: &dyn LocomotionGait,
        surface_up: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        let up_kick = std::mem::take(&mut self.ascend_queued);
        if world.apply_pull(position, 1.0).length_squared() > 1e-4 {
            self.drift = Vec3::ZERO;
            return self.origin.advance(dt, position, stride, gait, surface_up, world);
        }
        let cmd_v = if dt > 1e-6 { stride / dt } else { Vec3::ZERO };
        let swim_up = if up_kick { surface_up * EVA_SWIM_SPEED } else { Vec3::ZERO };
        let damped = self.drift * (1.0 - (EVA_DRAG * dt).clamp(0.0, 1.0));
        let next_v = damped + (cmd_v + swim_up) * (3.5 * dt).clamp(0.0, 1.0);
        self.drift = if next_v.length_squared() > MAX_EVA_DRIFT * MAX_EVA_DRIFT {
            next_v.normalize() * MAX_EVA_DRIFT
        } else {
            next_v
        };
        position + self.drift * dt
    }
}
