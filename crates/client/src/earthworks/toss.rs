use glam::Vec3;
use world::PlanetaryWorld;
use crate::attire::{BeltSatchel, EvaAttire, HeldRelic, StrideRhythm};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
use super::SurfaceReach;

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

    pub fn try_cast(
        self,
        satchel: BeltSatchel,
        trigger: InputTrigger,
        bindings: &ActionBindings,
        attire: EvaAttire,
        rhythm: StrideRhythm,
        eye: Vec3,
        view_dir: Vec3,
        world: &mut dyn PlanetaryWorld,
    ) -> (BeltSatchel, Self, StrideRhythm, bool) {
        let side = if bindings.matches(ControlAction::SatchelRight, trigger) {
            1.0
        } else if bindings.matches(ControlAction::SatchelLeft, trigger) {
            -1.0
        } else {
            return (satchel, self, rhythm, false);
        };
        let Some(relic) = self.ready_on(side) else {
            return (satchel, self, rhythm, false);
        };
        let (next_satchel, taken) = satchel.take_cast_relic(relic);
        if !taken || !world.cast_surface_find(eye, view_dir, relic.surface_find()) {
            return (satchel, Self::Quiet, rhythm, false);
        }
        (next_satchel, Self::Quiet, SurfaceReach::swing_rhythm(attire, rhythm, side), true)
    }
}
