use glam::{Mat4, Vec3};
use world::{MeshTarget, PlanetaryWorld, SightCone};
use crate::attire::{EvaAttire, PhotoShutter, SightMode};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::flight::GravitationalFlight;
#[cfg(test)]
use crate::flight::SurfaceFlight;
use super::embodied::EmbodiedAvatar;
use super::mortal_motor::MortalMotor;
use super::rain_ear::RainEarMotor;
use super::satchel_guard::SatchelAvatar;
use super::{Avatar, AvatarMotor, AvatarOptics, AvatarPresence, BlackBoxDeck, PhotoPort};

/// Player avatar composing rain-acoustic, mortality-, and satchel-guarded `AvatarMotor` layers over `EmbodiedAvatar`.
pub struct PlanetaryAvatar {
    motor: RainEarMotor<MortalMotor<SatchelAvatar<EmbodiedAvatar>>>,
}

impl PlanetaryAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_motor(motor: RainEarMotor<MortalMotor<SatchelAvatar<EmbodiedAvatar>>>) -> Self {
        Self { motor }
    }

    pub fn with_attire_and_bindings(
        initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: EvaAttire, bindings: ActionBindings,
    ) -> Self {
        Self::from_motor(RainEarMotor::new(MortalMotor::new(SatchelAvatar::new(
            EmbodiedAvatar::with_attire_and_bindings(initial_position, flight, attire, bindings),
        ))))
    }

    pub fn with_starting_satchel(mut self, provider: &dyn crate::server_satchel::StartingSatchel) -> Self {
        self.core_mut().provision_satchel(provider);
        self
    }

    #[cfg(test)]
    pub fn with_test_engines(self) -> Self {
        self.with_starting_satchel(&crate::server_satchel::EvaKitSatchel::new())
    }
    #[cfg(test)]
    pub fn with_attire(initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: EvaAttire) -> Self {
        Self::with_attire_and_bindings(initial_position, flight, attire, ActionBindings::seeded())
    }
    #[cfg(test)]
    pub fn with_flight(initial_position: Vec3, flight: Box<dyn GravitationalFlight>) -> Self {
        Self::with_attire(initial_position, flight, EvaAttire::astronaut())
    }
    #[cfg(test)]
    pub fn at(initial_position: Vec3) -> Self {
        Self::with_flight(initial_position, Box::new(SurfaceFlight::terrestrial(1.0)))
    }

    const fn core(&self) -> &EmbodiedAvatar {
        self.motor.inner().inner().inner()
    }

    fn core_mut(&mut self) -> &mut EmbodiedAvatar {
        self.motor.inner_mut().inner_mut().inner_mut()
    }
}

impl AvatarOptics for PlanetaryAvatar {
    fn sight_cone(&self) -> SightCone { self.core().sight_cone() }
    fn sight_cone_for_mode(&self, sight: SightMode) -> SightCone { self.core().sight_cone_for_mode(sight) }
    fn view_projection_matrix(&self, aspect: f32) -> Mat4 { self.core().view_projection_matrix(aspect) }
    fn view_projection_for_sight(&self, aspect: f32, sight: SightMode) -> Mat4 { self.core().view_projection_for_sight(aspect, sight) }
    fn collect_camera_previews(&self, aspect: f32, out: &mut Vec<Mat4>) -> u8 {
        self.core().collect_camera_previews(aspect, out)
    }
    fn emit_intercom_bust(&self, aspect: f32, target: &mut dyn MeshTarget) -> Option<Mat4> {
        self.core().emit_intercom_bust(aspect, target)
    }
    fn emit_inspection_stage(&self, aspect: f32, target: &mut dyn MeshTarget) -> Option<(Mat4, Vec3)> {
        self.core().emit_inspection_stage(aspect, target)
    }
}

