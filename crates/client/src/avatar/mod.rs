mod bearing;
mod berth;
mod body;
mod condition;
mod deck;
mod embodied;
mod gear;
mod helm_optics;
#[cfg(test)]
mod gesture;
mod planetary;
mod satchel_guard;
mod sight;
mod span;

use glam::{Mat4, Quat, Vec3};
#[cfg(test)]
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{PhotoShutter, SightMode, YouthAttire};
use crate::black_box::{IncidentMedium, PresenceSink};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use fauna::{SightCone, WoodlandFauna};
use crate::menu::ScreenPoint;

#[cfg(test)]
pub use self::gesture::HandGesture;
pub use self::planetary::PlanetaryAvatar;

/// Focused camera optics and sight-cone projection contract.
pub trait AvatarOptics {
    fn sight_cone(&self) -> SightCone { SightCone::omnidirectional(Vec3::ZERO) }
    fn sight_cone_for_mode(&self, _sight: SightMode) -> SightCone { self.sight_cone() }
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
    fn view_projection_for_sight(&self, aspect: f32, _sight: SightMode) -> Mat4 {
        self.view_projection_matrix(aspect)
    }
    fn surface_rotation(&self) -> Quat;
}

/// Focused 3D appearance, meadow horizon, and presence-reporting contract.
pub trait AvatarPresence {
    fn emit_appearance(&self, target: &mut dyn MeshTarget);
    fn emit_appearance_for_sight(&self, _sight: SightMode, target: &mut dyn MeshTarget) {
        self.emit_appearance(target);
    }
    fn emit_figure(&self, target: &mut dyn MeshTarget) {
        self.emit_appearance(target);
    }
    fn emit_figure_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) {
        self.emit_appearance_for_sight(sight, target);
    }
    fn present_vessel(&self, _sink: &mut dyn world::VesselSink) {}
    fn planned_mount(&self, _world: &dyn PlanetaryWorld) -> Option<world::PlannedMount> {
        None
    }
    fn emit_placement_preview(&self, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        if let Some(planned) = self.planned_mount(world) {
            planned.emit_hologram(target);
        }
    }
    fn sector_stamp(&self) -> (u8, i32, i32, i32) { (0, 0, 0, 0) }
    fn visible_slices(&self, world: &dyn PlanetaryWorld) -> u32 { world.visible_slice_mask(Vec3::ZERO) }
    fn drain_meadow(&self, _world: &dyn PlanetaryWorld, _ring: &mut world::MeadowRing, _target: &mut dyn MeshTarget) -> Option<usize> {
        None
    }
    fn sync_world_presence(&self, _world: &mut dyn PlanetaryWorld) {}
    fn report_presence(&self, _sink: &mut dyn PresenceSink) {}
}

/// Focused diegetic camera shutter and Polaroid exposure contract.
pub trait PhotoPort {
    fn take_photo_shutter(&mut self) -> PhotoShutter { PhotoShutter::Idle }
    fn apply_polaroid_mosaic(&mut self, _mosaic: Vec<Vec3>) {}
    fn trigger_log_sheet(&mut self, _ticks: u16, _anomalies: u8) {}
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { 0.0 }
}

/// Focused flight black-box telemetry control and dossier export contract.
pub trait BlackBoxDeck {
    fn black_box_active(&self) -> bool { false }
    fn black_box_populated(&self) -> bool { false }
    fn black_box_summary(&self) -> (usize, u8) { (0, 0) }
    fn toggle_black_box(&mut self) -> bool { false }
    fn export_incident_dossier(&mut self, _medium: &mut dyn IncidentMedium) -> Option<u8> { None }
}

/// Focused active motor, hand interaction, and reticle contract decorated by guard layers.
pub trait AvatarMotor {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool;
    fn release_hand(&mut self, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn step_digging(&mut self, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn adjust_aimed_rig(&mut self, _trigger: InputTrigger, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn cast_triggered_relic(&mut self, _trigger: InputTrigger, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn hunt_fauna(&mut self, _side: f32, _fauna: &mut WoodlandFauna, _world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        (false, false)
    }
    fn emit_reticle(&self, target: &mut dyn MeshTarget) { ScreenPoint::center().emit_crosshair(target); }
}

/// Unified player avatar interface composed from focused optics, presence, photo, black-box, and motor contracts.
pub trait Avatar: AvatarMotor + AvatarOptics + AvatarPresence + PhotoPort + BlackBoxDeck {
    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.act_hand(-1.0, world)
    }
    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.act_hand(1.0, world)
    }
    fn apply_attire(&mut self, attire: YouthAttire);
    fn apply_bindings(&mut self, _bindings: ActionBindings) {}
    fn dispatch_trigger(&mut self, _trigger: InputTrigger) -> bool { false }
    fn typing_text(&self) -> bool { false }
    fn accept_typed_text(&mut self, _text: &str) -> bool { false }
    fn blend_subtick(&mut self, _alpha: f32) {}
    fn step_fauna(&mut self, _dt: f32, _fauna: &mut WoodlandFauna, _world: &dyn PlanetaryWorld) {}
    fn land_on_world(&mut self, _world: &dyn PlanetaryWorld) {}
    #[cfg(test)]
    fn install_blueprint_archive(&mut self, _archive: Box<dyn crate::blueprint::BlueprintArchive>) {}
    #[cfg(test)]
    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool {
        self.dispatch_trigger(InputTrigger::Key(code))
    }
    #[cfg(test)]
    fn press_world_key(&mut self, code: KeyCode, world: &mut dyn PlanetaryWorld) -> bool {
        let trigger = InputTrigger::Key(code);
        self.adjust_aimed_rig(trigger, world)
            || self.cast_triggered_relic(trigger, world)
            || self.dispatch_trigger(trigger)
    }
    #[cfg(test)]
    fn rotate_degrees(&mut self, yaw_deg: f32, pitch_deg: f32) {
        let px_per_deg = std::f32::consts::PI / (180.0 * 0.003);
        self.rotate_view(yaw_deg * px_per_deg, pitch_deg * px_per_deg);
    }
}
