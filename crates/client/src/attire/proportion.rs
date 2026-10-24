/// Anatomical body region for independent size (Größe) and thickness (Dicke) customization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyRegion {
    Crown,
    Torso,
    Arms,
    Legs,
}

impl BodyRegion {
    pub fn all() -> [Self; 4] {
        [Self::Crown, Self::Torso, Self::Arms, Self::Legs]
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Crown => "HEAD",
            Self::Torso => "TORSO",
            Self::Arms => "ARMS",
            Self::Legs => "LEGS",
        }
    }
}

/// Immutable size (length/height) and girth (thickness/width) steps for an individual body part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartProportion {
    size_step: u8,
    girth_step: u8,
}

impl PartProportion {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(size_step: u8, girth_step: u8) -> Self {
        Self { size_step, girth_step }
    }

    pub const fn squire() -> Self { Self::new(4, 4) }
    pub const fn scout() -> Self { Self::new(4, 5) }
    pub const fn apprentice() -> Self { Self::new(4, 6) }

    pub fn adjusted(self, delta_size: i8, delta_girth: i8) -> Self {
        let next_size = (self.size_step as i8 + delta_size).clamp(0, 8) as u8;
        let next_girth = (self.girth_step as i8 + delta_girth).clamp(0, 8) as u8;
        Self::new(next_size, next_girth)
    }

    pub fn scale_size(self, measure: f32) -> f32 {
        let factor = match self.size_step {
            0 => 0.72, 1 => 0.79, 2 => 0.86, 3 => 0.93, 4 => 1.00,
            5 => 1.08, 6 => 1.16, 7 => 1.24, _ => 1.32,
        };
        measure * factor
    }

    pub fn scale_girth(self, measure: f32) -> f32 {
        let factor = match self.girth_step {
            0 => 0.68, 1 => 0.74, 2 => 0.80, 3 => 0.87, 4 => 0.94,
            5 => 1.00, 6 => 1.08, 7 => 1.18, _ => 1.28,
        };
        measure * factor
    }

    pub fn scale_span(self, measure: f32) -> f32 { self.scale_girth(measure) / 0.94 }
    pub fn size_ratio(self) -> f32 { (self.size_step.min(8) as f32) / 8.0 }
    pub fn girth_ratio(self) -> f32 { (self.girth_step.min(8) as f32) / 8.0 }
    pub fn packed_byte(self) -> u8 { self.size_step.min(8) * 9 + self.girth_step.min(8) }
    pub fn from_packed_byte(byte: u8) -> Self { let v = (byte & 0x7F).min(80); Self::new(v / 9, v % 9) }
}

/// Customizable stature build scaling individual body regions (Kopf, Torso, Arme, Beine) and gender form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YouthPhysique {
    crown: PartProportion,
    torso: PartProportion,
    arms: PartProportion,
    legs: PartProportion,
    feminine: bool,
}

impl YouthPhysique {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_feminine(crown: PartProportion, torso: PartProportion, arms: PartProportion, legs: PartProportion, feminine: bool) -> Self {
        Self { crown, torso, arms, legs, feminine }
    }

    pub const fn new(crown: PartProportion, torso: PartProportion, arms: PartProportion, legs: PartProportion) -> Self {
        Self::with_feminine(crown, torso, arms, legs, false)
    }

    pub const fn as_feminine(self, feminine: bool) -> Self {
        Self::with_feminine(self.crown, self.torso, self.arms, self.legs, feminine)
    }

    pub const fn feminine(self) -> bool { self.feminine }

    #[allow(non_upper_case_globals)]
    pub const SlenderSquire: Self = Self::new(PartProportion::squire(), PartProportion::squire(), PartProportion::squire(), PartProportion::squire());
    #[allow(non_upper_case_globals)]
    pub const AgileScout: Self = Self::new(PartProportion::scout(), PartProportion::scout(), PartProportion::scout(), PartProportion::scout());
    #[allow(non_upper_case_globals)]
    pub const SturdyApprentice: Self = Self::new(PartProportion::apprentice(), PartProportion::apprentice(), PartProportion::apprentice(), PartProportion::apprentice());

