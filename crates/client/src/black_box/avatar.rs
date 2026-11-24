use glam::{Mat4, Quat, Vec3};
use fauna::SightCone;
use world::{CompanionPresence, MeshTarget, PlanetaryWorld, PresenceAttire, PresenceKinematics};
use crate::attire::{PhotoShutter, SightMode, YouthAttire};
use crate::avatar::{Avatar, AvatarMotor, AvatarOptics, AvatarPresence, BlackBoxDeck, PhotoPort};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls, LocomotionTarget};
use super::{AnomalyKind, FlightEvent, FlightTape, IncidentMedium, PresenceSink, TickTrace};

/// Stack-allocated `PresenceSink` capturing an avatar's exact 3D foot position and orientation without getters.
#[derive(Clone, Copy, Debug)]
pub struct PresenceProbe {
    captured: CompanionPresence,
}

impl PresenceProbe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(captured: CompanionPresence) -> Self {
        Self { captured }
    }

    pub const fn at_origin() -> Self {
        let kin = PresenceKinematics::new(Vec3::ZERO, Quat::IDENTITY, Quat::IDENTITY, (0.8, 0.0));
        Self::new(CompanionPresence::new(0, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire()))
    }

    pub fn position(&self) -> Vec3 {
        self.captured.transfer(|_, kin, _, _| kin.transfer(|pos, _, _, _, _| pos))
    }

    pub fn view_direction(&self) -> Vec3 {
        self.captured.transfer(|_, kin, _, _| {
            kin.transfer(|_, _, view_rot, _, _| (view_rot * Vec3::NEG_Z).normalize_or_zero())
        })
    }

    #[cfg(test)]
    pub const fn presence(&self) -> CompanionPresence {
        self.captured
    }
}

impl PresenceSink for PresenceProbe {
    fn receive_presence(&mut self, presence: CompanionPresence) {
        self.captured = presence;
    }
}

struct OrderSample {
    intent: (f32, f32),
    flags: u8,
}

impl OrderSample {
    const fn idle() -> Self {
        Self { intent: (0.0, 0.0), flags: 0 }
    }
}

impl LocomotionTarget for OrderSample {
    fn travel(&mut self, direction: Vec3) { self.intent = (direction.z, direction.x); }
    fn jump(&mut self) { self.flags |= 1; }
    fn sprint(&mut self) { self.flags |= 2; }
    fn sneak(&mut self) { self.flags |= 4; }
}

/// Pure-OOP `Avatar` decorator recording a rolling 120 Hz flight black-box ring without touching domain classes.
pub struct BlackBoxAvatar {
    origin: Box<dyn Avatar>,
    tape: Box<dyn FlightTape>,
    world_blocks: u32,
}

impl BlackBoxAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn Avatar>, tape: Box<dyn FlightTape>, world_blocks: u32) -> Self {
        Self { origin, tape, world_blocks }
    }

    pub fn wrapping(origin: Box<dyn Avatar>, tape: Box<dyn FlightTape>) -> Self {
        Self::new(origin, tape, 0)
    }

    fn calibrate_scale(&mut self, world: &dyn PlanetaryWorld) -> u32 {
        if self.world_blocks == 0 {
            let probe = world.constrain_to_surface(Vec3::new(0.25, 4_096.0, 0.25), 0.0);
            let approx_half = probe.y.max(4.0).round() as u32;
            self.world_blocks = (approx_half * 2).max(8);
        }
        self.world_blocks
    }

    fn classify_anomaly(pos: Vec3, view: Vec3, up: Vec3, half: f32, world: &dyn PlanetaryWorld) -> u8 {
        if !pos.is_finite() || !view.is_finite() {
            return AnomalyKind::NonFiniteKinematics.code();
        }
        if world.lethal_at(pos) {
            return AnomalyKind::LethalCoreContact.code();
        }
        let surf = world.constrain_to_surface(pos, 1.0);
        let depth = (surf - pos).dot(up);
        if depth > 0.45 && depth < 2.4 && pos.abs().max_element() < half - 0.15 {
            let head = pos + up * 0.72;
            let confined = world.confine_beneath_crust(head);
            if (confined - head).length_squared() < 1e-5 {
                return AnomalyKind::CrustPenetration.code();
            }
        }
        0
    }
}

