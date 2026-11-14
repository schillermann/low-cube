mod bearing_gear;
mod body;
#[cfg(test)]
mod gesture;
mod planetary;
mod sight;

use glam::{Mat4, Quat, Vec3};
#[cfg(test)]
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{PhotoShutter, SightMode, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::menu::ScreenPoint;

#[cfg(test)]
pub use self::gesture::HandGesture;
pub use self::planetary::PlanetaryAvatar;

/// Interface for player avatars acting within planetary gravity.
pub trait Avatar {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool;
    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool;
    fn release_hand(&mut self, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn step_digging(&mut self, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn apply_attire(&mut self, attire: YouthAttire);
    fn apply_bindings(&mut self, _bindings: ActionBindings) {}
    fn dispatch_trigger(&mut self, _trigger: InputTrigger) -> bool { false }
    fn adjust_aimed_rig(&mut self, _trigger: InputTrigger, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn cast_triggered_relic(&mut self, _trigger: InputTrigger, _world: &mut dyn PlanetaryWorld) -> bool { false }
    #[cfg(test)]
    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool { self.dispatch_trigger(InputTrigger::Key(code)) }
    #[cfg(test)]
    fn press_world_key(&mut self, code: KeyCode, world: &mut dyn PlanetaryWorld) -> bool {
        let trigger = InputTrigger::Key(code);
        self.cast_triggered_relic(trigger, world) || self.dispatch_trigger(trigger)
    }
    #[cfg(test)]
    fn rotate_degrees(&mut self, yaw_deg: f32, pitch_deg: f32) {
        let px_per_deg = std::f32::consts::PI / (180.0 * 0.003);
        self.rotate_view(yaw_deg * px_per_deg, pitch_deg * px_per_deg);
    }
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn step_fauna(&self, _dt: f32, _fauna: &mut crate::roe_deer::WoodlandFauna, _world: &dyn PlanetaryWorld) {}
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
    fn view_projection_for_sight(&self, aspect: f32, _sight: SightMode) -> Mat4 { self.view_projection_matrix(aspect) }
    fn surface_rotation(&self) -> Quat;
    fn emit_appearance(&self, target: &mut dyn MeshTarget);
    fn emit_appearance_for_sight(&self, _sight: SightMode, target: &mut dyn MeshTarget) { self.emit_appearance(target); }
    fn emit_surroundings(&self, _world: &dyn PlanetaryWorld, _target: &mut dyn MeshTarget) {}
    fn meadow_anchor(&self) -> (i32, i32, i32) { (0, 0, 0) }
    fn take_photo_shutter(&mut self) -> PhotoShutter { PhotoShutter::Idle }
    fn apply_polaroid_mosaic(&mut self, _mosaic: Vec<Vec3>) {}
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { 0.0 }
    fn emit_reticle(&self, target: &mut dyn MeshTarget) { ScreenPoint::center().emit_crosshair(target); }
    fn sync_world_presence(&self, _world: &mut dyn PlanetaryWorld) {}
    fn land_on_world(&mut self, _world: &dyn PlanetaryWorld) {}
}