impl AvatarPresence for PlanetaryAvatar {
    fn emit_appearance(&self, target: &mut dyn MeshTarget) { self.core().emit_appearance(target); }
    fn emit_appearance_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) { self.core().emit_appearance_for_sight(sight, target); }
    fn emit_figure(&self, target: &mut dyn MeshTarget) { self.core().emit_figure(target); }
    fn emit_figure_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) { self.core().emit_figure_for_sight(sight, target); }
    fn present_vessel(&self, sink: &mut dyn world::VesselSink) { self.core().present_vessel(sink); }
    fn planned_mount(&self, world: &dyn PlanetaryWorld) -> Option<world::PlannedMount> { self.core().planned_mount(world) }
    fn sector_stamp(&self) -> (u8, i32, i32, i32) { self.core().sector_stamp() }
    fn visible_slices(&self, world: &dyn PlanetaryWorld) -> u32 { self.core().visible_slices(world) }
    fn drain_meadow(&self, world: &dyn PlanetaryWorld, ring: &mut world::MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.core().drain_meadow(world, ring, target)
    }
    fn sync_world_presence(&self, world: &mut dyn PlanetaryWorld) { self.core().sync_world_presence(world); }
    fn report_presence(&self, sink: &mut dyn crate::black_box::PresenceSink) { self.core().report_presence(sink); }
}

impl PhotoPort for PlanetaryAvatar {
    fn take_photo_shutter(&mut self) -> PhotoShutter { self.core_mut().take_photo_shutter() }
    fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) { self.core_mut().apply_polaroid_mosaic(mosaic); }
    fn trigger_log_sheet(&mut self, ticks: u16, anomalies: u8) { self.core_mut().trigger_log_sheet(ticks, anomalies); }
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { self.core().polaroid_tilt() }
}

impl BlackBoxDeck for PlanetaryAvatar {}

impl AvatarMotor for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) { self.motor.rotate_view(dx, dy); }
    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool { self.motor.act_hand(side, world) }
    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.motor.release_hand(world) }
    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.motor.step_digging(world) }
    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool { self.motor.adjust_aimed_rig(trigger, world) }
    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool { self.motor.cast_triggered_relic(trigger, world) }
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) { self.motor.step_motion(dt, controls, world); }
    fn emit_reticle_for_aspect(&self, aspect: f32, target: &mut dyn MeshTarget) {
        self.motor.emit_reticle_for_aspect(aspect, target);
    }
}

impl Avatar for PlanetaryAvatar {
    fn apply_attire(&mut self, attire: EvaAttire) { self.core_mut().apply_attire(attire); }
    fn apply_bindings(&mut self, bindings: ActionBindings) { self.core_mut().apply_bindings(bindings); }
    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool { self.core_mut().dispatch_trigger(trigger) }
    fn typing_text(&self) -> bool { self.core().typing_text() }
    fn accept_typed_text(&mut self, text: &str) -> bool { self.core_mut().accept_typed_text(text) }
    fn blend_subtick(&mut self, alpha: f32) { self.core_mut().blend_subtick(alpha); }
    fn reconcile_cosmic_frame(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.motor.inner_mut().reconcile_cosmic_frame(world)
    }
    fn land_on_world(&mut self, world: &dyn PlanetaryWorld) { self.core_mut().land_on_world(world); }
    fn flush_world_notes(&mut self, world: &mut dyn PlanetaryWorld) { self.core_mut().flush_world_notes(world); }
    fn solar_chart_snapshot(&self, world: &dyn PlanetaryWorld) -> (crate::drive::OrrerySnapshot, f32, bool) {
        self.core().solar_chart_snapshot(world)
    }
    fn install_navigation_course(&mut self, course: crate::drive::NavigationCourse) {
        self.core_mut().install_navigation_course(course);
    }
    #[cfg(test)]
    fn install_blueprint_archive(&mut self, archive: Box<dyn crate::blueprint::BlueprintArchive>) {
        self.core_mut().install_blueprint_archive(archive);
    }
}