impl AvatarOptics for BlackBoxAvatar {
    fn sight_cone(&self) -> SightCone { self.origin.sight_cone() }
    fn sight_cone_for_mode(&self, sight: SightMode) -> SightCone { self.origin.sight_cone_for_mode(sight) }
    fn view_projection_matrix(&self, aspect: f32) -> Mat4 { self.origin.view_projection_matrix(aspect) }
    fn view_projection_for_sight(&self, aspect: f32, sight: SightMode) -> Mat4 { self.origin.view_projection_for_sight(aspect, sight) }
    fn collect_camera_previews(&self, aspect: f32, out: &mut Vec<Mat4>) -> u8 {
        self.origin.collect_camera_previews(aspect, out)
    }
    fn emit_intercom_bust(&self, aspect: f32, target: &mut dyn MeshTarget) -> Option<Mat4> {
        self.origin.emit_intercom_bust(aspect, target)
    }
    fn emit_inspection_stage(&self, aspect: f32, target: &mut dyn MeshTarget) -> Option<(Mat4, Vec3)> {
        self.origin.emit_inspection_stage(aspect, target)
    }
}

impl AvatarPresence for BlackBoxAvatar {
    fn emit_appearance(&self, target: &mut dyn MeshTarget) { self.origin.emit_appearance(target); }
    fn emit_appearance_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) { self.origin.emit_appearance_for_sight(sight, target); }
    fn emit_figure(&self, target: &mut dyn MeshTarget) { self.origin.emit_figure(target); }
    fn emit_figure_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) { self.origin.emit_figure_for_sight(sight, target); }
    fn present_vessel(&self, sink: &mut dyn world::VesselSink) { self.origin.present_vessel(sink); }
    fn planned_mount(&self, world: &dyn PlanetaryWorld) -> Option<world::PlannedMount> { self.origin.planned_mount(world) }
    fn sector_stamp(&self) -> (u8, i32, i32, i32) { self.origin.sector_stamp() }
    fn visible_slices(&self, world: &dyn PlanetaryWorld) -> u32 { self.origin.visible_slices(world) }
    fn drain_meadow(&self, world: &dyn PlanetaryWorld, ring: &mut world::MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.origin.drain_meadow(world, ring, target)
    }
    fn sync_world_presence(&self, world: &mut dyn PlanetaryWorld) { self.origin.sync_world_presence(world); }
    fn report_presence(&self, sink: &mut dyn PresenceSink) { self.origin.report_presence(sink); }
}

impl PhotoPort for BlackBoxAvatar {
    fn take_photo_shutter(&mut self) -> PhotoShutter { self.origin.take_photo_shutter() }
    fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) { self.origin.apply_polaroid_mosaic(mosaic); }
    fn trigger_log_sheet(&mut self, ticks: u16, anomalies: u8) { self.origin.trigger_log_sheet(ticks, anomalies); }
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { self.origin.polaroid_tilt() }
}

impl BlackBoxDeck for BlackBoxAvatar {
    fn black_box_active(&self) -> bool { self.tape.active() }
    fn black_box_populated(&self) -> bool { self.tape.populated() }
    fn black_box_summary(&self) -> (usize, u8) { self.tape.summary_counts() }

    fn toggle_black_box(&mut self) -> bool {
        self.tape = self.tape.toggled();
        self.tape.active()
    }

