mod controls;

use glam::{Quat, Vec3};
use world::{CockpitAxes, OpticMount, PlanetaryWorld, VehicleAssembly};
use crate::attire::{BodyPose, RocketControls};
use crate::controls::KeyboardControls;
use crate::menu::GuidanceReadout;
use super::helm_chart::{HelmChart, OpticTelemetry};
use super::space_step::{
    GravitationStep, GyroStep, InertialDrift, NozzleThrustStep, SpaceStep, TouchdownStep,
};
use super::{CraftDrive, SEAT_EYE_SUPPORT};
use self::controls::{ConsolePedals, HelmTrim, SpaceKinematics};

/// 6-DOF spacecraft propulsion composing `SpaceKinematics`, cockpit controls, `HelmTrim`, and `HelmChart`.
pub struct RocketDrive {
    kinematics: SpaceKinematics,
    controls: RocketControls,
    trim: HelmTrim,
    chart: HelmChart,
}

impl RocketDrive {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(
        kinematics: SpaceKinematics,
        controls: RocketControls,
        trim: HelmTrim,
        chart: HelmChart,
    ) -> Self {
        Self { kinematics, controls, trim, chart }
    }

    pub fn boarded(assembly: &VehicleAssembly, world: &dyn PlanetaryWorld) -> Self {
        Self::new(
            SpaceKinematics::at_rest(),
            RocketControls::neutral(),
            HelmTrim::pad_engaged(),
            HelmChart::boarded(assembly, world),
        )
    }
}

impl CraftDrive for RocketDrive {
    fn motion_channels(&self) -> (f32, f32) {
        self.trim.pose_channels()
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
        self.trim.toggle_retro(assembly.attitude_capable(), self.kinematics.moving())
    }

    fn toggle_flight_guidance(&mut self, assembly: &VehicleAssembly) -> bool {
        self.trim.toggle_guidance(assembly.spacefaring())
    }

    fn guidance_readout(&self) -> GuidanceReadout {
        self.trim.readout()
    }

    fn mark_nose_target(
        &mut self,
        rays: ((Vec3, Vec3), (Vec3, Vec3)),
        assembly: &VehicleAssembly,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        self.kinematics.transfer(|vel, _| {
            self.chart.mark_target(rays, (assembly.origin(), vel), world)
        })
    }

    fn optic_telemetry(&self, mount: OpticMount, assembly: &VehicleAssembly) -> OpticTelemetry {
        self.kinematics.transfer(|vel, _| self.chart.optic_telemetry(mount, assembly, vel))
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
        let axes = CockpitAxes::from_pose(assembly.pose());
        let (lin_vel, ang_vel) = self.kinematics.transfer(|l, a| (l, a));
        let omega_local = axes.to_local(ang_vel);
        let order = pedals.to_order(omega_local, self.trim.attitude_guidance());
        if order.manual_input() {
            self.trim.cancel_flip();
        }
        let flipping = self.kinematics.step_retro_flip(&mut self.trim, assembly, dt);
        let (stick_x, stick_z, vert) = pedals.stick_channels(flipping);
        let target_controls = RocketControls::clamped(stick_x, stick_z, vert);
        self.controls = self.controls.blended(target_controls, 14.0 * dt);
        self.trim.advance_burn(order, flipping, dt);
        let chain = TouchdownStep::new(
            GyroStep::new(
                NozzleThrustStep::new(
                    GravitationStep::new(
                        InertialDrift::new(assembly.origin(), lin_vel, ang_vel),
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
        self.chart.sync_flight(assembly, next_vel, world);
        let next_up = assembly.normal().normalize_or_zero();
        let seat_pos = next_origin + next_up * (SEAT_EYE_SUPPORT + 0.12 + assembly.seat_floor_offset());
        (seat_pos, next_up, 0.0)
    }
}
