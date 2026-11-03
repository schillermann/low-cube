use std::fs;
use crate::toml_document::TomlDocument;
use super::bindings::{ActionBindings, ControlAction, TriggerNotation};

/// Interface for persistent TOML storage of player keybindings.
pub trait BindingProfile {
    fn load_or_seeded(&self) -> ActionBindings;
    fn save(&self, bindings: &ActionBindings);
}

/// Persistent TOML storage for customizable single-trigger action bindings.
pub struct FileBindings {
    path: String,
}

impl FileBindings {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Secondary constructor targeting the local controls.toml profile.
    pub fn profile() -> Self {
        Self::new("controls.toml".to_string())
    }
}

impl BindingProfile for FileBindings {
    fn load_or_seeded(&self) -> ActionBindings {
        let Some(doc) = TomlDocument::from_file(&self.path) else {
            return ActionBindings::seeded();
        };
        let mut bindings = ActionBindings::seeded();
        for action in ControlAction::all() {
            if let Some(token) = doc.property(action.config_key()) {
                if let Some(trigger) = bindings.trigger_for(action).parse_token(&token) {
                    bindings = bindings.with_binding(action, trigger);
                }
            }
        }
        bindings
    }

    fn save(&self, bindings: &ActionBindings) {
        let mut text = String::from("# Low-Cube Single-Trigger Action Bindings\n[controls]\n");
        for action in ControlAction::all() {
            text.push_str(&format!("{} = \"{}\"\n", action.config_key(), bindings.trigger_for(action).label()));
        }
        fs::write(&self.path, text).expect("Failed to persist ActionBindings profile to disk");
    }
}
