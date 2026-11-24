mod acoustics;
mod controls;

use glam::{Quat, Vec3};
use world::{AttitudeGuidance, CockpitAxes, OpticMount, PilotOrder, PlanetaryWorld, VehicleAssembly};
use crate::attire::{BodyPose, RocketControls};
use crate::controls::KeyboardControls;
use crate::menu::GuidanceReadout;
use super::helm_chart::{HelmChart, OpticTelemetry, OrrerySnapshot};
use super::space_step::{
    GravitationStep, GyroStep, InertialDrift, NozzleThrustStep, SpaceStep, TouchdownStep,
};
use super::transfer_arc::{FlipBurnReadout, NavigationCourse};
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

    fn peak_burn_accel(assembly: &VehicleAssembly) -> f32 {
        let full = PilotOrder::from_channels(Vec3::Y, Vec3::ZERO, Vec3::ZERO, AttitudeGuidance::Engaged);
        let imp = assembly.sample_impulse(&full);
        if assembly.fusion_equipped() {
            imp.fusion_velocity_delta(1.0).length()
        } else {
            imp.velocity_delta(1.0).length()
        }
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

    fn proper_g_load(&self) -> f32 {
        self.kinematics.proper_g()
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

    fn flip_burn_readout(&self, assembly: &VehicleAssembly) -> FlipBurnReadout {
        self.kinematics.transfer(|vel, _| self.chart.flip_burn_readout(assembly, vel))
    }

    fn solar_chart_snapshot(&self, assembly: &VehicleAssembly) -> Option<(OrrerySnapshot, f32, bool)> {
        let accel = Self::peak_burn_accel(assembly);
        let fusion = assembly.fusion_equipped();
        Some(self.kinematics.transfer(|vel, _| (self.chart.cockpit_snapshot(assembly, vel), accel, fusion)))
    }

    fn install_course(&mut self, course: NavigationCourse) {
        self.chart.install_course(course);
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
        let order = self.trim.active_order();
        self.kinematics.transfer(|vel, _| self.chart.optic_telemetry(mount, assembly, vel, &order))
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
        let fusion = assembly.fusion_equipped();
        self.trim.advance_burn(order, assembly, flipping, dt);

        let imp = assembly.sample_impulse(&order);
        let thrust_dv = if fusion { imp.fusion_velocity_delta(dt) } else { imp.velocity_delta(dt) };
        let proper_g = if dt > 1e-5 { thrust_dv.length() / (dt * 9.81) } else { 0.0 };

        let chain = TouchdownStep::new(
            GyroStep::new(
                NozzleThrustStep::new(
                    GravitationStep::new(
                        InertialDrift::new(assembly.origin(), lin_vel, ang_vel),
                        world,
                    ),
                    assembly,
                    order,
                    world,
                ),
                assembly,
                order,
            ),
            assembly,
            world,
        );
        let stepped = chain.resolve(dt);
        let (next_origin, next_vel, next_omega, step_rot) = stepped.transfer(|o, v, w, r| (o, v, w, r));
        self.kinematics = SpaceKinematics::new(next_vel, next_omega, proper_g);
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
