use glam::{Mat4, Vec3};
use fauna::SightCone;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{CameraMount, PhotoShutter, SightMode, YouthAttire};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
use super::super::{Avatar, AvatarOptics, AvatarPresence, BlackBoxDeck, PhotoPort};
use super::EmbodiedAvatar;

impl AvatarOptics for EmbodiedAvatar {
    fn sight_cone(&self) -> SightCone {
        self.sight_cone_for_mode(self.bearing.sight())
    }

    fn sight_cone_for_mode(&self, sight: SightMode) -> SightCone {
        let vpos = self.body.visual_position();
        self.sight_line.sight_cone_at(sight, vpos, &self.pose_at(vpos), self.active_camera_lens())
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        self.project_at_sight(aspect, self.bearing.sight(), self.body.hull_camera_lens())
    }

    fn view_projection_for_sight(&self, aspect: f32, sight: SightMode) -> Mat4 {
        self.project_at_sight(aspect, sight, self.active_camera_lens())
    }

    fn collect_camera_previews(&self, aspect: f32, out: &mut Vec<Mat4>) -> u8 {
        let mut rays = Vec::new();
        let active_slot = self.body.push_hull_camera_rays(&mut rays);
        for (cam_eye, cam_fwd, cam_up) in rays {
            out.push(self.sight_line.project_lens_view(aspect, cam_eye, cam_fwd, cam_up));
        }
        active_slot
    }

    fn emit_intercom_bust(&self, aspect: f32, target: &mut dyn MeshTarget) -> Option<Mat4> {
        let gimbal_angles = self.body.intercom_gimbal()?;
        let angles = gimbal_angles.unwrap_or_else(|| {
            let torso = self.visual_pose().torso_frame();
            let t0 = torso.point(0.0, 0.0, 0.0);
            let tfwd = (torso.point(0.0, 0.0, 1.0) - t0).normalize_or_zero();
            let tright = (torso.point(1.0, 0.0, 0.0) - t0).normalize_or_zero();
            let tup = (torso.point(0.0, 1.0, 0.0) - t0).normalize_or_zero();
            let gaze = self.sight_line.view_direction();
            ((-gaze.dot(tright)).atan2(gaze.dot(tfwd)), gaze.dot(tup).clamp(-0.9, 0.9).asin())
        });
        Some(self.gear.emit_intercom(aspect, angles, target))
    }

    fn emit_inspection_stage(&self, aspect: f32, target: &mut dyn MeshTarget) -> Option<(Mat4, Vec3)> {
        if !self.gear.satchel_open() || self.body.perished() {
            return None;
        }
        let pose = self.visual_pose();
        self.gear.emit_inspection(&pose, target);
        let vp = self.project_at_sight(aspect, SightMode::FirstPersonSight, None);
        Some((vp, pose.inspection_light()))
    }
}

impl AvatarPresence for EmbodiedAvatar {
    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        self.emit_figure(target);
    }

    fn emit_appearance_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        self.emit_figure_for_sight(sight, target);
    }

    fn emit_figure(&self, target: &mut dyn MeshTarget) {
        let sight = if self.body.hull_camera_lens().is_some() {
            SightMode::LookingGlassSight
        } else {
            self.bearing.sight()
        };
        self.gear.emit_appearance(
            &self.visual_pose(),
            sight,
            self.sight_line.view_direction(),
            self.sight_line.surface_up(),
            self.bearing.rhythm(),
            target,
        );
    }

    fn emit_figure_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) {
        if !matches!(sight, SightMode::FirstPersonSight) {
            self.gear.emit_appearance(
                &self.visual_pose(),
                sight,
                self.sight_line.view_direction(),
                self.sight_line.surface_up(),
                self.bearing.rhythm(),
                target,
            );
        }
    }

    fn present_vessel(&self, sink: &mut dyn world::VesselSink) {
        self.body.present_vessel(sink);
    }

    fn planned_mount(&self, world: &dyn PlanetaryWorld) -> Option<world::PlannedMount> {
        (!self.body.driving() && !self.body.perished() && self.body.hull_camera_lens().is_none())
            .then(|| self.gear.planned_mount(self.surface_reach(), world))
            .flatten()
    }

    fn sector_stamp(&self) -> (u8, i32, i32, i32) {
        self.body.sector_stamp()
    }

    fn visible_slices(&self, world: &dyn PlanetaryWorld) -> u32 {
        self.body.visible_slices(world)
    }

    fn drain_meadow(
        &self,
        world: &dyn PlanetaryWorld,
        ring: &mut world::MeadowRing,
        target: &mut dyn MeshTarget,
    ) -> Option<usize> {
        self.body.drain_meadow(world, ring, target)
    }

    fn sync_world_presence(&self, world: &mut dyn PlanetaryWorld) {
        world.sync_presence(self.current_presence());
        world.sync_pilot_craft(self.body.pilot_craft());
    }

    fn report_presence(&self, sink: &mut dyn crate::black_box::PresenceSink) {
        sink.receive_presence(self.current_presence());
    }
}

