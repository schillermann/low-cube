mod saved;

use glam::Vec3;
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{
    ActionBindings, BindingProfile, ControlAction, InputTrigger, KeyboardControls,
};
use super::glyph::PixelStamp;
use super::view::MenuView;
use super::widget::{ScreenBounds, ScreenPoint};

pub use saved::SavedControls;

/// Outcome returned from interactive keybindings panel actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsDirective {
    StayInSettings,
    ReturnToSettings,
}

/// Polymorphic state for whether the settings studio is idle or awaiting a replacement input trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebindFocus {
    Idle,
    Awaiting(ControlAction),
}

/// Narrow Tell-Don't-Ask recipient for updated action bindings.
pub trait BindingSink {
    fn apply_bindings(&mut self, bindings: ActionBindings);
}

impl BindingSink for &mut dyn Avatar {
    fn apply_bindings(&mut self, bindings: ActionBindings) {
        Avatar::apply_bindings(*self, bindings);
    }
}

impl BindingSink for &mut dyn KeyboardControls {
    fn apply_bindings(&mut self, bindings: ActionBindings) {
        KeyboardControls::apply_bindings(*self, bindings);
    }
}

/// Focused interface for interactive keybinding studios and persistence decorators.
pub trait ControlPanel {
    fn move_pointer(&mut self, next: ScreenPoint);
    fn cancel_focus(&mut self);
    fn press_pointer(&mut self, sink: &mut dyn BindingSink) -> SettingsDirective;
    fn handle_trigger(&mut self, trigger: InputTrigger, sink: &mut dyn BindingSink) -> bool;
    fn sync_controls(&self, sink: &mut dyn BindingSink);
    fn emit_overlay(&self, target: &mut dyn MeshTarget);
}

impl MenuView for &dyn ControlPanel {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.emit_overlay(target);
    }
}

/// Stateless screen bounds layout for the 19-slot Keybindings studio.
pub struct ControlLayout;

impl ControlLayout {
    /// Primary constructor initializing the keybindings layout.
    pub const fn new() -> Self {
        Self
    }

    pub fn action_bounds(&self, action: ControlAction) -> ScreenBounds {
        let idx = action.slot_index();
        let (x0, x1, y1) = if idx < 7 {
            (-0.94, -0.37, 0.67 - (idx as f32) * 0.16)
        } else if idx < 14 {
            (-0.28, 0.28, 0.67 - ((idx - 7) as f32) * 0.16)
        } else {
            (0.37, 0.94, 0.67 - ((idx - 14) as f32) * 0.16)
        };
        ScreenBounds::new(x0, y1 - 0.11, x1, y1)
    }

    pub const fn reset_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.40, -0.95, -0.04, -0.84)
    }

    pub const fn back_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.04, -0.95, 0.40, -0.84)
    }
}

/// Pure in-memory keybinding configuration studio enforcing strictly one trigger per action.
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

    #[cfg(test)]
    pub const fn seeded() -> Self {
        Self::new(ActionBindings::seeded(), RebindFocus::Idle, ScreenPoint::center())
    }

    pub fn from_profile(profile: &dyn BindingProfile) -> Self {
        Self::new(profile.load_or_seeded(), RebindFocus::Idle, ScreenPoint::center())
    }
}

impl MenuView for ControlStudio {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let layout = ControlLayout::new();
        let (panel_bg, panel_rim, btn_col, gold) = (
            Vec3::new(0.08, 0.10, 0.14),
            Vec3::new(0.28, 0.34, 0.44),
            Vec3::new(0.16, 0.21, 0.29),
            Vec3::new(0.95, 0.80, 0.32),
        );
        ScreenBounds::new(-0.97, 0.84, 0.97, 0.96).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::title(gold).emit_centered("KEYBINDINGS", 0.0, 0.90, target);

        ScreenBounds::new(-0.97, -0.81, -0.34, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("MOVEMENT", -0.655, 0.75, target);

        ScreenBounds::new(-0.31, -0.81, 0.31, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("SATCHEL + MOLD", 0.0, 0.75, target);

        ScreenBounds::new(0.34, -0.81, 0.97, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("SPACEFLIGHT", 0.655, 0.75, target);

        for action in ControlAction::all() {
            let awaiting = self.focus == RebindFocus::Awaiting(action);
            let label = if awaiting {
                format!("{}: PRESS KEY", action.title())
            } else {
                format!("{}: {}", action.title(), self.bindings.label_for(action))
            };
            layout.action_bounds(action).emit_button(&label, self.pointer, btn_col, awaiting, target);
        }

        layout.reset_button().emit_button("RESET KEYS", self.pointer, btn_col, false, target);
        layout.back_button().emit_button("BACK", self.pointer, btn_col, false, target);
    }
}

impl ControlPanel for ControlStudio {
    fn move_pointer(&mut self, next: ScreenPoint) {
        self.pointer = next;
    }

    fn cancel_focus(&mut self) {
        self.focus = RebindFocus::Idle;
    }

    fn press_pointer(&mut self, sink: &mut dyn BindingSink) -> SettingsDirective {
        let layout = ControlLayout::new();
        let pt = self.pointer;
        for action in ControlAction::all() {
            if layout.action_bounds(action).contains(pt) {
                self.focus = RebindFocus::Awaiting(action);
                return SettingsDirective::StayInSettings;
            }
        }
        if layout.reset_button().contains(pt) {
            self.focus = RebindFocus::Idle;
            self.bindings = ActionBindings::seeded();
            sink.apply_bindings(self.bindings);
            return SettingsDirective::StayInSettings;
        }
        if layout.back_button().contains(pt) {
            self.focus = RebindFocus::Idle;
            return SettingsDirective::ReturnToSettings;
        }
        self.focus = RebindFocus::Idle;
        SettingsDirective::StayInSettings
    }

    fn handle_trigger(&mut self, trigger: InputTrigger, sink: &mut dyn BindingSink) -> bool {
        if let RebindFocus::Awaiting(action) = self.focus {
            self.focus = RebindFocus::Idle;
            self.bindings = self.bindings.with_binding(action, trigger);
            sink.apply_bindings(self.bindings);
            return true;
        }
        false
    }

    fn sync_controls(&self, sink: &mut dyn BindingSink) {
        sink.apply_bindings(self.bindings);
    }

    fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        self.emit(target);
    }
}
