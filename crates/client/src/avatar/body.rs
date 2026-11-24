use glam::{Mat4, Vec3};
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::BodyPose;
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::envelope::{BodyEnvelope, PhysicalEnvelope};
use crate::flight::GravitationalFlight;
use crate::order::LocomotionOrder;
use crate::posture::AvatarPosture;
use super::berth::CockpitBerth;
use super::sight::AvatarSight;
use super::span::BodySpan;

const EYE_SUPPORT_HEIGHT: f32 = 1.0;

/// Encapsulated physical avatar body state: interpolated position span, gravitational flight, posture, and cockpit berth.
pub struct AvatarBody {
    span: BodySpan,
    flight: Box<dyn GravitationalFlight>,
    posture: AvatarPosture,
    berth: CockpitBerth,
}

impl AvatarBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn with_berth(span: BodySpan, flight: Box<dyn GravitationalFlight>, posture: AvatarPosture, berth: CockpitBerth) -> Self {
        Self { span, flight, posture, berth }
    }

    pub fn new(position: Vec3, flight: Box<dyn GravitationalFlight>, posture: AvatarPosture) -> Self {
        Self::with_berth(BodySpan::anchored(position), flight, posture, CockpitBerth::pedestrian())
    }

    pub const fn position(&self) -> Vec3 { self.span.current() }
    pub const fn posture(&self) -> AvatarPosture { self.posture }
    pub const fn standing(&self) -> bool { self.berth.standing() }
    pub const fn driving(&self) -> bool { self.berth.driving() }
    pub fn perished(&self) -> bool { self.posture.perished() }
    pub fn eye_offset(&self, terrain_up: Vec3) -> Vec3 { terrain_up * self.posture.eye_elevation() }
    pub fn relocate(&mut self, next_pos: Vec3) { self.span = self.span.advanced(next_pos); }
    pub fn settle_span(&mut self) { self.span = self.span.settled(); }
    pub fn shape_pose(&self, pose: BodyPose) -> BodyPose { self.berth.shape_pose(pose) }
    pub fn pilot_craft(&self) -> world::PilotCraft { self.berth.pilot_craft() }
    pub fn emit_vehicle(&self, target: &mut dyn MeshTarget) { self.berth.emit(target); }
    pub fn present_vessel(&self, sink: &mut dyn world::VesselSink) { self.berth.present_vessel(sink); }
    pub fn hull_camera_lens(&self) -> Option<(Vec3, Vec3, Vec3, f32)> { self.berth.hull_camera_lens() }
    pub fn push_hull_camera_rays(&self, out: &mut Vec<(Vec3, Vec3, Vec3)>) -> u8 {
        self.berth.push_hull_camera_rays(out)
    }
    pub fn intercom_gimbal(&self) -> Option<Option<(f32, f32)>> { self.berth.intercom_gimbal() }
    pub fn mark_nose_target(&mut self, world: &dyn PlanetaryWorld) -> bool {
        self.berth.mark_nose_target(world)
    }
    pub fn solar_chart_snapshot(&self) -> Option<(crate::drive::OrrerySnapshot, f32, bool)> {
        self.berth.solar_chart_snapshot()
    }
    pub fn install_course(&mut self, course: crate::drive::NavigationCourse) {
        self.berth.install_course(course);
    }

    pub fn reconcile_cosmic_frame(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let (probe_pos, probe_vel) = self.berth.driving_state().unwrap_or((self.span.current(), Vec3::ZERO));
        let Some(handoff) = world.reconcile_cosmic_frame(probe_pos, probe_vel) else {
            return false;
        };
        let next_body = self
            .berth
            .apply_frame_handoff(handoff, world)
            .unwrap_or_else(|| handoff.next_pos());
        self.span = BodySpan::anchored(next_body);
        true
    }
    pub fn emit_hud(
        &self,
        sight: &AvatarSight,
        bindings: &ActionBindings,
        vp: Mat4,
        aspect: f32,
        target: &mut dyn MeshTarget,
    ) {
        self.berth.emit_hud(self.posture, sight, bindings, vp, aspect, target);
    }
    pub fn rotate_view(&mut self, dx: f32, dy: f32, sight_line: &mut AvatarSight) {
        self.berth.rotate_view(dx, dy, sight_line);
    }
    pub fn handle_cockpit_trigger(&mut self, trigger: InputTrigger, bindings: &ActionBindings) -> bool {
        self.berth.handle_cockpit_trigger(trigger, bindings)
    }
    pub fn nourish(&mut self) { self.posture = self.posture.nourish(); }
    pub fn pace_boost(&self) -> f32 { self.posture.pace_boost() }

    pub fn visual_position(&self) -> Vec3 {
        if self.berth.standing() || self.berth.riding() {
            self.span.visual()
        } else {
            self.span.current()
        }
    }

    pub fn blend_subtick(&mut self, alpha: f32) {
        self.span = self.span.blended(alpha);
        self.berth.blend_subtick(alpha);
    }

    pub fn sector_stamp(&self) -> (u8, i32, i32, i32) {
        let p = self.span.current();
        (
            world::ClimateZone::dominant(p).index() as u8,
            ((p.x * 0.125).floor() as i32) * 16,
            ((p.y * 0.125).floor() as i32) * 16,
            ((p.z * 0.125).floor() as i32) * 16,
        )
    }

    pub fn visible_slices(&self, world: &dyn PlanetaryWorld) -> u32 {
        world.visible_slice_mask(self.span.current())
    }

    pub fn drain_meadow(&self, world: &dyn PlanetaryWorld, ring: &mut world::MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        world.drain_stale_meadow_patch(self.span.current(), ring, target)
    }

    pub fn gore(&mut self, impulse: Vec3, world: &dyn PlanetaryWorld) {
        if self.perished() || self.driving() {
            return;
        }
        let pos = self.span.current();
        let up = world.surface_up_at(pos);
        self.span = BodySpan::anchored(world.constrain_to_surface(pos + impulse + up * 0.35, EYE_SUPPORT_HEIGHT));
        self.posture = self.posture.gore(0.36);
        if !self.perished() {
            use acoustics::{AcousticWave, CardiacPulse};
            CardiacPulse::studio().thump();
        }
    }

    pub fn respawn(&mut self, world: &dyn PlanetaryWorld) {
        let spawn = if let Some(beacon) = world.assigned_spawn_point() {
            world.constrain_to_surface(beacon, 1.0)
        } else {
            let probe_y = world
                .celestial_atlas()
                .ephemeris()
                .map_or(1_024.0, |e| (e.proportions().earth_half() * 2.0).max(1_024.0))
                + 64.0;
            [(0.25, 0.25), (2.25, 0.25), (-2.25, 0.25), (0.25, 2.25), (0.25, -2.25)]
                .into_iter()
                .map(|(x, z)| world.constrain_to_surface(Vec3::new(x, probe_y, z), 1.0))
                .fold(world.constrain_to_surface(Vec3::new(0.25, probe_y, 0.25), 1.0), |acc, c| {
                    if c.y > acc.y + 1.5 { c } else { acc }
                })
        };
        self.span = BodySpan::anchored(spawn);
        self.posture = AvatarPosture::standing();
        self.berth = CockpitBerth::pedestrian();
        self.flight.ground();
    }

    pub fn interact_seated_vehicle(
        &mut self, eye: Vec3, view_dir: Vec3, sight_line: &mut AvatarSight, world: &mut dyn PlanetaryWorld,
    ) -> Option<bool> {
        let (pos, mounted) = self.berth.interact_seated(eye, view_dir, sight_line, world)?;
        self.span = BodySpan::anchored(pos);
        if mounted {
            self.posture = AvatarPosture::crouching();
            self.flight.ground();
        }
        Some(true)
    }

    pub fn perch_vehicle(
        &mut self, eye: Vec3, view_dir: Vec3, sight_line: &mut AvatarSight, world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let Some(pos) = self.berth.perch(eye, view_dir, sight_line, world) else {
            return false;
        };
        self.span = BodySpan::anchored(pos);
        self.posture = AvatarPosture::crouching();
        self.flight.ground();
        true
    }

    pub fn reconcile_helm(&mut self, sight_line: &mut AvatarSight, world: &mut dyn PlanetaryWorld) {
        if let Some(fallback) = self.berth.reconcile_helm(sight_line, world) {
            self.span = BodySpan::anchored(fallback);
            self.posture = AvatarPosture::standing();
            self.flight.ground();
        }
    }

    pub fn step_driving(
        &mut self, dt: f32, controls: &dyn KeyboardControls, sight_line: &mut AvatarSight, world: &dyn PlanetaryWorld,
    ) -> bool {
        let Some((next_span, next_posture, ground_flight)) =
            self.berth.step_driving(dt, controls, self.span, self.posture, sight_line, world)
        else {
            return false;
        };
        self.span = next_span;
        self.posture = next_posture;
        if ground_flight {
            self.flight.ground();
        }
        true
    }

    pub fn advance_perish(&mut self, dt: f32) -> (bool, bool) {
        let (next_posture, beat) = self.posture.perish().advance_perish(dt);
        if next_posture.blackout_complete() {
            return (true, false);
        }
        self.posture = next_posture;
        (false, beat)
    }

    pub fn prepare_stride(
        &mut self, order: &LocomotionOrder, standing_env: BodyEnvelope, terrain_up: Vec3, dt: f32, world: &dyn PlanetaryWorld,
    ) -> BodyEnvelope {
        let pos = self.span.current();
        order.dispatch_jump(&mut *self.flight);
        let next_posture = order.advance_posture(self.posture, dt);
        self.posture = if standing_env.crust(world).overhead_obstructed(pos, terrain_up, EYE_SUPPORT_HEIGHT) {
            self.posture.crouch_down(dt)
        } else {
            next_posture
        };
        let active_env = standing_env.with_crouch_and_facing(self.posture.crouch_drop(), Vec3::ZERO);
        self.flight.apply_envelope(active_env);
        active_env
    }

    pub fn apply_flight(
        &mut self, active_env: BodyEnvelope, order: &LocomotionOrder, dt: f32, terrain_up: Vec3, pace: f32, world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        self.flight.apply_envelope(active_env);
        order.advance_flight(&mut *self.flight, dt, self.span.current(), terrain_up, pace, world)
    }

    pub fn advance_leap(&mut self, next_pos: Vec3, terrain_up: Vec3, dt: f32) {
        let leaped = self.posture.advance_leap(self.span.current(), next_pos, terrain_up, dt);
        let (recovered, beat) = leaped.step_vital_recovery(dt);
        self.posture = recovered;
        if beat {
            use acoustics::{AcousticWave, CardiacPulse};
            CardiacPulse::studio().thump();
        }
    }

    pub fn step_ship_deck(
        &mut self,
        order: &LocomotionOrder,
        dt: f32,
        world: &dyn PlanetaryWorld,
    ) -> Option<(Vec3, Vec3)> {
        let (final_pos, deck_up) = self.berth.step_aboard(
            self.span.current(),
            order,
            6.0 * self.pace_boost(),
            EYE_SUPPORT_HEIGHT,
            dt,
            world,
        )?;
        self.posture = order.advance_posture(self.posture, dt);
        self.advance_leap(final_pos, deck_up, dt);
        Some((final_pos, deck_up))
    }

    /// Confines pedestrian flight against deep shafts and subterranean crust ceilings, returning `(final_pos, wedged, lethal_hit)`.
    pub fn step_pedestrian(
        &mut self,
        order: &LocomotionOrder,
        standing_env: BodyEnvelope,
        facing: Vec3,
        terrain_up: Vec3,
        dt: f32,
        world: &dyn PlanetaryWorld,
    ) -> (Vec3, bool, bool) {
        let _ = self.prepare_stride(order, standing_env, terrain_up, dt, world);
        let active_env = standing_env.with_crouch_and_facing(self.posture.crouch_drop(), facing);
        let active_crust = active_env.crust(world);
        let pace = 6.0 * self.pace_boost();
        let raw_next = self.apply_flight(active_env, order, dt, terrain_up, pace, world);
        if world.lethal_at(raw_next) {
            self.span = BodySpan::anchored(raw_next);
            return (raw_next, false, true);
        }
        let in_shaft = active_crust.in_deep_shaft(raw_next, terrain_up, EYE_SUPPORT_HEIGHT);
        let pre_eye = self.eye_offset(terrain_up);
        let next_pos = if in_shaft { world.confine_beneath_crust(raw_next + pre_eye) - pre_eye } else { active_crust.confine(raw_next, terrain_up) };
        self.advance_leap(next_pos, terrain_up, dt);
        let post_eye = self.eye_offset(terrain_up);
        let final_pos = if in_shaft { world.confine_beneath_crust(next_pos + post_eye) - post_eye } else { active_crust.confine(next_pos, terrain_up) };
        let wedged = active_env.wedged_in_shaft(final_pos, terrain_up, EYE_SUPPORT_HEIGHT, world);
        (final_pos, wedged, false)
    }
}
