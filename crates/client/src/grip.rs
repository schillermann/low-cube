use crate::attire::StrideRhythm;

const RIGHT_HELD: u8 = 1;
const LEFT_HELD: u8 = 2;
const BOTH_HELD: u8 = RIGHT_HELD | LEFT_HELD;
const RIGHT_PLACE: u8 = 4;
const LEFT_PLACE: u8 = 8;
const ANY_PLACE: u8 = RIGHT_PLACE | LEFT_PLACE;
const RIGHT_DIG: u8 = 16;
const LEFT_DIG: u8 = 32;
const BOTH_DIG: u8 = RIGHT_DIG | LEFT_DIG;
const RIGHT_PERCH: u8 = 64;
const LEFT_PERCH: u8 = 128;
const ANY_PERCH: u8 = RIGHT_PERCH | LEFT_PERCH;
const ANY_PENDING: u8 = ANY_PLACE | ANY_PERCH;

/// Encapsulates dual-hand button press, deferred block placement, and active digging states without exposing bitmasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DigGrip {
    mask: u8,
}

impl DigGrip {
    pub const fn idle() -> Self {
        Self { mask: 0 }
    }

    const fn hand_bit(side: f32) -> u8 {
        if side > 0.0 { RIGHT_HELD } else { LEFT_HELD }
    }

    pub const fn any_held(self) -> bool {
        (self.mask & BOTH_HELD) != 0
    }

    pub const fn both_held(self) -> bool {
        (self.mask & BOTH_HELD) == BOTH_HELD
    }

    pub const fn held_on(self, side: f32) -> bool {
        (self.mask & Self::hand_bit(side)) != 0
    }

    pub const fn active_side(self) -> f32 {
        if (self.mask & RIGHT_HELD) != 0 { 1.0 } else { -1.0 }
    }

    pub const fn pending_place_any(self) -> bool {
        (self.mask & ANY_PLACE) != 0
    }

    pub const fn pending_place_side(self) -> f32 {
        if (self.mask & RIGHT_PLACE) != 0 { 1.0 } else { -1.0 }
    }

    pub const fn pending_place_on(self, side: f32) -> bool {
        (self.mask & (Self::hand_bit(side) << 2)) != 0
    }

    pub const fn pending_perch_any(self) -> bool {
        (self.mask & ANY_PERCH) != 0
    }

    pub const fn pending_perch_on(self, side: f32) -> bool {
        (self.mask & (Self::hand_bit(side) << 6)) != 0
    }

    pub const fn pending_click_any(self) -> bool {
        (self.mask & ANY_PENDING) != 0
    }

    pub const fn pending_click_on(self, side: f32) -> bool {
        self.pending_place_on(side) || self.pending_perch_on(side)
    }

    pub const fn combined_both_held(self, side: f32) -> bool {
        ((self.mask & BOTH_HELD) | Self::hand_bit(side)) == BOTH_HELD
    }

    pub const fn with_pending_place(side: f32) -> Self {
        let b = Self::hand_bit(side);
        Self { mask: b | (b << 2) }
    }

    pub const fn with_pending_perch(side: f32) -> Self {
        let b = Self::hand_bit(side);
        Self { mask: b | (b << 6) }
    }

    pub const fn promoted_to_dig(self, side: f32) -> Self {
        Self {
            mask: (self.mask & !ANY_PENDING) | (Self::hand_bit(side) << 4),
        }
    }

    pub const fn with_dig_on(side: f32, weightless_place: bool) -> Self {
        let b = Self::hand_bit(side);
        let place = if weightless_place { b << 2 } else { 0 };
        Self {
            mask: b | (b << 4) | place,
        }
    }

    pub const fn with_both_digging() -> Self {
        Self {
            mask: BOTH_HELD | BOTH_DIG,
        }
    }

    pub const fn without_pending_place(self) -> Self {
        Self {
            mask: self.mask & !ANY_PENDING,
        }
    }

    pub const fn bare_hand_digging(self, r_block: bool, l_block: bool, progress: u32) -> (bool, bool) {
        let both = self.both_held();
        let dig_r = r_block
            && (self.mask & RIGHT_HELD) != 0
            && (progress > 0 || (self.mask & ANY_PLACE) != 0 || (both && l_block));
        let dig_l = l_block
            && (self.mask & LEFT_HELD) != 0
            && (progress > 0 || (self.mask & ANY_PLACE) != 0 || (both && r_block));
        (dig_r, dig_l)
    }
}

/// Encapsulates a single digging stroke's effort, grip transition, recipient hand, and swing rhythm.
#[derive(Clone, Copy, Debug)]
pub struct DigStroke {
    effort: u32,
    grip: DigGrip,
    recipient_side: f32,
    rhythm: StrideRhythm,
}

impl DigStroke {
    pub const fn new(effort: u32, grip: DigGrip, recipient_side: f32, rhythm: StrideRhythm) -> Self {
        Self { effort, grip, recipient_side, rhythm }
    }

    pub fn spade(side: f32, other_receives: bool, rhythm: StrideRhythm) -> Self {
        let recipient = if other_receives { -side } else { side };
        let next_rhythm = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
        Self::new(4, DigGrip::with_dig_on(side, false), recipient, next_rhythm)
    }

    pub fn dual_palm(
        side: f32, prev_grip: DigGrip, active_receives: bool, weightless: bool, weightless_place: bool, rhythm: StrideRhythm,
    ) -> Self {
        let step = if prev_grip.both_held() { 2 } else { 1 + u32::from(prev_grip.pending_click_any()) };
        let recipient = if active_receives { side } else { -side };
        let next_grip = if weightless { DigGrip::with_dig_on(side, weightless_place) } else { DigGrip::with_both_digging() };
        Self::new(if weightless { 4 } else { step }, next_grip, recipient, rhythm.trigger_right_strike().trigger_left_strike())
    }

    pub fn single_palm(side: f32, deferred: u32, weightless: bool, weightless_place: bool, rhythm: StrideRhythm) -> Self {
        let next_rhythm = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
        Self::new(if weightless { 4 } else { 1 + deferred }, DigGrip::with_dig_on(side, weightless_place), side, next_rhythm)
    }

    pub const fn advance_progress(self, current: u32) -> u32 {
        current + self.effort
    }

    pub const fn next_grip(self) -> DigGrip {
        self.grip
    }

    pub const fn recipient_side(self) -> f32 {
        self.recipient_side
    }

    pub const fn next_rhythm(self) -> StrideRhythm {
        self.rhythm
    }
}
