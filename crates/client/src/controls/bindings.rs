use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Distinct gameplay actions each bound to a single input trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlAction {
    MoveForward, MoveBackward, MoveLeft, MoveRight,
    Jump, Sneak, Sprint,
    Satchel, CraftingBook, SatchelLeft, SatchelRight,
    MoldForm, MoldLeft, MoldRight,
    FlightGuidance, RollLeft, RollRight, RcsTranslate, RetroFlip, ControlsLegend, SystemMap,
}

impl ControlAction {
    pub const fn all() -> [Self; 21] {
        [
            Self::MoveForward, Self::MoveBackward, Self::MoveLeft, Self::MoveRight,
            Self::Jump, Self::Sneak, Self::Sprint,
            Self::Satchel, Self::CraftingBook, Self::SatchelLeft, Self::SatchelRight,
            Self::MoldForm, Self::MoldLeft, Self::MoldRight,
            Self::FlightGuidance, Self::RollLeft, Self::RollRight,
            Self::RcsTranslate, Self::RetroFlip, Self::ControlsLegend, Self::SystemMap,
        ]
    }

    pub const fn slot_index(self) -> usize {
        match self {
            Self::MoveForward => 0, Self::MoveBackward => 1, Self::MoveLeft => 2, Self::MoveRight => 3,
            Self::Jump => 4, Self::Sneak => 5, Self::Sprint => 6,
            Self::Satchel => 7, Self::CraftingBook => 8, Self::SatchelLeft => 9, Self::SatchelRight => 10,
            Self::MoldForm => 11, Self::MoldLeft => 12, Self::MoldRight => 13,
            Self::FlightGuidance => 14, Self::RollLeft => 15, Self::RollRight => 16,
            Self::RcsTranslate => 17, Self::RetroFlip => 18, Self::ControlsLegend => 19,
            Self::SystemMap => 20,
        }
    }

    pub const fn spaceflight_domain(self) -> bool {
        self.slot_index() >= 14
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::MoveForward => "MOVE FORWARD", Self::MoveBackward => "MOVE BACK",
            Self::MoveLeft => "MOVE LEFT", Self::MoveRight => "MOVE RIGHT",
            Self::Jump => "JUMP", Self::Sneak => "SNEAK", Self::Sprint => "SPRINT",
            Self::Satchel => "SATCHEL", Self::CraftingBook => "CRAFTING BOOK",
            Self::SatchelLeft => "EMPTY LEFT", Self::SatchelRight => "EMPTY RIGHT",
            Self::MoldForm => "TURN BLOCK", Self::MoldLeft => "PREV SHAPE", Self::MoldRight => "NEXT SHAPE",
            Self::FlightGuidance => "FLIGHT COMPUTER", Self::RollLeft => "ROLL LEFT",
            Self::RollRight => "ROLL RIGHT", Self::RcsTranslate => "RCS STRAFE",
            Self::RetroFlip => "RETRO FLIP", Self::ControlsLegend => "CONTROLS HUD",
            Self::SystemMap => "SYSTEM MAP",
        }
    }

    pub const fn config_key(self) -> &'static str {
        match self {
            Self::MoveForward => "move_forward", Self::MoveBackward => "move_backward",
            Self::MoveLeft => "move_left", Self::MoveRight => "move_right",
            Self::Jump => "jump", Self::Sneak => "sneak", Self::Sprint => "sprint",
            Self::Satchel => "satchel", Self::CraftingBook => "crafting_book",
            Self::SatchelLeft => "satchel_left", Self::SatchelRight => "satchel_right",
            Self::MoldForm => "mold_form", Self::MoldLeft => "mold_left", Self::MoldRight => "mold_right",
            Self::FlightGuidance => "flight_guidance", Self::RollLeft => "roll_left",
            Self::RollRight => "roll_right", Self::RcsTranslate => "rcs_translate",
            Self::RetroFlip => "retro_flip", Self::ControlsLegend => "controls_legend",
            Self::SystemMap => "system_map",
        }
    }
}

/// Physical keyboard key or mouse wheel action bound to a gameplay ControlAction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputTrigger {
    Key(KeyCode),
    WheelClick,
    WheelUp,
    WheelDown,
}

/// Interface for formatting and parsing input triggers as compact uppercase tokens.
trait TriggerNotation {
    fn label(self) -> &'static str;
    fn parse_token(&self, token: &str) -> Option<InputTrigger>;
}

