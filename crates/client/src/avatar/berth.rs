use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::attire::BodyPose;
use crate::controls::{ActionBindings, ControlAction, InputTrigger, KeyboardControls};
use crate::drive::VehiclePilot;
use crate::order::LocomotionOrder;
use crate::posture::AvatarPosture;
use super::deck::DeckFooting;
use super::sight::AvatarSight;
use super::span::BodySpan;

/// Encapsulated cockpit seating, helm optics, ship-deck footing, and vehicle mesh delegation.
pub(super) struct CockpitBerth {
    pilot: VehiclePilot,
    footing: DeckFooting,
    optic_channel: u8,
}

impl CockpitBerth {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(pilot: VehiclePilot, footing: DeckFooting, optic_channel: u8) -> Self {
        Self { pilot, footing, optic_channel }
    }

    pub(super) const fn pedestrian() -> Self {
        Self::new(VehiclePilot::Pedestrian, DeckFooting::Planetary, 0)
    }

    pub(super) const fn standing(&self) -> bool { self.pilot.standing() }
    pub(super) const fn driving(&self) -> bool { self.pilot.driving() }
    pub(super) const fn riding(&self) -> bool { self.pilot.riding() }

    pub(super) fn blend_subtick(&mut self, alpha: f32) {
        self.pilot.blend_subtick(alpha);
    }

    pub(super) fn shape_pose(&self, pose: BodyPose) -> BodyPose {
        self.pilot.shape_pose(pose)
    }

    pub(super) fn pilot_craft(&self) -> world::PilotCraft {
        self.pilot.pilot_craft()
    }

    pub(super) fn emit(&self, target: &mut dyn MeshTarget) {
        self.pilot.emit(target);
    }

    pub(super) fn hull_camera_lens(&self) -> Option<(Vec3, Vec3, Vec3)> {
        if self.optic_channel == 0 || !self.pilot.driving() {
            return None;
        }
        let mut views = Vec::new();
        self.pilot.camera_views(&mut views);
        views.get((self.optic_channel as usize).saturating_sub(1)).copied()
    }

    pub(super) fn handle_cockpit_trigger(&mut self, trigger: InputTrigger, bindings: &ActionBindings) -> bool {
        if !self.pilot.driving() {
            return false;
        }
        if bindings.matches(ControlAction::MoldRight, trigger) || bindings.matches(ControlAction::MoldLeft, trigger) {
            let mut views = Vec::new();
            self.pilot.camera_views(&mut views);
            if !views.is_empty() {
                let total = (views.len() + 1) as u8;
                self.optic_channel = if bindings.matches(ControlAction::MoldRight, trigger) {
                    (self.optic_channel + 1) % total
                } else {
                    (self.optic_channel + total - 1) % total
                };
                return true;
            }
        }
        if (self.optic_channel == 0 && bindings.matches(ControlAction::MoldForm, trigger))
            || trigger == InputTrigger::Key(KeyCode::KeyR)
        {
            return self.pilot.toggle_retro_flip();
        }
        false
    }

    pub(super) fn step_aboard(
        &mut self,
        pos: Vec3,
        order: &LocomotionOrder,
        pace: f32,
        player_height: f32,
        dt: f32,
        world: &dyn PlanetaryWorld,
    ) -> Option<(Vec3, Vec3)> {
        self.footing.step_aboard(pos, order, pace, player_height, dt, world)
    }

    fn mount_assembly(&mut self, assembly: VehicleAssembly, sight_line: &mut AvatarSight, world: &dyn PlanetaryWorld) -> Vec3 {
        self.footing.reset();
        let (pos, heading) = self.pilot.mount(assembly, world);
        self.optic_channel = self.pilot.helm_optic_channel();
        sight_line.anchor_vehicle_heading(heading);
        if self.pilot.free_look() {
            sight_line.constrain_shoulders(false, 0.0);
        }
        pos
    }

    pub(super) fn interact_seated(
        &mut self,
        eye: Vec3,
        view_dir: Vec3,
        sight_line: &mut AvatarSight,
        world: &mut dyn PlanetaryWorld,
    ) -> Option<(Vec3, bool)> {
        if self.pilot.driving() {
            if let Some(perch_pos) = self.pilot.release_wheel(world) {
                self.optic_channel = 0;
                sight_line.constrain_shoulders(true, 1.42);
                return Some((perch_pos, false));
            }
        }
        if self.pilot.seated() {
            if let Some(assembly) = world.board_vehicle(eye, view_dir, true) {
                let pos = self.mount_assembly(assembly, sight_line, world);
                return Some((pos, true));
            }
        }
        None
    }

    pub(super) fn perch(
        &mut self,
        eye: Vec3,
        view_dir: Vec3,
        sight_line: &mut AvatarSight,
        world: &mut dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        self.optic_channel = 0;
        self.footing.reset();
        if let Some(perch) = world.board_vehicle(eye, view_dir, false) {
            return Some(self.mount_assembly(perch, sight_line, world));
        }
        let (craft, local_seat, frame) = world.board_passenger_seat(eye, view_dir)?;
        let (pos, heading) = self.pilot.mount_passenger(craft, local_seat, frame);
        sight_line.anchor_vehicle_heading(heading);
        Some(pos)
    }

    pub(super) fn reconcile_helm(&mut self, sight_line: &mut AvatarSight, world: &mut dyn PlanetaryWorld) -> Option<Vec3> {
        let fallback = self.pilot.reconcile_helm(world)?;
        self.optic_channel = 0;
        sight_line.constrain_shoulders(false, 0.0);
        Some(fallback)
    }

    pub(super) fn step_driving(
        &mut self,
        dt: f32,
        controls: &dyn KeyboardControls,
        span: BodySpan,
        posture: AvatarPosture,
        sight_line: &mut AvatarSight,
        world: &dyn PlanetaryWorld,
    ) -> Option<(BodySpan, AvatarPosture, bool)> {
        let was_riding = self.pilot.riding();
        let (seat_pos, seat_up, steer_px) = self.pilot.step(dt, controls, world)?;
        let next_span = if was_riding && self.pilot.riding() {
            span.advanced(seat_pos)
        } else {
            BodySpan::anchored(seat_pos)
        };
        let (next_posture, ground_flight) = if self.pilot.standing() {
            self.optic_channel = 0;
            sight_line.constrain_shoulders(false, 0.0);
            sight_line.reorient_surface(seat_up, dt);
            (AvatarPosture::standing(), true)
        } else {
            sight_line.constrain_shoulders(!self.pilot.free_look(), 1.42);
            if steer_px.abs() > 1e-5 {
                sight_line.steer_vehicle(steer_px);
            }
            sight_line.reorient_cockpit(seat_up, dt);
            (posture.crouch_down(dt), false)
        };
        Some((next_span, next_posture, ground_flight))
    }
}
