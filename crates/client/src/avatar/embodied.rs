use glam::{Mat4, Quat, Vec3};
use fauna::{SightCone, WoodlandFauna};
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, PhotoShutter, SightMode, StrideRhythm, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::earthworks::{PalmEarthworks, SurfaceReach};
use crate::flight::GravitationalFlight;
use crate::lens::PerspectiveLens;
use crate::order::LocomotionOrder;
use crate::orientation::{MouseLook, SensitivityLook, ShaftLook};
use crate::posture::AvatarPosture;
use crate::server_satchel::StartingSatchel;
use super::bearing::AvatarBearing;
use super::body::AvatarBody;
use super::condition::AvatarCondition;
use super::gear::AvatarGear;
use super::sight::AvatarSight;
use super::{Avatar, AvatarMotor, AvatarOptics, AvatarPresence, BlackBoxDeck, PhotoPort};

/// Core embodied player entity executing unguarded living kinematics, sight, and hand interactions.
pub(super) struct EmbodiedAvatar {
    body: AvatarBody,
    sight_line: AvatarSight,
    bearing: AvatarBearing,
    gear: AvatarGear,
}

impl EmbodiedAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(body: AvatarBody, sight_line: AvatarSight, bearing: AvatarBearing, gear: AvatarGear) -> Self {
        Self { body, sight_line, bearing, gear }
    }

    pub(super) fn with_attire_and_bindings(
        initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire, bindings: ActionBindings,
    ) -> Self {
        let look_chain = SensitivityLook::calibrated(Box::new(ShaftLook::unobstructed(Box::new(MouseLook::idle()))));
        Self::new(
            AvatarBody::new(initial_position, flight, AvatarPosture::standing()),
            AvatarSight::new(Quat::IDENTITY, Quat::IDENTITY, look_chain, PerspectiveLens::human_eye()),
            AvatarBearing::new(0.0, StrideRhythm::idle(), SightMode::FirstPersonSight),
            AvatarGear::new(attire, PalmEarthworks::empty().reconcile_attire(attire), bindings),
        )
    }

    pub(super) fn provision_satchel(&mut self, provider: &dyn StartingSatchel) {
        self.gear.provision_satchel(provider);
    }

    fn pose_at(&self, pos: Vec3) -> BodyPose {
        self.body.shape_pose(self.sight_line.body_pose(pos, self.body.posture(), self.bearing.rhythm()))
    }

    fn visual_pose(&self) -> BodyPose {
        self.pose_at(self.body.visual_position())
    }

    fn surface_reach(&self) -> SurfaceReach {
        let pos = self.body.position();
        self.sight_line.surface_reach(self.pose_at(pos).eye_origin(), pos, self.body.standing(), self.body.driving())
    }

    fn reset_to_world(&mut self, world: &dyn PlanetaryWorld) {
        self.body.respawn(world);
        self.sight_line.reset_horizon();
        self.bearing.reset();
        self.gear.calm_hands();
    }

    fn tick_ambient(&mut self, dt: f32) {
        self.bearing.step_polaroid(dt);
        self.gear.step_timers(dt);
    }

    fn advance_blackout(&mut self, dt: f32, world: &dyn PlanetaryWorld) {
        let (respawn_now, beat) = self.body.advance_perish(dt);
        if respawn_now {
            self.reset_to_world(world);
            return;
        }
        self.bearing.step_perish_pulse(dt, beat);
        self.sight_line.transition_perish_lens(self.body.posture(), dt);
    }

    fn project_at_sight(&self, aspect: f32, sight: SightMode, placed: Option<(Vec3, Vec3, Vec3)>) -> Mat4 {
        let vpos = self.body.visual_position();
        let tilt = self.bearing.tilt_radians() + self.body.posture().vertigo_sway();
        self.sight_line.project_placed_or_sight(aspect, tilt, sight, self.body.posture(), vpos, &self.pose_at(vpos), placed)
    }

    fn current_presence(&self) -> world::CompanionPresence {
        let pose = self.pose_at(self.body.position());
        world::CompanionPresence::new(0, pose.capture_kinematics(), self.bearing.rhythm().wire_tuple(), self.gear.presence_attire())
    }
}

impl AvatarCondition for EmbodiedAvatar {
    fn perished(&self) -> bool { self.body.perished() }
    fn lethal_contact(&self, world: &dyn PlanetaryWorld) -> bool { world.lethal_at(self.body.position()) }
    fn satchel_open(&self) -> bool { self.gear.satchel_open() }
    fn driving(&self) -> bool { self.body.driving() }

    fn abort_hands(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.gear.abort_hands(self.surface_reach(), world)
    }

    fn step_satchel_idle(&mut self, dt: f32) {
        self.tick_ambient(dt);
        self.body.settle_span();
        self.bearing.calm();
    }

    fn step_blackout(&mut self, dt: f32, world: &dyn PlanetaryWorld) {
        self.tick_ambient(dt);
        self.body.settle_span();
        self.advance_blackout(dt, world);
    }
}

impl AvatarOptics for EmbodiedAvatar {
    fn sight_cone(&self) -> SightCone { self.sight_cone_for_mode(self.bearing.sight()) }
    fn sight_cone_for_mode(&self, sight: SightMode) -> SightCone {
        let vpos = self.body.visual_position();
        self.sight_line.sight_cone_at(sight, vpos, &self.pose_at(vpos), self.gear.placed_camera_lens())
    }
    fn view_projection_matrix(&self, aspect: f32) -> Mat4 { self.project_at_sight(aspect, self.bearing.sight(), None) }
    fn view_projection_for_sight(&self, aspect: f32, sight: SightMode) -> Mat4 { self.project_at_sight(aspect, sight, self.gear.placed_camera_lens()) }
    fn surface_rotation(&self) -> Quat { self.sight_line.surface_rotation() }
}

