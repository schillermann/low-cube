use glam::{Mat4, Vec3};
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::attire::BodyPose;
use crate::controls::{ActionBindings, ControlAction, InputTrigger, KeyboardControls};
use crate::drive::VehiclePilot;
use crate::menu::{ControlsSlate, ControlsSlateHud, GuidanceReadout, SlateContext};
use crate::order::LocomotionOrder;
use crate::posture::AvatarPosture;
use super::deck::DeckFooting;
use super::helm_optics::HelmOptics;
use super::sight::AvatarSight;
use super::span::BodySpan;

/// Encapsulated cockpit seating, helm optics, ship-deck footing, and toggleable controls legend slate.
pub(super) struct CockpitBerth {
    pilot: VehiclePilot,
    footing: DeckFooting,
    optics: HelmOptics,
    slate: ControlsSlate,
}

impl CockpitBerth {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        pilot: VehiclePilot,
        footing: DeckFooting,
        optics: HelmOptics,
        slate: ControlsSlate,
    ) -> Self {
        Self { pilot, footing, optics, slate }
    }

    pub(super) fn pedestrian() -> Self {
        Self::new(
            VehiclePilot::Pedestrian,
            DeckFooting::Planetary,
            HelmOptics::cockpit(),
            ControlsSlate::Hidden,
        )
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

    pub(super) fn active_hull(&self, world: &dyn PlanetaryWorld) -> Option<(world::CraftIdentity, world::CraftPose)> {
        self.pilot.active_hull(world).or_else(|| self.footing.active_hull(world))
    }

    pub(super) fn emit(&self, target: &mut dyn MeshTarget) {
        self.pilot.emit(target);
    }

    pub(super) fn present_vessel(&self, sink: &mut dyn world::VesselSink) {
        self.pilot.present_vessel(sink);
    }

    pub(super) fn push_hull_camera_rays(&self, out: &mut Vec<(Vec3, Vec3, Vec3)>) -> u8 {
        if !self.pilot.driving() {
            return 0;
        }
        self.optics.push_preview_rays(&self.pilot.collected_views(), out)
    }

    pub(super) fn intercom_gimbal(&self) -> Option<Option<(f32, f32)>> {
        let views = self.pilot.collected_views();
        (!views.is_empty()).then(|| self.optics.gimbal_head_angles())
    }

    pub(super) fn solar_chart_snapshot(&self) -> Option<(crate::drive::OrrerySnapshot, f32, bool)> {
        self.pilot.solar_chart_snapshot()
    }

    pub(super) fn install_course(&mut self, course: crate::drive::NavigationCourse) {
        self.pilot.install_course(course);
    }

    pub(super) fn driving_state(&self) -> Option<(Vec3, Vec3)> {
        self.pilot.driving_state()
    }

    pub(super) fn apply_frame_handoff(&mut self, handoff: world::FrameHandoff, world: &dyn PlanetaryWorld) -> Option<Vec3> {
        self.pilot.apply_frame_handoff(handoff, world)
    }

    pub(super) fn mark_nose_target(&mut self, world: &dyn PlanetaryWorld) -> bool {
        let views = self.pilot.collected_views();
        let Some(rays) = self.optics.nose_sight_rays(&views) else {
            return false;
        };
        self.pilot.mark_nose_target(rays, world)
    }

    pub(super) fn hull_camera_lens(&self) -> Option<(Vec3, Vec3, Vec3, f32)> {
        if self.optics.cockpit_view() || !self.pilot.driving() {
            return None;
        }
        let zoom = self.optics.zoom();
        self.optics
            .resolve_boresight(&self.pilot.collected_views())
            .map(|b| b.lens_with_zoom(zoom))
    }

    pub(super) fn rotate_view(&mut self, dx: f32, dy: f32, sight_line: &mut AvatarSight) {
        if self.pilot.driving() && self.optics.active() {
            self.optics.rotate(dx, dy);
        } else {
            sight_line.rotate(dx, dy);
        }
    }

    pub(super) fn emit_hud(
        &self,
        posture: AvatarPosture,
        sight: &AvatarSight,
        bindings: &ActionBindings,
        vp: Mat4,
        aspect: f32,
        target: &mut dyn MeshTarget,
    ) {
        self.optics.emit_hud(&self.pilot, posture, sight, vp, aspect, target);
        self.emit_controls_slate(bindings, aspect, target);
    }

    fn emit_controls_slate(
        &self,
        bindings: &ActionBindings,
        aspect: f32,
        target: &mut dyn MeshTarget,
    ) {
        if !self.slate.visible() {
            return;
        }
        let context = if !self.pilot.driving() {
            SlateContext::Pedestrian
        } else if self.pilot.guidance_readout() != GuidanceReadout::Absent {
            SlateContext::Spacecraft
        } else {
            SlateContext::Wheeled
        };
        ControlsSlateHud::new(bindings, context, aspect).emit(target);
    }

    pub(super) fn handle_cockpit_trigger(&mut self, trigger: InputTrigger, bindings: &ActionBindings) -> bool {
        if bindings.matches(ControlAction::ControlsLegend, trigger) {
            self.slate = self.slate.toggled();
            return true;
        }
        if !self.pilot.driving() {
            return false;
        }
        if bindings.matches(ControlAction::MoldRight, trigger) || bindings.matches(ControlAction::MoldLeft, trigger) {
            let mut views = Vec::new();
            self.pilot.camera_views(&mut views);
            if !views.is_empty() {
                let total = (views.len() + 1) as u8;
                self.optics.cycle(total, bindings.matches(ControlAction::MoldRight, trigger));
                return true;
            }
        }
        if bindings.matches(ControlAction::FlightGuidance, trigger) {
            return self.pilot.toggle_flight_guidance();
        }
        if self.optics.active() && bindings.matches(ControlAction::MoldForm, trigger) {
            self.optics.cycle_zoom();
            return true;
        }
        if (self.optics.cockpit_view() && bindings.matches(ControlAction::MoldForm, trigger))
            || bindings.matches(ControlAction::RetroFlip, trigger)
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
        self.optics.select_channel(self.pilot.helm_optic_channel());
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
                self.optics.reset();
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
        self.optics.reset();
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
        self.optics.reset();
        sight_line.constrain_shoulders(false, 0.0);
        Some(fallback)
    }

    pub(super) fn drain_flight_residue(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.pilot.drain_flight_residue(world)
    }

    pub(super) fn step_driving(
        &mut self,
        dt: f32,
        cargo_mass: f32,
        controls: &dyn KeyboardControls,
        span: BodySpan,
        posture: AvatarPosture,
        sight_line: &mut AvatarSight,
        world: &dyn PlanetaryWorld,
    ) -> Option<(BodySpan, AvatarPosture, bool)> {
        self.pilot.sync_cargo_ballast(cargo_mass);
        let was_riding = self.pilot.riding();
        let (seat_pos, seat_up, steer_px) = self.pilot.step(dt, controls, world)?;
        let next_span = if was_riding && self.pilot.riding() {
            span.advanced(seat_pos)
        } else {
            BodySpan::anchored(seat_pos)
        };
        let (next_posture, ground_flight) = if self.pilot.standing() {
            self.optics.reset();
            sight_line.constrain_shoulders(false, 0.0);
            sight_line.reorient_surface(seat_up, dt);
            (AvatarPosture::standing(), true)
        } else {
            sight_line.constrain_shoulders(!self.pilot.free_look(), 1.42);
            if steer_px.abs() > 1e-5 {
                sight_line.steer_vehicle(steer_px);
            }
            sight_line.reorient_cockpit(seat_up, dt);
            let g_load = self.pilot.proper_g_load();
            (posture.crouch_down(dt).sustain_high_g(g_load, dt), false)
        };
        Some((next_span, next_posture, ground_flight))
    }
}
