use glam::{Quat, Vec3};
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

/// 6-DOF spacecraft propulsion composing velocity, plume channels, Flip-and-Burn trim, and the `SpaceStep` chain.
pub struct RocketDrive {
    velocity: Vec3,
    plume: (f32, f32),
    roar_phase: f32,
    flip_retro: bool,
}

impl RocketDrive {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(velocity: Vec3, plume: (f32, f32), roar_phase: f32, flip_retro: bool) -> Self {
        Self { velocity, plume, roar_phase, flip_retro }
    }

    pub const fn pad_idle() -> Self {
        Self::new(Vec3::ZERO, (0.0, 0.0), 0.0, false)
    }

    fn step_retro_flip(&mut self, assembly: &mut VehicleAssembly, dt: f32) -> bool {
        if !self.flip_retro {
            return false;
        }
        let speed_sq = self.velocity.length_squared();
        if speed_sq <= 1.0 || !assembly.attitude_capable() {
            self.flip_retro = false;
            return false;
        }
        let retro_dir = -self.velocity / speed_sq.sqrt();
        let ship_up = assembly.normal().normalize_or_zero();
        let dot = ship_up.dot(retro_dir).clamp(-1.0, 1.0);
        if dot > 0.9992 {
            let snap = Quat::from_rotation_arc(ship_up, retro_dir);
            assembly.rotate_3d(snap);
            self.flip_retro = false;
            return false;
        }
        let max_step = 2.8 * dt;
        let step_rot = if dot < -0.996 {
            let fwd = (assembly.heading() - ship_up * assembly.heading().dot(ship_up)).normalize_or_zero();
            let right = fwd.cross(ship_up).normalize_or_zero();
            let axis = if right.length_squared() > 0.25 { right } else { Vec3::X };
            Quat::from_axis_angle(axis, max_step)
        } else {
            let full = Quat::from_rotation_arc(ship_up, retro_dir);
            let angle = dot.acos().max(1e-4);
            Quat::IDENTITY.slerp(full, (max_step / angle).clamp(0.0, 1.0))
        };
        assembly.rotate_3d(step_rot);
        true
    }
}

impl CraftDrive for RocketDrive {
    fn motion_channels(&self) -> (f32, f32) {
        self.plume
    }

    fn free_look(&self) -> bool {
        true
    }

    fn toggle_retro_flip(&mut self, assembly: &VehicleAssembly) -> bool {
        if !assembly.attitude_capable() || self.velocity.length_squared() <= 1.0 {
            return false;
        }
        self.flip_retro = !self.flip_retro;
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
        if order.planar_active() {
            self.flip_retro = false;
        }
        let flipping = self.step_retro_flip(assembly, dt);
        let mut target_plume = order.plume_channels();
        if flipping && target_plume.1.abs() < 0.85 {
            target_plume.1 = 0.85;
        }
        let blend = (14.0 * dt).clamp(0.0, 1.0);
        self.plume = (
            self.plume.0 + (target_plume.0 - self.plume.0) * blend,
            self.plume.1 + (target_plume.1 - self.plume.1) * blend,
        );
        if order.active_burn() || flipping {
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
        if step_rot != Quat::IDENTITY {
            assembly.rotate_3d(step_rot);
        }
        assembly.translate_to(next_origin);
        let next_up = assembly.normal().normalize_or_zero();
        let seat_pos = next_origin + next_up * (SEAT_EYE_SUPPORT + 0.12 + assembly.seat_floor_offset());
        (seat_pos, next_up, 0.0)
    }
}