impl AvatarPresence for EmbodiedAvatar {
    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        self.gear.emit_appearance(&self.visual_pose(), self.bearing.sight(), self.sight_line.view_direction(), self.sight_line.surface_up(), self.bearing.rhythm(), target);
    }

    fn emit_appearance_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        if !matches!(sight, SightMode::FirstPersonSight) {
            self.gear.emit_appearance(&self.visual_pose(), sight, self.sight_line.view_direction(), self.sight_line.surface_up(), self.bearing.rhythm(), target);
        }
    }

    fn sector_stamp(&self) -> (u8, i32, i32, i32) { self.body.sector_stamp() }
    fn visible_slices(&self, world: &dyn PlanetaryWorld) -> u32 { self.body.visible_slices(world) }
    fn drain_meadow(&self, world: &dyn PlanetaryWorld, ring: &mut world::MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
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
    fn take_photo_shutter(&mut self) -> PhotoShutter { self.bearing.take_shutter() }
    fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) { self.bearing.apply_polaroid_mosaic(mosaic); }
    fn trigger_log_sheet(&mut self, ticks: u16, anomalies: u8) { self.bearing.trigger_log_sheet(ticks, anomalies); }
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { self.bearing.polaroid_tilt() }
}

impl BlackBoxDeck for EmbodiedAvatar {}

impl AvatarMotor for EmbodiedAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) { self.sight_line.rotate(dx, dy); }

    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        let reach = self.surface_reach();
        if !self.gear.satchel_open() {
            if let Some(outcome) = self.body.interact_seated_vehicle(reach.eye(), reach.view_dir(), &mut self.sight_line, world) {
                self.bearing.calm();
                return outcome;
            }
        }
        self.gear.act_with_hand(side, reach, &mut self.body, &mut self.bearing, world)
    }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let reach = self.surface_reach();
        self.gear.release_hand(reach, &mut self.body, &mut self.sight_line, &mut self.bearing, world)
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.body.reconcile_helm(&mut self.sight_line, world);
        let (next_rhythm, changed) = self.gear.continue_digging(self.surface_reach(), self.bearing.rhythm(), world);
        self.bearing.apply_rhythm(next_rhythm);
        changed
    }

    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        let reach = self.surface_reach();
        self.gear.try_adjust_rig(trigger, &mut self.bearing, reach.eye(), reach.view_dir(), world)
    }

    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        let reach = self.surface_reach();
        self.gear.try_cast_relic(trigger, &mut self.bearing, reach.eye(), reach.view_dir(), world)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        self.tick_ambient(dt);
        let pos = self.body.position();
        let terrain_up = world.surface_up_at(pos);
        if self.body.step_driving(dt, controls, &mut self.sight_line, world) {
            let cur = self.body.position();
            self.bearing.step_vehicle_stride(self.body.driving(), cur, world.surface_up_at(cur), dt);
            return;
        }
        self.sight_line.reorient_surface(terrain_up, dt);
        let mut order = LocomotionOrder::idle();
        self.sight_line.drive_order(controls, &mut order);
        let order = self.gear.encumber_order(order);
        let facing = self.sight_line.view_direction();
        self.bearing.decay_tilt(dt);
        let (final_pos, wedged, lethal) = self.body.step_pedestrian(
            &order, self.gear.standing_envelope(facing), facing, terrain_up, dt, world,
        );
        if lethal {
            self.advance_blackout(dt, world);
            return;
        }
        self.sight_line.advance_lens(&order, self.body.posture(), dt);
        self.bearing.advance_stride(pos, final_pos, terrain_up, dt);
        self.body.relocate(final_pos);
        self.sight_line.constrain_shoulders(wedged, self.gear.shaft_yaw_play());
    }

    fn hunt_fauna(&mut self, side: f32, fauna: &mut WoodlandFauna, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        self.gear.strike_fauna(side, self.surface_reach(), &mut self.bearing, fauna, world)
    }

    fn emit_reticle(&self, target: &mut dyn MeshTarget) {
        self.body.posture().emit_reticle(target);
        self.bearing.emit_polaroid(target);
    }
}

impl Avatar for EmbodiedAvatar {
    fn apply_attire(&mut self, attire: YouthAttire) { self.gear.apply_attire(attire); }
    fn apply_bindings(&mut self, bindings: ActionBindings) { self.gear.apply_bindings(bindings); }
    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        self.gear.dispatch_trigger(trigger, &mut self.bearing, self.body.perished())
    }
    fn blend_subtick(&mut self, alpha: f32) { self.body.blend_subtick(alpha); }
    fn step_fauna(&mut self, dt: f32, fauna: &mut WoodlandFauna, world: &dyn PlanetaryWorld) {
        let crouching = self.body.posture().crouch_drop() > 0.18;
        if let Some(shove) = fauna.step_spatial(dt, self.body.position(), self.sight_line.view_direction(), crouching, world) {
            self.body.gore(shove, world);
        }
    }
    fn land_on_world(&mut self, world: &dyn PlanetaryWorld) { self.reset_to_world(world); }
}