    fn export_incident_dossier(&mut self, medium: &mut dyn IncidentMedium) -> Option<u8> {
        let (ticks, _) = self.tape.summary_counts();
        let anomalies = self.tape.inscribe_dossier(medium)?;
        self.origin.trigger_log_sheet(ticks.min(u16::MAX as usize) as u16, anomalies);
        Some(anomalies)
    }
}

impl AvatarMotor for BlackBoxAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) { self.origin.rotate_view(dx, dy); }

    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        let acted = self.origin.act_hand(side, world);
        if acted {
            let ev = if side > 0.0 { FlightEvent::RightHandUsed } else { FlightEvent::LeftHandUsed };
            self.tape.record_event(ev);
        }
        acted
    }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let acted = self.origin.release_hand(world);
        if acted {
            self.tape.record_event(FlightEvent::HandReleased);
        }
        acted
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.origin.step_digging(world) }

    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        let acted = self.origin.adjust_aimed_rig(trigger, world);
        if acted {
            self.tape.record_event(FlightEvent::RigAdjusted);
        }
        acted
    }

    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        let acted = self.origin.cast_triggered_relic(trigger, world);
        if acted {
            self.tape.record_event(FlightEvent::RelicCast);
        }
        acted
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        if !self.tape.active() {
            self.origin.step_motion(dt, controls, world);
            return;
        }
        let blocks = self.calibrate_scale(world);
        let mut order = OrderSample::idle();
        controls.drive_locomotion(Vec3::Z, Vec3::X, &mut order);
        self.origin.step_motion(dt, controls, world);
        let mut probe = PresenceProbe::at_origin();
        self.origin.report_presence(&mut probe);
        let pos = probe.position();
        let view = probe.view_direction();
        let up = world.surface_up_at(pos);
        let anomaly = Self::classify_anomaly(pos, view, up, (blocks as f32) * 0.5, world);
        let packed_flags = (order.flags & 0x0F) | (anomaly << 4);
        let trace = TickTrace::new(pos, (view, dt), (up, packed_flags), order.intent);
        self.tape.record_tick(blocks, trace);
    }

    fn hunt_fauna(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        self.origin.hunt_fauna(side, world)
    }

    fn emit_reticle_for_aspect(&self, aspect: f32, target: &mut dyn MeshTarget) {
        self.origin.emit_reticle_for_aspect(aspect, target);
    }
}

impl Avatar for BlackBoxAvatar {
    fn apply_attire(&mut self, attire: YouthAttire) { self.origin.apply_attire(attire); }
    fn apply_bindings(&mut self, bindings: ActionBindings) { self.origin.apply_bindings(bindings); }
    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool { self.origin.dispatch_trigger(trigger) }
    fn typing_text(&self) -> bool { self.origin.typing_text() }
    fn accept_typed_text(&mut self, text: &str) -> bool { self.origin.accept_typed_text(text) }
    fn blend_subtick(&mut self, alpha: f32) { self.origin.blend_subtick(alpha); }
    fn step_fauna(&mut self, dt: f32, world: &mut dyn PlanetaryWorld) { self.origin.step_fauna(dt, world); }
    fn reconcile_cosmic_frame(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.reconcile_cosmic_frame(world)
    }
    #[cfg(test)]
    fn install_blueprint_archive(&mut self, archive: Box<dyn crate::blueprint::BlueprintArchive>) {
        self.origin.install_blueprint_archive(archive);
    }

    fn land_on_world(&mut self, world: &dyn PlanetaryWorld) {
        self.world_blocks = 0;
        let blocks = self.calibrate_scale(world);
        self.tape.record_event(FlightEvent::WorldLanded { blocks });
        self.origin.land_on_world(world);
    }

    fn solar_chart_snapshot(&self, world: &dyn PlanetaryWorld) -> (crate::drive::OrrerySnapshot, f32, bool) {
        self.origin.solar_chart_snapshot(world)
    }

    fn install_navigation_course(&mut self, course: crate::drive::NavigationCourse) {
        self.origin.install_navigation_course(course);
    }
}
