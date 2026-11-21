use glam::{Quat, Vec3};
use world::{AttitudeGuidance, PilotOrder, VehicleAssembly};
use acoustics::{AcousticWave, NozzleRoar};
use crate::controls::LocomotionTarget;
use crate::menu::GuidanceReadout;

/// Cockpit 6-DOF directional thruster burn collector implementing `LocomotionTarget` for `FlightConsole`.
pub(super) struct ConsolePedals {
    planar: (f32, f32),
    vertical: f32,
    roll: f32,
    strafe_mode: bool,
}

impl ConsolePedals {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        planar: (f32, f32),
        vertical: f32,
        roll: f32,
        strafe_mode: bool,
    ) -> Self {
        Self { planar, vertical, roll, strafe_mode }
    }

    pub(super) const fn idle() -> Self {
        Self::new((0.0, 0.0), 0.0, 0.0, false)
    }

    pub(super) fn stick_channels(&self, flipping: bool) -> (f32, f32, f32) {
        let stick_z = if flipping && self.planar.1.abs() < 0.85 { 0.85 } else { self.planar.1 };
        let stick_x = if self.planar.0.abs() >= self.roll.abs() { self.planar.0 } else { self.roll };
        (stick_x, stick_z, self.vertical)
    }

    pub(super) fn to_order(&self, omega_local: Vec3, guidance: AttitudeGuidance) -> PilotOrder {
        let (linear, angular) = if self.strafe_mode {
            (
                Vec3::new(self.planar.0, self.vertical, self.planar.1),
                Vec3::new(0.0, 0.0, self.roll),
            )
        } else {
            (
                Vec3::new(0.0, self.vertical, self.planar.1 * 0.90),
                Vec3::new(-self.planar.1, -self.planar.0, self.roll),
            )
        };
        PilotOrder::from_channels(linear, angular, omega_local, guidance)
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
    fn roll(&mut self, roll_dir: f32) {
        self.roll = roll_dir.clamp(-1.0, 1.0);
    }
    fn strafe_shift(&mut self) {
        self.strafe_mode = true;
    }
}

/// Encapsulated flight guidance computer toggle, retro-flip state, acoustic phase, and packed pose telemetry.
#[derive(Clone, Copy, Debug)]
pub(super) struct HelmTrim {
    guidance: AttitudeGuidance,
    flip_retro: bool,
    roar_phase: f32,
    pose_motion: (f32, f32),
}

impl HelmTrim {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        guidance: AttitudeGuidance,
        flip_retro: bool,
        roar_phase: f32,
        pose_motion: (f32, f32),
    ) -> Self {
        Self { guidance, flip_retro, roar_phase, pose_motion }
    }

    pub(super) const fn pad_engaged() -> Self {
        Self::new(AttitudeGuidance::Engaged, false, 0.0, (0.0, 0.0))
    }

    pub(super) const fn pose_channels(self) -> (f32, f32) {
        self.pose_motion
    }

    pub(super) const fn attitude_guidance(self) -> AttitudeGuidance {
        self.guidance
    }

    pub(super) fn cancel_flip(&mut self) {
        self.flip_retro = false;
    }

    pub(super) const fn flipping(self) -> bool {
        self.flip_retro
    }

    pub(super) fn toggle_retro(&mut self, capable: bool, moving: bool) -> bool {
        if !capable || !self.guidance.engaged() || !moving {
            return false;
        }
        self.flip_retro = !self.flip_retro;
        true
    }

    pub(super) fn toggle_guidance(&mut self, spacefaring: bool) -> bool {
        if !spacefaring {
            return false;
        }
        self.guidance = self.guidance.toggled();
        if !self.guidance.engaged() {
            self.flip_retro = false;
        }
        true
    }

    pub(super) fn readout(self) -> GuidanceReadout {
        if self.guidance.engaged() {
            GuidanceReadout::Engaged
        } else {
            GuidanceReadout::Manual
        }
    }

    pub(super) fn advance_burn(&mut self, order: PilotOrder, flipping: bool, dt: f32) {
        self.pose_motion = order.pose_motion();
        if order.active_burn() || flipping {
            let next_phase = self.roar_phase + dt * 3.4;
            if self.roar_phase.floor() != next_phase.floor() {
                NozzleRoar::ignition().thump();
            }
            self.roar_phase = next_phase;
        }
    }
}

/// Encapsulated linear and angular 6-DOF spacecraft momentum vectors.
#[derive(Clone, Copy, Debug)]
pub(super) struct SpaceKinematics {
    linear: Vec3,
    angular: Vec3,
}

impl SpaceKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }

    pub(super) const fn at_rest() -> Self {
        Self::new(Vec3::ZERO, Vec3::ZERO)
    }

    pub(super) fn transfer<R>(self, f: impl FnOnce(Vec3, Vec3) -> R) -> R {
        f(self.linear, self.angular)
    }

    pub(super) fn moving(self) -> bool {
        self.linear.length_squared() > 1.0
    }

    pub(super) fn step_retro_flip(
        &mut self,
        trim: &mut HelmTrim,
        assembly: &mut VehicleAssembly,
        dt: f32,
    ) -> bool {
        if !trim.flipping() {
            return false;
        }
        let speed_sq = self.linear.length_squared();
        if speed_sq <= 1.0 || !assembly.attitude_capable() || !trim.attitude_guidance().engaged() {
            trim.cancel_flip();
            return false;
        }
        let retro_dir = -self.linear / speed_sq.sqrt();
        let ship_up = assembly.normal().normalize_or_zero();
        let dot = ship_up.dot(retro_dir).clamp(-1.0, 1.0);
        if dot > 0.9992 {
            assembly.rotate_3d(Quat::from_rotation_arc(ship_up, retro_dir));
            self.angular = Vec3::ZERO;
            trim.cancel_flip();
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
