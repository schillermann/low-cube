use glam::Mat4;
use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::attire::{AttireProfile, EvaAttire, FileAttire};
use crate::menu::widget::ScreenPoint;
use super::{AttireSink, StudioDirective, WardrobePanel, WardrobeStudio};

/// Decorator over `AttireSink` persisting every wardrobe customization into an `AttireProfile`.
struct PersistedAttireSink<'a> {
    inner: &'a mut dyn AttireSink,
    profile: &'a dyn AttireProfile,
}

impl<'a> PersistedAttireSink<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(inner: &'a mut dyn AttireSink, profile: &'a dyn AttireProfile) -> Self {
        Self { inner, profile }
    }
}

impl AttireSink for PersistedAttireSink<'_> {
    fn apply_attire(&mut self, attire: EvaAttire) {
        self.profile.save(&attire);
        self.inner.apply_attire(attire);
    }
}

/// Decorator over `WardrobePanel` persisting character customization to disk without coupling UI to I/O.
pub struct SavedWardrobe {
    inner: Box<dyn WardrobePanel>,
    profile: Box<dyn AttireProfile>,
}

impl SavedWardrobe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn WardrobePanel>, profile: Box<dyn AttireProfile>) -> Self {
        Self { inner, profile }
    }

    /// Secondary constructor loading and persisting the player's local `character.toml` profile.
    pub fn live() -> Self {
        let store = FileAttire::profile();
        let studio = WardrobeStudio::from_profile(&store);
        Self::new(Box::new(studio), Box::new(store))
    }
}

impl WardrobePanel for SavedWardrobe {
    fn step(&mut self, dt: f32) {
        self.inner.step(dt);
    }

    fn stage_projection(&self, aspect: f32) -> Mat4 {
        self.inner.stage_projection(aspect)
    }

    fn emit_stage(&self, target: &mut dyn MeshTarget) {
        self.inner.emit_stage(target);
    }

    fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        self.inner.emit_overlay(target);
    }

    fn release_grip(&mut self) {
        self.inner.release_grip();
    }

    fn move_pointer(&mut self, next: ScreenPoint) {
        self.inner.move_pointer(next);
    }

    fn handle_key(&mut self, code: KeyCode, sink: &mut dyn AttireSink) -> bool {
        let mut persisted = PersistedAttireSink::new(sink, &*self.profile);
        self.inner.handle_key(code, &mut persisted)
    }

    fn press_pointer(&mut self, sink: &mut dyn AttireSink) -> StudioDirective {
        let mut persisted = PersistedAttireSink::new(sink, &*self.profile);
        self.inner.press_pointer(&mut persisted)
    }
}