const KEY_TABLE: [(KeyCode, &str); 54] = [
    (KeyCode::KeyA, "A"), (KeyCode::KeyB, "B"), (KeyCode::KeyC, "C"), (KeyCode::KeyD, "D"),
    (KeyCode::KeyE, "E"), (KeyCode::KeyF, "F"), (KeyCode::KeyG, "G"), (KeyCode::KeyH, "H"),
    (KeyCode::KeyI, "I"), (KeyCode::KeyJ, "J"), (KeyCode::KeyK, "K"), (KeyCode::KeyL, "L"),
    (KeyCode::KeyM, "M"), (KeyCode::KeyN, "N"), (KeyCode::KeyO, "O"), (KeyCode::KeyP, "P"),
    (KeyCode::KeyQ, "Q"), (KeyCode::KeyR, "R"), (KeyCode::KeyS, "S"), (KeyCode::KeyT, "T"),
    (KeyCode::KeyU, "U"), (KeyCode::KeyV, "V"), (KeyCode::KeyW, "W"), (KeyCode::KeyX, "X"),
    (KeyCode::KeyY, "Y"), (KeyCode::KeyZ, "Z"), (KeyCode::Digit0, "0"), (KeyCode::Digit1, "1"),
    (KeyCode::Digit2, "2"), (KeyCode::Digit3, "3"), (KeyCode::Digit4, "4"), (KeyCode::Digit5, "5"),
    (KeyCode::Digit6, "6"), (KeyCode::Digit7, "7"), (KeyCode::Digit8, "8"), (KeyCode::Digit9, "9"),
    (KeyCode::Space, "SPACE"), (KeyCode::Tab, "TAB"), (KeyCode::ShiftLeft, "LSHIFT"), (KeyCode::ShiftRight, "RSHIFT"),
    (KeyCode::ControlLeft, "LCTRL"), (KeyCode::ControlRight, "RCTRL"), (KeyCode::AltLeft, "LALT"), (KeyCode::AltRight, "RALT"),
    (KeyCode::ArrowUp, "UP"), (KeyCode::ArrowDown, "DOWN"), (KeyCode::ArrowLeft, "LEFT"), (KeyCode::ArrowRight, "RIGHT"),
    (KeyCode::Enter, "ENTER"), (KeyCode::Backspace, "BACKSPACE"), (KeyCode::CapsLock, "CAPSLOCK"),
    (KeyCode::Comma, "COMMA"), (KeyCode::Period, "PERIOD"), (KeyCode::Slash, "SLASH"),
];

impl TriggerNotation for InputTrigger {
    fn label(self) -> &'static str {
        match self {
            Self::WheelClick => "WHEEL CLICK",
            Self::WheelUp => "WHEEL UP",
            Self::WheelDown => "WHEEL DOWN",
            Self::Key(code) => KEY_TABLE.iter().find_map(|&(k, name)| if k == code { Some(name) } else { None }).unwrap_or("KEY"),
        }
    }

    fn parse_token(&self, token: &str) -> Option<InputTrigger> {
        let trimmed = token.trim().trim_matches('"');
        if trimmed.eq_ignore_ascii_case("WHEEL CLICK") { return Some(Self::WheelClick); }
        if trimmed.eq_ignore_ascii_case("WHEEL UP") { return Some(Self::WheelUp); }
        if trimmed.eq_ignore_ascii_case("WHEEL DOWN") { return Some(Self::WheelDown); }
        KEY_TABLE.iter().find_map(|&(k, name)| if name.eq_ignore_ascii_case(trimmed) { Some(Self::Key(k)) } else { None })
    }
}

/// Immutable one-to-one mapping assigning strictly one InputTrigger to each ControlAction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionBindings {
    triggers: [InputTrigger; 21],
}

impl ActionBindings {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(triggers: [InputTrigger; 21]) -> Self {
        Self { triggers }
    }

    /// Secondary constructor initializing all 21 actions with their single seeded triggers.
    pub const fn seeded() -> Self {
        Self::new([
            InputTrigger::Key(KeyCode::KeyW), InputTrigger::Key(KeyCode::KeyS),
            InputTrigger::Key(KeyCode::KeyA), InputTrigger::Key(KeyCode::KeyD),
            InputTrigger::Key(KeyCode::Space), InputTrigger::Key(KeyCode::ShiftLeft),
            InputTrigger::Key(KeyCode::ControlLeft), InputTrigger::Key(KeyCode::Tab),
            InputTrigger::Key(KeyCode::KeyC), InputTrigger::Key(KeyCode::KeyQ),
            InputTrigger::Key(KeyCode::KeyE), InputTrigger::WheelClick,
            InputTrigger::WheelUp, InputTrigger::WheelDown,
            InputTrigger::Key(KeyCode::KeyT), InputTrigger::Key(KeyCode::KeyQ),
            InputTrigger::Key(KeyCode::KeyE), InputTrigger::Key(KeyCode::AltLeft),
            InputTrigger::Key(KeyCode::KeyR), InputTrigger::Key(KeyCode::KeyH),
            InputTrigger::Key(KeyCode::KeyM),
        ])
    }

    #[cfg(test)]
    pub const fn trigger_for(&self, action: ControlAction) -> InputTrigger {
        self.triggers[action.slot_index()]
    }

    pub fn matches(&self, action: ControlAction, trigger: InputTrigger) -> bool {
        self.triggers[action.slot_index()] == trigger
    }

    pub fn active_in(&self, action: ControlAction, active: &HashSet<InputTrigger>) -> bool {
        active.contains(&self.triggers[action.slot_index()])
    }

    pub fn label_for(&self, action: ControlAction) -> &'static str {
        self.triggers[action.slot_index()].label()
    }

    pub fn rebind_token(self, action: ControlAction, token: &str) -> Self {
        match self.triggers[action.slot_index()].parse_token(token) {
            Some(trigger) => self.with_binding(action, trigger),
            None => self,
        }
    }

    /// Rebinds `action` to `trigger`, swapping within the same control domain so every domain action stays unique.
    pub fn with_binding(self, action: ControlAction, trigger: InputTrigger) -> Self {
        if trigger == InputTrigger::Key(KeyCode::Escape) {
            return self;
        }
        let mut next = self.triggers;
        let idx = action.slot_index();
        let previous = next[idx];
        let domain = action.spaceflight_domain();
        for peer in ControlAction::all() {
            let pidx = peer.slot_index();
            if peer.spaceflight_domain() == domain && next[pidx] == trigger {
                next[pidx] = previous;
            }
        }
        next[idx] = trigger;
        Self::new(next)
    }

    #[cfg(test)]
    pub fn with_key_binding(self, action: ControlAction, code: KeyCode) -> Self {
        self.with_binding(action, InputTrigger::Key(code))
    }
}
