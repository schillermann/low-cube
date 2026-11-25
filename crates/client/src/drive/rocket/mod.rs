mod acoustics;
mod controls;
mod residue;

use glam::{Quat, Vec3};
use world::{AttitudeGuidance, CockpitAxes, OpticMount, PilotOrder, PlanetaryWorld, VehicleAssembly};
use crate::attire::{BodyPose, RocketControls};
use crate::controls::KeyboardControls;
use crate::menu::GuidanceReadout;
use super::helm_chart::{HelmChart, OpticTelemetry, OrrerySnapshot};
use super::reentry_step::AtmosphericEntryStep;
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

    fn peak_burn_accel(assembly: &VehicleAssembly, cargo_mass: f32) -> f32 {
        let full = PilotOrder::from_channels(Vec3::Y, Vec3::ZERO, Vec3::ZERO, AttitudeGuidance::Engaged);
        let imp = assembly.sample_loaded_impulse(&full, cargo_mass);
        if assembly.fusion_equipped() {
            imp.keel_fusion_velocity_delta(assembly.aerospace_framed(), 1.0).length()
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

    fn sync_cargo_ballast(&mut self, cargo_mass: f32) {
        self.trim.sync_cargo_ballast(cargo_mass);
    }

    fn drain_flight_residue(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.trim.residue_mut().drain_into(world)
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
        let accel = Self::peak_burn_accel(assembly, self.trim.cargo_ballast());
        self.kinematics.transfer(|vel, _| self.chart.flip_burn_readout(assembly, vel, accel))
    }

    fn solar_chart_snapshot(&self, assembly: &VehicleAssembly) -> Option<(OrrerySnapshot, f32, bool)> {
        let accel = Self::peak_burn_accel(assembly, self.trim.cargo_ballast());
        let fusion = assembly.fusion_equipped();
        Some(self.kinematics.transfer(|vel, _| (self.chart.cockpit_snapshot(assembly, vel), accel, fusion)))
    }

    fn install_course(&mut self, course: NavigationCourse) {
        self.chart.install_course(course);
    }

    fn space_velocity(&self) -> Vec3 {
        self.kinematics.transfer(|vel, _| vel)
    }

    fn apply_handoff(&mut self, next_vel: Vec3, pos_shift: Vec3, world: &dyn PlanetaryWorld) {
        let (_, omega) = self.kinematics.transfer(|l, a| (l, a));
        let g = self.kinematics.proper_g();
        self.kinematics = SpaceKinematics::new(next_vel, omega, g);
        self.chart.shift_frame(pos_shift, world);
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
        let cargo = self.trim.cargo_ballast();
        self.trim.advance_burn(order, assembly, flipping, dt);

        let imp = assembly.sample_loaded_impulse(&order, cargo);
        let thrust_dv = if fusion {
            imp.keel_fusion_velocity_delta(assembly.aerospace_framed(), dt)
        } else {
            imp.velocity_delta(dt)
        };
        let proper_g = if dt > 1e-5 { thrust_dv.length() / (dt * 9.81) } else { 0.0 };

        let burn = (order, cargo);
        let pre_flight = AtmosphericEntryStep::new(
            GyroStep::new(
                NozzleThrustStep::new(
                    GravitationStep::new(
                        InertialDrift::new(assembly.origin(), lin_vel, ang_vel),
                        world,
                    ),
                    assembly,
                    burn,
                    world,
                ),
                assembly,
                burn,
            ),
            assembly,
            world,
        );
        let pre_vel = pre_flight.resolve(dt).transfer(|_, v, _, _| v);
        let stepped = TouchdownStep::new(pre_flight, assembly, world).resolve(dt);
        let (next_origin, next_vel, next_omega, step_rot) = stepped.transfer(|o, v, w, r| (o, v, w, r));
        let fusion_lit = fusion && order.vertical_burn() > 0.35;
        self.trim.residue_mut().note_fusion_exhaust(fusion_lit, next_origin, world);
        self.trim.residue_mut().note_touchdown(next_origin, pre_vel, next_vel, world);

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
