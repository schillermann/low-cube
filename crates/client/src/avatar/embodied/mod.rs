mod optics;

use glam::{Mat4, Quat, Vec3};
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, EvaAttire, SightMode, StrideRhythm};
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
use super::AvatarMotor;

/// Core embodied player entity executing unguarded living kinematics, sight, and hand interactions.
pub(in crate::avatar) struct EmbodiedAvatar {
    body: AvatarBody,
    sight_line: AvatarSight,
    bearing: AvatarBearing,
    gear: AvatarGear,
}

impl EmbodiedAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(in crate::avatar) const fn new(
        body: AvatarBody,
        sight_line: AvatarSight,
        bearing: AvatarBearing,
        gear: AvatarGear,
    ) -> Self {
        Self { body, sight_line, bearing, gear }
    }

    pub(in crate::avatar) fn with_attire_and_bindings(
        initial_position: Vec3,
        flight: Box<dyn GravitationalFlight>,
        attire: EvaAttire,
        bindings: ActionBindings,
    ) -> Self {
        let look_chain = SensitivityLook::calibrated(Box::new(ShaftLook::unobstructed(Box::new(MouseLook::idle()))));
        Self::new(
            AvatarBody::new(initial_position, flight, AvatarPosture::standing()),
            AvatarSight::new(Quat::IDENTITY, Quat::IDENTITY, look_chain, PerspectiveLens::human_eye()),
            AvatarBearing::new(0.0, StrideRhythm::idle(), SightMode::FirstPersonSight),
            AvatarGear::new(attire, PalmEarthworks::empty().reconcile_attire(attire), bindings),
        )
    }

    pub(in crate::avatar) fn provision_satchel(&mut self, provider: &dyn StartingSatchel) {
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
        self.sight_line.reorient_surface(world.surface_up_at(self.body.position()), 32.0);
        self.bearing.reset();
        self.gear.calm_hands();
    }

    pub(super) fn land_from_world(&mut self, world: &dyn PlanetaryWorld) {
        if let Some(welcome) = world.assigned_survivor() {
            self.gear.apply_survivor_manifest(welcome.manifest());
        }
        self.body.land(world);
        self.sight_line.reset_horizon();
        self.sight_line.reorient_surface(world.surface_up_at(self.body.position()), 32.0);
        self.bearing.reset();
        self.gear.calm_hands();
    }

    pub(super) fn flush_world_notes(&mut self, world: &mut dyn PlanetaryWorld) {
        let _ = self.body.drain_flight_residue(world);
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
        self.gear.relinquish_perished();
        self.bearing.step_perish_pulse(dt, beat);
        self.sight_line.transition_perish_lens(self.body.posture(), dt);
    }

    fn project_at_sight(
        &self,
        aspect: f32,
        sight: SightMode,
        placed: Option<(Vec3, Vec3, Vec3, f32)>,
    ) -> Mat4 {
        let vpos = self.body.visual_position();
        let tilt = self.bearing.tilt_radians();
        self.sight_line.project_placed_or_sight(aspect, tilt, sight, self.body.posture(), vpos, &self.pose_at(vpos), placed)
    }

    fn active_camera_lens(&self) -> Option<(Vec3, Vec3, Vec3, f32)> {
        self.body
            .hull_camera_lens()
            .or_else(|| self.gear.placed_camera_lens().map(|(e, f, u)| (e, f, u, 1.0)))
    }

    fn current_presence(&self) -> world::CompanionPresence {
        let pose = self.pose_at(self.body.visual_position());
        world::CompanionPresence::new(0, pose.capture_kinematics(), self.bearing.rhythm().wire_tuple(), self.gear.presence_attire())
    }
}

impl AvatarCondition for EmbodiedAvatar {
    fn perished(&self) -> bool { self.body.perished() }
    fn lethal_contact(&self, world: &dyn PlanetaryWorld) -> bool { world.lethal_at(self.body.position()) }
    fn satchel_open(&self) -> bool { self.gear.satchel_open() }
    fn sheltered_berth(&self) -> bool { !self.body.standing() }
    fn ear_locus(&self) -> Vec3 { self.body.position() }

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

    fn revive_on_earth(&mut self, world: &dyn PlanetaryWorld) {
        self.reset_to_world(world);
    }

    fn reconcile_frame(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.body.reconcile_cosmic_frame(world)
    }
}

impl AvatarMotor for EmbodiedAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        self.body.rotate_view(dx, dy, &mut self.sight_line);
    }

    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        let eye = self.pose_at(self.body.position()).eye_origin();
        if !self.gear.satchel_open() {
            if side < 0.0 && self.body.mark_nose_target(world) {
                self.bearing.calm();
                return true;
            }
            if let Some(outcome) = self.body.interact_seated_vehicle(eye, self.sight_line.view_direction(), &mut self.sight_line, world) {
                self.bearing.calm();
                return outcome;
            }
        }
        self.gear.act_with_hand(side, self.surface_reach(), &mut self.bearing, world)
    }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let reach = self.surface_reach();
        self.gear.release_hand(reach, &mut self.body, &mut self.sight_line, &mut self.bearing, world)
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.body.reconcile_helm(&mut self.sight_line, world);
        let residue = self.body.drain_flight_residue(world);
        if self.body.driving() {
            return residue;
        }
        let (next_rhythm, changed) = self.gear.continue_digging(self.surface_reach(), self.bearing.rhythm(), world);
        self.bearing.apply_rhythm(next_rhythm);
        changed || residue
    }

    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.driving() {
            return false;
        }
        let reach = self.surface_reach();
        self.gear.try_adjust_rig(trigger, &mut self.bearing, reach.eye(), reach.view_dir(), world)
    }

    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.driving() {
            return false;
        }
        let reach = self.surface_reach();
        self.gear.try_cast_relic(trigger, &mut self.bearing, reach.eye(), reach.view_dir(), world)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        self.tick_ambient(dt);
        let pos = self.body.position();
        let terrain_up = world.surface_up_at(pos);
        let cargo = self.gear.cargo_ballast();
        if self.body.step_driving(dt, cargo, controls, &mut self.sight_line, world) {
            let cur = self.body.position();
            self.bearing.step_vehicle_stride(self.body.driving(), cur, world.surface_up_at(cur), dt);
            return;
        }
        self.sight_line.reorient_surface(terrain_up, dt);
        let mut order = LocomotionOrder::idle();
        self.sight_line.drive_order(controls, &mut order);
        let order = self.gear.encumber_order(order);
        if let Some((deck_pos, deck_up)) = self.body.step_ship_deck(&order, dt, world) {
            self.sight_line.reorient_surface(deck_up, dt);
            self.bearing.decay_tilt(dt);
            self.sight_line.advance_lens(&order, self.body.posture(), dt);
            self.bearing.advance_stride(pos, deck_pos, deck_up, dt);
            self.body.relocate(deck_pos);
            self.sight_line.constrain_shoulders(false, 0.0);
            return;
        }
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

    fn emit_reticle_for_aspect(&self, aspect: f32, target: &mut dyn MeshTarget) {
        let vp = self.project_at_sight(aspect, self.bearing.sight(), self.body.hull_camera_lens());
        self.body.emit_hud(&self.sight_line, self.gear.bindings(), vp, aspect, target);
        self.bearing.emit_polaroid(target);
    }
}
