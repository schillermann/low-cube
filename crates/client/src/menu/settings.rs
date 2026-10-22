use glam::Vec3;
use world::MeshTarget;
use crate::controls::{ActionBindings, BindingProfile, ControlAction, FileBindings, InputTrigger, TriggerNotation};
use super::glyph::PixelStamp;
use super::widget::{ScreenBounds, ScreenPoint};

/// Outcome returned from interactive settings panel actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsDirective {
    StayInSettings,
    ReturnToPauseMenu,
}

/// Polymorphic state for whether the settings studio is idle or awaiting a replacement input trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebindFocus {
    Idle,
    Awaiting(ControlAction),
}

/// Interactive keybinding configuration studio enforcing strictly one trigger per action.
#[derive(Clone, Copy, Debug)]
pub struct ControlStudio {
    bindings: ActionBindings,
    focus: RebindFocus,
    pointer: ScreenPoint,
}

impl ControlStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bindings: ActionBindings, focus: RebindFocus, pointer: ScreenPoint) -> Self {
        Self { bindings, focus, pointer }
    }

    pub fn from_profile() -> Self {
        Self::new(FileBindings::profile().load_or_seeded(), RebindFocus::Idle, ScreenPoint::center())
    }

    pub const fn current_bindings(&self) -> ActionBindings {
        self.bindings
    }

    pub fn move_pointer(&mut self, next: ScreenPoint) {
        self.pointer = next;
    }

    pub fn cancel_focus(&mut self) {
        self.focus = RebindFocus::Idle;
    }

    fn update_bindings(&mut self, updated: ActionBindings) {
        self.bindings = updated;
        FileBindings::profile().save(&self.bindings);
    }

    pub fn action_bounds(action: ControlAction) -> ScreenBounds {
        let idx = action.slot_index();
        let (x0, x1, y1) = if idx < 6 {
            (-0.92, -0.07, 0.67 - (idx as f32) * 0.16)
        } else if idx == 6 {
            (-0.92, -0.07, -0.62)
        } else if idx < 11 {
            (0.07, 0.92, 0.67 - ((idx - 7) as f32) * 0.16)
        } else {
            (0.07, 0.92, -0.14 - ((idx - 11) as f32) * 0.16)
        };
        ScreenBounds::new(x0, y1 - 0.11, x1, y1)
    }

    pub const fn reset_button() -> ScreenBounds {
        ScreenBounds::new(-0.40, -0.95, -0.04, -0.84)
    }

    pub const fn back_button() -> ScreenBounds {
        ScreenBounds::new(0.04, -0.95, 0.40, -0.84)
    }

    pub fn press_pointer(&mut self) -> (SettingsDirective, bool) {
        let pt = self.pointer;
        for action in ControlAction::all() {
            if Self::action_bounds(action).contains(pt) {
                self.focus = RebindFocus::Awaiting(action);
                return (SettingsDirective::StayInSettings, false);
            }
        }
        if Self::reset_button().contains(pt) {
            self.focus = RebindFocus::Idle;
            self.update_bindings(ActionBindings::seeded());
            return (SettingsDirective::StayInSettings, true);
        }
        if Self::back_button().contains(pt) {
            self.focus = RebindFocus::Idle;
            return (SettingsDirective::ReturnToPauseMenu, false);
        }
        self.focus = RebindFocus::Idle;
        (SettingsDirective::StayInSettings, false)
    }

    pub fn handle_trigger(&mut self, trigger: InputTrigger) -> bool {
        if let RebindFocus::Awaiting(action) = self.focus {
            self.focus = RebindFocus::Idle;
            self.update_bindings(self.bindings.with_binding(action, trigger));
            return true;
        }
        false
    }

    pub fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        let (panel_bg, panel_rim, btn_col, gold) = (
            Vec3::new(0.08, 0.10, 0.14),
            Vec3::new(0.28, 0.34, 0.44),
            Vec3::new(0.16, 0.21, 0.29),
            Vec3::new(0.95, 0.80, 0.32),
        );
        ScreenBounds::new(-0.97, 0.84, 0.97, 0.96).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::title(gold).emit_centered("KEYBINDINGS", 0.0, 0.90, target);

        ScreenBounds::new(-0.97, -0.44, -0.02, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("MOVEMENT", -0.495, 0.75, target);

        ScreenBounds::new(-0.97, -0.81, -0.02, -0.48).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("VIEW", -0.495, -0.54, target);

        ScreenBounds::new(0.02, 0.04, 0.97, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("SATCHEL", 0.495, 0.75, target);

        ScreenBounds::new(0.02, -0.81, 0.97, 0.00).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("MOLD", 0.495, -0.06, target);

        for action in ControlAction::all() {
            let awaiting = self.focus == RebindFocus::Awaiting(action);
            let label = if awaiting {
                format!("{}: PRESS KEY", action.title())
            } else {
                format!("{}: {}", action.title(), self.bindings.trigger_for(action).label())
            };
            Self::action_bounds(action).emit_button(&label, self.pointer, btn_col, awaiting, target);
        }

        Self::reset_button().emit_button("RESET KEYS", self.pointer, btn_col, false, target);
        Self::back_button().emit_button("BACK", self.pointer, btn_col, false, target);
    }
}
