use glam::Vec3;
use world::MeshTarget;
use crate::controls::{ActionBindings, ControlAction};
use super::glyph::PixelStamp;
use super::widget::ScreenBounds;

/// Polymorphic visibility state of the toggleable HUD controls slate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlsSlate {
    Hidden,
    Shown,
}

impl ControlsSlate {
    pub const fn toggled(self) -> Self {
        match self {
            Self::Hidden => Self::Shown,
            Self::Shown => Self::Hidden,
        }
    }

    pub const fn visible(self) -> bool {
        matches!(self, Self::Shown)
    }
}

/// Polymorphic gameplay context selecting which control rows appear on `ControlsSlateHud`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlateContext {
    Pedestrian,
    Wheeled,
    Spacecraft,
}

/// Toggleable on-screen HUD card listing live bound controls for the active locomotion mode.
#[derive(Clone, Copy, Debug)]
pub struct ControlsSlateHud<'a> {
    bindings: &'a ActionBindings,
    context: SlateContext,
    inv_aspect: f32,
}

impl<'a> ControlsSlateHud<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_parts(bindings: &'a ActionBindings, context: SlateContext, inv_aspect: f32) -> Self {
        Self { bindings, context, inv_aspect }
    }

    pub fn new(bindings: &'a ActionBindings, context: SlateContext, aspect: f32) -> Self {
        Self::from_parts(bindings, context, (1.0 / aspect.max(0.5)).clamp(0.25, 1.5))
    }

    pub fn emit(self, target: &mut dyn MeshTarget) {
        let b = self.bindings;
        let title = match self.context {
            SlateContext::Spacecraft => "ROCKET CONTROLS",
            SlateContext::Wheeled => "VEHICLE CONTROLS",
            SlateContext::Pedestrian => "CONTROLS",
        };
        let rows: Vec<(&str, String)> = match self.context {
            SlateContext::Spacecraft => vec![
                ("THRUST / BRAKE", format!("{} / {}", b.label_for(ControlAction::Jump), b.label_for(ControlAction::Sneak))),
                ("PITCH", format!("{} / {}", b.label_for(ControlAction::MoveForward), b.label_for(ControlAction::MoveBackward))),
                ("YAW", format!("{} / {}", b.label_for(ControlAction::MoveLeft), b.label_for(ControlAction::MoveRight))),
                ("ROLL", format!("{} / {}", b.label_for(ControlAction::RollLeft), b.label_for(ControlAction::RollRight))),
                ("RCS STRAFE", format!("{} + WASD", b.label_for(ControlAction::RcsTranslate))),
                ("FLIGHT COMP.", b.label_for(ControlAction::FlightGuidance).to_string()),
                ("FLIP + BURN", b.label_for(ControlAction::RetroFlip).to_string()),
                ("MARK TARGET", "L-CLICK (NOSE)".to_string()),
                ("CAMERA", format!("{} / {}", b.label_for(ControlAction::MoldLeft), b.label_for(ControlAction::MoldRight))),
            ],
            SlateContext::Wheeled => vec![
                ("DRIVE / REV", format!("{} / {}", b.label_for(ControlAction::MoveForward), b.label_for(ControlAction::MoveBackward))),
                ("STEER", format!("{} / {}", b.label_for(ControlAction::MoveLeft), b.label_for(ControlAction::MoveRight))),
                ("BRAKE", b.label_for(ControlAction::Jump).to_string()),
                ("CAMERA", format!("{} / {}", b.label_for(ControlAction::MoldLeft), b.label_for(ControlAction::MoldRight))),
            ],
            SlateContext::Pedestrian => vec![
                ("MOVE", format!("{}{}{}{}", b.label_for(ControlAction::MoveForward), b.label_for(ControlAction::MoveLeft), b.label_for(ControlAction::MoveBackward), b.label_for(ControlAction::MoveRight))),
                ("JUMP / SNEAK", format!("{} / {}", b.label_for(ControlAction::Jump), b.label_for(ControlAction::Sneak))),
                ("SPRINT", format!("{} + {}", b.label_for(ControlAction::Sprint), b.label_for(ControlAction::MoveForward))),
                ("BAG / BOOK", format!("{} / {}", b.label_for(ControlAction::Satchel), b.label_for(ControlAction::CraftingBook))),
                ("EMPTY HAND", format!("{} / {}", b.label_for(ControlAction::SatchelLeft), b.label_for(ControlAction::SatchelRight))),
                ("TURN BLOCK", b.label_for(ControlAction::MoldForm).to_string()),
                ("CYCLE SHAPE", format!("{} / {}", b.label_for(ControlAction::MoldLeft), b.label_for(ControlAction::MoldRight))),
            ],
        };

        let ux = self.inv_aspect;
        let row_step = 0.068;
        let card_h = 0.145 + (rows.len() as f32) * row_step;
        let (x0, y1) = (-0.955, 0.78);
        let (x1, y0) = (x0 + 0.92 * ux, y1 - card_h);
        let cx = (x0 + x1) * 0.5;

        ScreenBounds::new(x0, y0, x1, y1).emit_fill(Vec3::new(0.05, 0.09, 0.14), target);
        ScreenBounds::new(x0, y0, x1, y1).emit_outline(Vec3::new(0.32, 0.74, 0.92), 0.0040 * ux, 0.0040, target);
        PixelStamp::new(0.0056 * ux, 0.0080, Vec3::new(0.98, 0.84, 0.30)).emit_centered(title, cx, y1 - 0.042, target);

        for (idx, (label, keys)) in rows.iter().enumerate() {
            let ry = y1 - 0.100 - (idx as f32) * row_step;
            ScreenBounds::new(x0 + 0.45 * ux, ry - 0.023, x1 - 0.025 * ux, ry + 0.023)
                .emit_framed(Vec3::new(0.11, 0.18, 0.27), Vec3::new(0.24, 0.38, 0.52), 0.0018, target);
            PixelStamp::new(0.0044 * ux, 0.0066, Vec3::new(0.76, 0.86, 0.95))
                .emit_centered(label, x0 + 0.225 * ux, ry, target);
            PixelStamp::new(0.0044 * ux, 0.0066, Vec3::new(0.52, 0.96, 1.00))
                .emit_centered(keys, x0 + 0.672 * ux, ry, target);
        }

        let hide_hint = format!("[{}] HIDE", b.label_for(ControlAction::ControlsLegend));
        PixelStamp::new(0.0042 * ux, 0.0062, Vec3::new(0.54, 0.68, 0.80))
            .emit_centered(&hide_hint, cx, y0 + 0.028, target);
    }
}
