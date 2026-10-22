use std::fs;
use winit::keyboard::KeyCode;
use super::bindings::{ActionBindings, ControlAction, InputTrigger, TriggerNotation};

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

    fn trigger_in_text(&self, content: &str, key_name: &str) -> Option<InputTrigger> {
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('['))
            .find_map(|line| {
                let (key, val) = line.split_once('=')?;
                if key.trim() == key_name {
                    InputTrigger::Key(KeyCode::KeyW).parse_token(val.split('#').next()?)
                } else {
                    None
                }
            })
    }
}

impl BindingProfile for FileBindings {
    fn load_or_seeded(&self) -> ActionBindings {
        let Ok(content) = fs::read_to_string(&self.path) else {
            return ActionBindings::seeded();
        };
        let mut bindings = ActionBindings::seeded();
        for action in ControlAction::all() {
            if let Some(trigger) = self.trigger_in_text(&content, action.config_key()) {
                bindings = bindings.with_binding(action, trigger);
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
