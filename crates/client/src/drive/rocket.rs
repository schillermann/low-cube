use glam::Vec3;
use world::{PilotOrder, PlanetaryWorld, VehicleAssembly};
use acoustics::{AcousticWave, NozzleRoar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use super::space_step::{
    GravitationStep, GyroStep, InertialDrift, NozzleThrustStep, SpaceStep, TouchdownStep,
};
use super::{CraftDrive, SEAT_EYE_SUPPORT};

/// Cockpit 6-DOF directional thruster burn collector implementing `LocomotionTarget` for `FlightConsole`.
struct ConsolePedals {
    planar: (f32, f32),
    vertical: f32,
}

impl ConsolePedals {
    const fn new(planar: (f32, f32), vertical: f32) -> Self {
        Self { planar, vertical }
    }

    const fn idle() -> Self {
        Self::new((0.0, 0.0), 0.0)
    }

    const fn to_order(&self) -> PilotOrder {
        PilotOrder::new(Vec3::new(self.planar.0, self.vertical, self.planar.1))
    }
}

impl LocomotionTarget for ConsolePedals {
    fn travel(&mut self, direction: Vec3) {
        self.planar = (direction.x, direction.y);
    }
    fn jump(&mut self) {
        self.vertical += 1.0;
    }
    fn sneak(&mut self) {
        self.vertical -= 1.0;
    }
    fn sprint(&mut self) {}
}

/// 6-DOF spacecraft propulsion composing velocity, plume channels, and the 5-layer `SpaceStep` chain.
pub struct RocketDrive {
    velocity: Vec3,
    plume: (f32, f32),
    roar_phase: f32,
}

impl RocketDrive {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(velocity: Vec3, plume: (f32, f32), roar_phase: f32) -> Self {
        Self { velocity, plume, roar_phase }
    }

    pub const fn pad_idle() -> Self {
        Self::new(Vec3::ZERO, (0.0, 0.0), 0.0)
    }
}

impl CraftDrive for RocketDrive {
    fn motion_channels(&self) -> (f32, f32) {
        self.plume
    }

    fn free_look(&self) -> bool {
        true
    }

    fn advance(
        &mut self,
        assembly: &mut VehicleAssembly,
        dt: f32,
        controls: &dyn KeyboardControls,
        world: &dyn PlanetaryWorld,
    ) -> (Vec3, Vec3, f32) {
        let mut pedals = ConsolePedals::idle();
        controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
        let order = pedals.to_order();
        let target_plume = order.plume_channels();
        let blend = (14.0 * dt).clamp(0.0, 1.0);
        self.plume = (
            self.plume.0 + (target_plume.0 - self.plume.0) * blend,
            self.plume.1 + (target_plume.1 - self.plume.1) * blend,
        );
        if order.active_burn() {
            let next_phase = self.roar_phase + dt * 3.4;
            if self.roar_phase.floor() != next_phase.floor() {
                NozzleRoar::ignition().thump();
            }
            self.roar_phase = next_phase;
        }
        let chain = TouchdownStep::new(
            GyroStep::new(
                NozzleThrustStep::new(
                    GravitationStep::new(InertialDrift::new(assembly.origin(), self.velocity), world),
                    assembly,
                    order,
                ),
                assembly,
                order,
            ),
            assembly.normal(),
            world,
        );
        let stepped = chain.resolve(dt);
        let (next_origin, next_vel, step_rot) = stepped.transfer(|o, v, r| (o, v, r));
        self.velocity = next_vel;
        if step_rot != glam::Quat::IDENTITY {
            assembly.rotate_3d(step_rot);
        }
        assembly.translate_to(next_origin);
        let next_up = assembly.normal().normalize_or_zero();
        let seat_pos = next_origin + next_up * (SEAT_EYE_SUPPORT + 0.12 + assembly.seat_floor_offset());
        (seat_pos, next_up, 0.0)
    }
}
