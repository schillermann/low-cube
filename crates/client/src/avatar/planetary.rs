use glam::{Mat4, Quat, Vec3};
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, PhotoShutter, SightMode, StrideRhythm, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::earthworks::{PalmEarthworks, SurfaceReach};
use fauna::{SightCone, WoodlandFauna};
use crate::flight::GravitationalFlight;
#[cfg(test)]
use crate::flight::SurfaceFlight;
use crate::lens::PerspectiveLens;
use crate::order::LocomotionOrder;
use crate::orientation::{MouseLook, SensitivityLook};
use crate::posture::AvatarPosture;
use super::bearing::AvatarBearing;
use super::body::AvatarBody;
use super::gear::AvatarGear;
use super::sight::AvatarSight;
use super::Avatar;

/// Fully encapsulated player entity composed of body, sight line, kinematic bearing, and gear.
pub struct PlanetaryAvatar {
    body: AvatarBody,
    sight_line: AvatarSight,
    bearing: AvatarBearing,
    gear: AvatarGear,
}

impl PlanetaryAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(body: AvatarBody, sight_line: AvatarSight, bearing: AvatarBearing, gear: AvatarGear) -> Self {
        Self { body, sight_line, bearing, gear }
    }

    pub fn with_attire_and_bindings(
        initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire, bindings: ActionBindings,
    ) -> Self {
        Self::new(
            AvatarBody::new(initial_position, flight, AvatarPosture::standing()),
            AvatarSight::new(
                Quat::IDENTITY,
                Quat::IDENTITY,
                SensitivityLook::calibrated(Box::new(MouseLook::idle())),
                PerspectiveLens::human_eye(),
            ),
            AvatarBearing::new(0.0, StrideRhythm::idle(), SightMode::FirstPersonSight),
            AvatarGear::new(attire, PalmEarthworks::empty().reconcile_attire(attire), bindings),
        )
    }

    pub fn with_starting_satchel(mut self, provider: &dyn crate::server_satchel::StartingSatchel) -> Self {
        self.gear.provision_satchel(provider);
        self
    }

    #[cfg(test)]
    pub fn with_test_engines(self) -> Self {
        self.with_starting_satchel(&crate::server_satchel::TestKitSatchel::new())
    }
    #[cfg(test)]
    pub fn with_attire(initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire) -> Self {
        Self::with_attire_and_bindings(initial_position, flight, attire, ActionBindings::seeded())
    }
    #[cfg(test)]
    pub fn with_flight(initial_position: Vec3, flight: Box<dyn GravitationalFlight>) -> Self {
        Self::with_attire(initial_position, flight, YouthAttire::squire())
    }
    #[cfg(test)]
    pub fn at(initial_position: Vec3) -> Self {
        Self::with_flight(initial_position, Box::new(SurfaceFlight::terrestrial(1.0)))
    }

    fn current_pose(&self) -> BodyPose {
        self.body.shape_pose(self.sight_line.body_pose(self.body.position(), self.body.posture(), self.bearing.rhythm()))
    }

    fn surface_reach(&self) -> SurfaceReach {
        self.sight_line.surface_reach(
            self.current_pose().eye_origin(),
            self.body.position(),
            self.body.standing(),
            self.body.driving(),
        )
    }

    fn act_with_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() || world.lethal_at(self.body.position()) {
            return false;
        }
        let reach = self.surface_reach();
        if !self.gear.satchel_open() {
            if let Some(outcome) = self.body.interact_seated_vehicle(reach.eye(), reach.view_dir(), &mut self.sight_line, world) {
                self.bearing.calm();
                return outcome;
            }
        }
        self.gear.act_with_hand(side, reach, &mut self.body, &mut self.bearing, world)
    }

    fn step_perish(&mut self, dt: f32, world: &dyn PlanetaryWorld) {
        let (respawn_now, beat) = self.body.advance_perish(dt);
        if respawn_now {
            self.body.respawn(world);
            self.sight_line.reset_horizon();
            self.bearing.reset();
            return;
        }
        self.bearing.tilt_perish(dt);
        self.sight_line.transition_perish_lens(self.body.posture(), dt);
        if beat {
            use acoustics::AcousticWave;
            acoustics::MonitoredPulse::clinical().thump();
        }
    }
}