impl PhotoPort for EmbodiedAvatar {
    fn take_photo_shutter(&mut self) -> PhotoShutter {
        self.bearing.take_shutter()
    }

    fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) {
        self.bearing.apply_polaroid_mosaic(mosaic);
    }

    fn trigger_log_sheet(&mut self, ticks: u16, anomalies: u8) {
        self.bearing.trigger_log_sheet(ticks, anomalies);
    }

    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 {
        self.bearing.polaroid_tilt()
    }
}

impl BlackBoxDeck for EmbodiedAvatar {}

impl Avatar for EmbodiedAvatar {
    fn apply_attire(&mut self, attire: YouthAttire) { self.gear.apply_attire(attire); }
    fn apply_bindings(&mut self, bindings: ActionBindings) { self.gear.apply_bindings(bindings); }
    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        if !self.gear.satchel_open() && !self.body.perished() {
            if self.body.hull_camera_lens().is_some()
                && trigger == InputTrigger::WheelClick
                && self.bearing.press_shutter(CameraMount::Handheld(SightMode::FirstPersonSight))
            {
                return true;
            }
            if self.body.handle_cockpit_trigger(trigger, self.gear.bindings()) {
                return true;
            }
            if self.body.driving()
                && (self.gear.bindings().matches(ControlAction::RollLeft, trigger)
                    || self.gear.bindings().matches(ControlAction::RollRight, trigger))
            {
                return false;
            }
        }
        self.gear.dispatch_trigger(trigger, &mut self.bearing, self.body.perished())
    }
    fn typing_text(&self) -> bool { self.gear.typing_text() }
    fn accept_typed_text(&mut self, text: &str) -> bool { self.gear.accept_text(text) }
    #[cfg(test)]
    fn install_blueprint_archive(&mut self, archive: Box<dyn crate::blueprint::BlueprintArchive>) {
        self.gear.install_blueprint_archive(archive);
    }
    fn blend_subtick(&mut self, alpha: f32) { self.body.blend_subtick(alpha); }
    fn step_fauna(&mut self, dt: f32, world: &mut dyn PlanetaryWorld) {
        let crouching = self.body.posture().crouch_drop() > 0.18;
        if let Some(shove) = world.step_wildlife(dt, self.body.position(), self.sight_line.view_direction(), crouching) {
            self.body.gore(shove, world);
        }
    }
    fn land_on_world(&mut self, world: &dyn PlanetaryWorld) { self.reset_to_world(world); }
    fn solar_chart_snapshot(&self, world: &dyn PlanetaryWorld) -> (crate::drive::OrrerySnapshot, f32, bool) {
        if let Some((snap, accel, fusion)) = self.body.solar_chart_snapshot() {
            let synced = snap.transfer(|_, pos, motion, course| {
                crate::drive::OrrerySnapshot::new(world.celestial_atlas(), pos, motion, course)
            });
            (synced, accel, fusion)
        } else {
            (
                crate::drive::OrrerySnapshot::new(
                    world.celestial_atlas(),
                    self.body.position(),
                    (self.sight_line.view_direction(), Vec3::ZERO),
                    crate::drive::NavigationCourse::Uncharted,
                ),
                0.0,
                false,
            )
        }
    }
    fn install_navigation_course(&mut self, course: crate::drive::NavigationCourse) {
        self.body.install_course(course);
    }
}
