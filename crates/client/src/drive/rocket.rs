use glam::{Quat, Vec3};
use world::{AttitudeGuidance, CelestialAtlas, CockpitAxes, PilotOrder, PlanetaryWorld, VehicleAssembly};
use acoustics::{AcousticWave, NozzleRoar};
use crate::attire::{BodyPose, RocketControls};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::menu::GuidanceReadout;
use super::space_step::{
    GravitationStep, GyroStep, InertialDrift, NozzleThrustStep, SpaceStep, TouchdownStep,
};
use super::transfer_arc::{HelmChart, OrrerySnapshot};
use super::{CraftDrive, SEAT_EYE_SUPPORT};

/// Cockpit 6-DOF directional thruster burn collector implementing `LocomotionTarget` for `FlightConsole`.
struct ConsolePedals {
    planar: (f32, f32),
    vertical: f32,
    roll: f32,
    strafe_mode: bool,
}

impl ConsolePedals {
    const fn new(planar: (f32, f32), vertical: f32, roll: f32, strafe_mode: bool) -> Self {
        Self { planar, vertical, roll, strafe_mode }
    }

    const fn idle() -> Self {
        Self::new((0.0, 0.0), 0.0, 0.0, false)
    }

    fn to_order(&self, omega_local: Vec3, guidance: AttitudeGuidance) -> PilotOrder {
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
    const fn new(
        guidance: AttitudeGuidance,
        flip_retro: bool,
        roar_phase: f32,
        pose_motion: (f32, f32),
    ) -> Self {
        Self { guidance, flip_retro, roar_phase, pose_motion }
    }

    const fn pad_engaged() -> Self {
        Self::new(AttitudeGuidance::Engaged, false, 0.0, (0.0, 0.0))
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
}

/// 6-DOF spacecraft propulsion composing `SpaceKinematics`, cockpit controls, `HelmTrim`, and `HelmChart`.
pub struct RocketDrive {
    kinematics: SpaceKinematics,
    controls: RocketControls,
    trim: HelmTrim,
    chart: HelmChart,
}

impl RocketDrive {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        kinematics: SpaceKinematics,
        controls: RocketControls,
        trim: HelmTrim,
        chart: HelmChart,
    ) -> Self {
        Self { kinematics, controls, trim, chart }
    }

    pub const fn pad_with_atlas(atlas: CelestialAtlas) -> Self {
        Self::new(
            SpaceKinematics::at_rest(),
            RocketControls::neutral(),
            HelmTrim::pad_engaged(),
            HelmChart::uncharted(atlas),
        )
    }

    fn step_retro_flip(&mut self, assembly: &mut VehicleAssembly, dt: f32) -> bool {
        if !self.trim.flip_retro {
            return false;
        }
        let speed_sq = self.kinematics.linear.length_squared();
        if speed_sq <= 1.0 || !assembly.attitude_capable() || !self.trim.guidance.engaged() {
            self.trim.flip_retro = false;
            return false;
        }
        let retro_dir = -self.kinematics.linear / speed_sq.sqrt();
        let ship_up = assembly.normal().normalize_or_zero();
        let dot = ship_up.dot(retro_dir).clamp(-1.0, 1.0);
        if dot > 0.9992 {
            let snap = Quat::from_rotation_arc(ship_up, retro_dir);
            assembly.rotate_3d(snap);
            self.kinematics.angular = Vec3::ZERO;
            self.trim.flip_retro = false;
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
        self.trim.pose_motion
    }

    fn shape_pilot_pose(&self, pose: BodyPose) -> BodyPose {
        pose.with_rocket_controls(self.controls)
    }

    fn free_look(&self) -> bool {
        true
    }

    fn helm_optic_channel(&self, assembly: &VehicleAssembly) -> u8 {
        u8::from(assembly.camera_equipped())
    }

    fn toggle_retro_flip(&mut self, assembly: &VehicleAssembly) -> bool {
        if !assembly.attitude_capable()
            || !self.trim.guidance.engaged()
            || self.kinematics.linear.length_squared() <= 1.0
        {
            return false;
        }
        self.trim.flip_retro = !self.trim.flip_retro;
        true
    }

    fn toggle_flight_guidance(&mut self, assembly: &VehicleAssembly) -> bool {
        if !assembly.spacefaring() {
            return false;
        }
        self.trim.guidance = self.trim.guidance.toggled();
        if !self.trim.guidance.engaged() {
            self.trim.flip_retro = false;
        }
        true
    }

    fn guidance_readout(&self) -> GuidanceReadout {
        if self.trim.guidance.engaged() {
            GuidanceReadout::Engaged
        } else {
            GuidanceReadout::Manual
        }
    }

    fn mark_nose_target(
        &mut self,
        rays: ((Vec3, Vec3), (Vec3, Vec3)),
        assembly: &VehicleAssembly,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        self.chart.mark_target(rays, (assembly.origin(), self.kinematics.linear), world)
    }

    fn orrery_snapshot(&self, assembly: &VehicleAssembly) -> Option<OrrerySnapshot> {
        Some(self.chart.snapshot(assembly.origin(), assembly.normal(), self.kinematics.linear))
    }

    fn advance(
        &mut self,
        assembly: &mut VehicleAssembly,
        dt: f32,
        controls: &dyn KeyboardControls,
        world: &dyn PlanetaryWorld,
    ) -> (Vec3, Vec3, f32) {
        self.chart.sync_world(world);
        let mut pedals = ConsolePedals::idle();
        controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
        let axes = CockpitAxes::from_pose(assembly.pose());
        let omega_local = axes.to_local(self.kinematics.angular);
        let order = pedals.to_order(omega_local, self.trim.guidance);
        if order.manual_input() {
            self.trim.flip_retro = false;
        }
        let flipping = self.step_retro_flip(assembly, dt);
        let stick_z = if flipping && pedals.planar.1.abs() < 0.85 { 0.85 } else { pedals.planar.1 };
        let stick_x = if pedals.planar.0.abs() >= pedals.roll.abs() { pedals.planar.0 } else { pedals.roll };
        let target_controls = RocketControls::clamped(stick_x, stick_z, pedals.vertical);
        self.controls = self.controls.blended(target_controls, 14.0 * dt);
        self.trim.pose_motion = order.pose_motion();
        if order.active_burn() || flipping {
            let next_phase = self.trim.roar_phase + dt * 3.4;
            if self.trim.roar_phase.floor() != next_phase.floor() {
                NozzleRoar::ignition().thump();
            }
            self.trim.roar_phase = next_phase;
        }
        let chain = TouchdownStep::new(
            GyroStep::new(
                NozzleThrustStep::new(
                    GravitationStep::new(
                        InertialDrift::new(assembly.origin(), self.kinematics.linear, self.kinematics.angular),
                        world,
                    ),
                    assembly,
                    order,
                ),
                assembly,
                order,
            ),
            assembly,
            world,
        );
        let stepped = chain.resolve(dt);
        let (next_origin, next_vel, next_omega, step_rot) = stepped.transfer(|o, v, w, r| (o, v, w, r));
        self.kinematics = SpaceKinematics::new(next_vel, next_omega);
        if step_rot != Quat::IDENTITY {
            assembly.rotate_3d(step_rot);
        }
        assembly.translate_to(next_origin);
        let next_up = assembly.normal().normalize_or_zero();
        let seat_pos = next_origin + next_up * (SEAT_EYE_SUPPORT + 0.12 + assembly.seat_floor_offset());
        (seat_pos, next_up, 0.0)
    }
}
