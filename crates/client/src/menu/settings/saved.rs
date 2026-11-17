use world::MeshTarget;
use crate::controls::{ActionBindings, BindingProfile, FileBindings, InputTrigger};
use crate::menu::widget::ScreenPoint;
use super::{BindingSink, ControlPanel, ControlStudio, SettingsDirective};

/// Decorator over `BindingSink` persisting every binding transition into a `BindingProfile`.
struct PersistedBindingSink<'a> {
    inner: &'a mut dyn BindingSink,
    profile: &'a dyn BindingProfile,
}

impl<'a> PersistedBindingSink<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(inner: &'a mut dyn BindingSink, profile: &'a dyn BindingProfile) -> Self {
        Self { inner, profile }
    }
}

impl BindingSink for PersistedBindingSink<'_> {
    fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.profile.save(&bindings);
        self.inner.apply_bindings(bindings);
    }
}

/// Decorator over `ControlPanel` persisting keybinding updates to disk without coupling UI to I/O.
pub struct SavedControls {
    inner: Box<dyn ControlPanel>,
    profile: Box<dyn BindingProfile>,
}

impl SavedControls {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn ControlPanel>, profile: Box<dyn BindingProfile>) -> Self {
        Self { inner, profile }
    }

    /// Secondary constructor loading and persisting the player's local `controls.toml` profile.
    pub fn live() -> Self {
        let store = FileBindings::profile();
        let studio = ControlStudio::from_profile(&store);
        Self::new(Box::new(studio), Box::new(store))
    }
}

impl ControlPanel for SavedControls {
    fn move_pointer(&mut self, next: ScreenPoint) {
        self.inner.move_pointer(next);
    }

    fn cancel_focus(&mut self) {
        self.inner.cancel_focus();
    }

    fn press_pointer(&mut self, sink: &mut dyn BindingSink) -> SettingsDirective {
        let mut persisted = PersistedBindingSink::new(sink, &*self.profile);
        self.inner.press_pointer(&mut persisted)
    }

    fn handle_trigger(&mut self, trigger: InputTrigger, sink: &mut dyn BindingSink) -> bool {
        let mut persisted = PersistedBindingSink::new(sink, &*self.profile);
        self.inner.handle_trigger(trigger, &mut persisted)
    }

    fn sync_controls(&self, sink: &mut dyn BindingSink) {
        self.inner.sync_controls(sink);
    }

    fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        self.inner.emit_overlay(target);
    }
}