impl Avatar for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        if !self.gear.satchel_open() && !self.body.perished() {
            self.sight_line.rotate(dx, dy);
        }
    }

    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(-1.0, world) }
    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(1.0, world) }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() || self.gear.satchel_open() { return false; }
        let reach = self.surface_reach();
        self.gear.release_hand(reach, &mut self.body, &mut self.sight_line, &mut self.bearing, world)
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() || self.gear.satchel_open() { return false; }
        let (next_rhythm, changed) = self.gear.continue_digging(self.surface_reach(), self.bearing.rhythm(), world);
        self.bearing.apply_rhythm(next_rhythm);
        changed
    }

    fn apply_attire(&mut self, attire: YouthAttire) { self.gear.apply_attire(attire); }
    fn apply_bindings(&mut self, bindings: ActionBindings) { self.gear.apply_bindings(bindings); }
    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        self.gear.dispatch_trigger(trigger, &mut self.bearing, self.body.perished())
    }
    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() { return false; }
        let reach = self.surface_reach();
        self.gear.try_adjust_rig(trigger, &mut self.bearing, reach.eye(), reach.view_dir(), world)
    }
    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() || world.lethal_at(self.body.position()) { return false; }
        let reach = self.surface_reach();
        self.gear.try_cast_relic(trigger, &mut self.bearing, reach.eye(), reach.view_dir(), world)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        self.bearing.step_polaroid(dt);
        self.gear.step_timers(dt);
        let pos = self.body.position();
        if self.gear.satchel_open() || self.body.perished() || world.lethal_at(pos) {
            if self.body.perished() || world.lethal_at(pos) { self.step_perish(dt, world); } else { self.bearing.calm(); }
            return;
        }
        let terrain_up = world.surface_up_at(pos);
        self.sight_line.reorient_surface(terrain_up, dt);
        if self.body.step_driving(dt, controls, &mut self.sight_line, world) {
            if self.body.driving() { self.bearing.calm(); } else { let cur = self.body.position(); self.bearing.advance_stride(cur, cur, terrain_up, dt); }
            self.bearing.decay_tilt(dt);
            return;
        }
        let mut order = LocomotionOrder::idle();
        self.sight_line.drive_order(controls, &mut order);
        let facing = self.sight_line.view_direction();
        self.bearing.decay_tilt(dt);
        let (final_pos, wedged, lethal) = self.body.step_pedestrian(
            &order, self.gear.carried_burden(), self.gear.standing_envelope(facing), facing, terrain_up, dt, world,
        );
        if lethal {
            self.step_perish(dt, world);
            return;
        }
        self.sight_line.advance_lens(&order, self.body.posture(), dt);
        self.bearing.advance_stride(pos, final_pos, terrain_up, dt);
        self.body.relocate(final_pos);
        self.sight_line.constrain_shoulders(wedged, self.gear.shaft_yaw_play());
    }

    fn step_fauna(&mut self, dt: f32, fauna: &mut WoodlandFauna, world: &dyn PlanetaryWorld) {
        let crouching = self.body.posture().crouch_drop() > 0.18;
        if let Some(shove) = fauna.step_spatial(dt, self.body.position(), self.sight_line.view_direction(), crouching, world) {
            self.body.gore(shove, world);
        }
    }

    fn hunt_fauna(&mut self, side: f32, fauna: &mut WoodlandFauna, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        if self.body.perished() || self.gear.satchel_open() || self.body.driving() || world.lethal_at(self.body.position()) {
            return (false, false);
        }
        self.gear.strike_fauna(side, self.surface_reach(), &mut self.bearing, fauna, world)
    }

    fn sight_cone(&self) -> SightCone {
        self.sight_cone_for_mode(self.bearing.sight())
    }

    fn sight_cone_for_mode(&self, sight: SightMode) -> SightCone {
        if let Some((eye, fwd, _)) = self.gear.placed_camera_lens() {
            return SightCone::from_view(eye, fwd);
        }
        if matches!(sight, SightMode::FirstPersonSight) {
            SightCone::from_view(self.current_pose().eye_origin(), self.sight_line.view_direction())
        } else {
            SightCone::omnidirectional(self.body.position())
        }
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let tilt = self.bearing.tilt_radians() + self.body.posture().vertigo_sway();
        self.sight_line.project_view(aspect, tilt, self.bearing.sight(), self.body.posture(), self.body.position(), &self.current_pose())
    }

    fn view_projection_for_sight(&self, aspect: f32, sight: SightMode) -> Mat4 {
        if let Some((eye, fwd, up)) = self.gear.placed_camera_lens() {
            return self.sight_line.project_lens_view(aspect, eye, fwd, up);
        }
        let tilt = self.bearing.tilt_radians() + self.body.posture().vertigo_sway();
        self.sight_line.project_view(aspect, tilt, sight, self.body.posture(), self.body.position(), &self.current_pose())
    }

    fn surface_rotation(&self) -> Quat { self.sight_line.surface_rotation() }
    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        self.gear.emit_appearance(&self.current_pose(), self.bearing.sight(), self.sight_line.view_direction(), self.sight_line.surface_up(), self.bearing.rhythm(), target);
    }
    fn emit_appearance_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        if !matches!(sight, SightMode::FirstPersonSight) {
            self.gear.emit_appearance(&self.current_pose(), sight, self.sight_line.view_direction(), self.sight_line.surface_up(), self.bearing.rhythm(), target);
        }
    }
    fn emit_surroundings(&self, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        world.emit_meadow_around(self.body.position(), self.sight_line.view_direction(), target);
    }
    fn meadow_anchor(&self) -> (i32, i32, i32) {
        let p = self.body.position();
        (((p.x * 0.25).floor() as i32) * 8, ((p.y * 0.25).floor() as i32) * 8, ((p.z * 0.25).floor() as i32) * 8)
    }
    fn take_photo_shutter(&mut self) -> PhotoShutter { self.bearing.take_shutter() }
    fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) { self.bearing.apply_polaroid_mosaic(mosaic); }
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { self.bearing.polaroid_tilt() }
    fn emit_reticle(&self, target: &mut dyn MeshTarget) {
        if !self.gear.satchel_open() {
            self.body.posture().emit_reticle(target);
            self.bearing.emit_polaroid(target);
        }
    }

    fn sync_world_presence(&self, world: &mut dyn PlanetaryWorld) {
        let presence = world::CompanionPresence::new(
            0, self.current_pose().capture_kinematics(), self.bearing.rhythm().wire_tuple(), self.gear.attire_wire(),
        );
        world.sync_presence(presence);
    }

    fn land_on_world(&mut self, world: &dyn PlanetaryWorld) {
        self.body.respawn(world);
        self.sight_line.reset_horizon();
        self.bearing.reset();
    }
}
