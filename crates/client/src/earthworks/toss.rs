use crate::attire::HeldRelic;

const DOUBLE_TAP_WINDOW_SECS: f32 = 0.38;

/// Polymorphic double-tap cadence window for casting a freshly stowed hand relic onto the ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TossCadence {
    Quiet,
    Primed {
        side_positive: bool,
        relic: HeldRelic,
        remaining: f32,
    },
}

impl TossCadence {
    pub const fn prime(side: f32, relic: HeldRelic) -> Self {
        if relic.gripped() {
            Self::Primed {
                side_positive: side > 0.0,
                relic,
                remaining: DOUBLE_TAP_WINDOW_SECS,
            }
        } else {
            Self::Quiet
        }
    }

    pub fn advance(self, dt: f32) -> Self {
        match self {
            Self::Quiet => Self::Quiet,
            Self::Primed { side_positive, relic, remaining } => {
                let next = remaining - dt.max(0.0);
                if next > 0.0 {
                    Self::Primed { side_positive, relic, remaining: next }
                } else {
                    Self::Quiet
                }
            }
        }
    }

    pub const fn ready_on(self, side: f32) -> Option<HeldRelic> {
        match self {
            Self::Primed { side_positive, relic, remaining }
                if side_positive == (side > 0.0) && remaining > 0.0 =>
            {
                Some(relic)
            }
            _ => None,
        }
    }
}