    pub fn next(self) -> Self {
        let base = if self.torso == PartProportion::squire() { Self::AgileScout }
            else if self.torso == PartProportion::scout() { Self::SturdyApprentice }
            else { Self::SlenderSquire };
        base.as_feminine(self.feminine)
    }

    pub fn from_code(code: u8) -> Self {
        let base = match (code & 7) % 3 { 0 => Self::SlenderSquire, 1 => Self::AgileScout, _ => Self::SturdyApprentice };
        base.as_feminine((code & 8) != 0)
    }

    pub fn code(self) -> u8 {
        let base = if self.torso == PartProportion::squire() { 0 } else if self.torso == PartProportion::scout() { 1 } else { 2 };
        base | if self.feminine { 8 } else { 0 }
    }

    pub fn adjusted(self, region: BodyRegion, delta_size: i8, delta_girth: i8) -> Self {
        match region {
            BodyRegion::Crown => Self::with_feminine(self.crown.adjusted(delta_size, delta_girth), self.torso, self.arms, self.legs, self.feminine),
            BodyRegion::Torso => Self::with_feminine(self.crown, self.torso.adjusted(delta_size, delta_girth), self.arms, self.legs, self.feminine),
            BodyRegion::Arms => Self::with_feminine(self.crown, self.torso, self.arms.adjusted(delta_size, delta_girth), self.legs, self.feminine),
            BodyRegion::Legs => Self::with_feminine(self.crown, self.torso, self.arms, self.legs.adjusted(delta_size, delta_girth), self.feminine),
        }
    }

    pub fn region_proportion(self, region: BodyRegion) -> PartProportion {
        match region {
            BodyRegion::Crown => self.crown,
            BodyRegion::Torso => self.torso,
            BodyRegion::Arms => self.arms,
            BodyRegion::Legs => self.legs,
        }
    }

    pub fn scale(self, measure: f32) -> f32 { self.torso.scale_girth(measure) }
    pub fn crown_size(self, measure: f32) -> f32 { self.crown.scale_size(measure) }
    pub fn crown_girth(self, measure: f32) -> f32 { self.crown.scale_girth(measure) }
    pub fn crown_span(self, measure: f32) -> f32 { self.crown.scale_span(measure) }
    pub fn torso_size(self, measure: f32) -> f32 { self.torso.scale_size(measure) }
    pub fn torso_depth(self, measure: f32) -> f32 { self.torso.scale_span(measure) }
    pub fn arm_size(self, measure: f32) -> f32 { self.arms.scale_size(measure) }
    pub fn arm_girth(self, measure: f32) -> f32 { self.arms.scale_girth(measure) }
    pub fn leg_size(self, measure: f32) -> f32 { self.legs.scale_size(measure) }
    pub fn leg_girth(self, measure: f32) -> f32 { self.legs.scale_girth(measure) }

    pub fn stature_height(self) -> f32 {
        self.leg_size(0.68) + self.torso_size(0.88) + self.crown_size(0.24)
    }

    pub fn upper_body_span(self) -> f32 {
        2.0 * (self.scale(if self.feminine { 0.194 } else { 0.222 }) + self.arm_girth(0.075) + self.arm_size(0.045))
    }

    pub fn body_depth(self) -> f32 {
        2.0 * (self.torso_depth(if self.feminine { 0.148 } else { 0.135 }) + self.crown_span(0.038) + self.arm_girth(0.025))
    }

    pub fn shaft_yaw_play(self) -> f32 {
        (1.05 - (self.upper_body_span() - 0.648) * 2.1).clamp(0.45, 1.35)
    }

    pub fn region_bytes(self) -> [u8; 4] {
        [self.crown.packed_byte() | if self.feminine { 0x80 } else { 0 }, self.torso.packed_byte(), self.arms.packed_byte(), self.legs.packed_byte()]
    }

    pub fn from_region_bytes(bytes: [u8; 4]) -> Self {
        Self::with_feminine(
            PartProportion::from_packed_byte(bytes[0]),
            PartProportion::from_packed_byte(bytes[1]),
            PartProportion::from_packed_byte(bytes[2]),
            PartProportion::from_packed_byte(bytes[3]),
            (bytes[0] & 0x80) != 0,
        )
    }
}
