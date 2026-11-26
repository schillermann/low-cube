mod forge;

use crate::attire::{BeltSatchel, HeldRelic, YouthAttire};
pub use self::forge::{AtmosphericBloomery, VacuumForge, WorkstationStation};

/// Focused nourishment sink decoupling `MealOutcome` from concrete `AvatarBody` internals.
pub trait NourishedBody {
    fn nourish_venison(&mut self);
    fn nourish_oven_roast(&mut self);
}

/// Ambient gaseous envelope surrounding a placed hearth (`AmbientAir` vs. oxygen-free `HardVacuum`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HearthMedium {
    AmbientAir,
    HardVacuum,
}

impl HearthMedium {
    pub const fn from_vacuum(vacuum: bool) -> Self {
        if vacuum { Self::HardVacuum } else { Self::AmbientAir }
    }

    pub const fn oxygen_rich(self) -> bool {
        matches!(self, Self::AmbientAir)
    }

    pub const fn airless_vacuum(self) -> bool {
        matches!(self, Self::HardVacuum)
    }
}

/// Polymorphic nourishment outcome when consuming raw venison vs. oven-baked roast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MealOutcome {
    Untouched,
    RawVenison,
    OvenRoast,
}

impl MealOutcome {
    pub const fn from_relic(relic: HeldRelic) -> Self {
        if relic.hearty_roast() {
            Self::OvenRoast
        } else {
            Self::RawVenison
        }
    }

    pub fn savor(self, body: &mut dyn NourishedBody) -> bool {
        use acoustics::{AcousticWave, HeartyBite};
        match self {
            Self::Untouched => false,
            Self::RawVenison => {
                body.nourish_venison();
                HeartyBite::savory().thump();
                true
            }
            Self::OvenRoast => {
                body.nourish_oven_roast();
                HeartyBite::savory().thump();
                true
            }
        }
    }
}

/// Pure-OOP contract for thermal hearth blocks (`BakingOven` & `BloomeryForge`).
pub trait ThermalHearth {
    fn fire_charge(
        &self,
        hands: (HeldRelic, HeldRelic),
        medium: HearthMedium,
        satchel: BeltSatchel,
        attire: YouthAttire,
        side: f32,
    ) -> Option<(BeltSatchel, YouthAttire, u8)>;
}

/// Null-Object `ThermalHearth` representing unheated stone or soil (`0` fields).
#[derive(Clone, Copy, Debug)]
pub struct ColdStone;

impl ColdStone {
    pub const fn new() -> Self {
        Self
    }
}

impl ThermalHearth for ColdStone {
    fn fire_charge(
        &self,
        _hands: (HeldRelic, HeldRelic),
        _medium: HearthMedium,
        _satchel: BeltSatchel,
        _attire: YouthAttire,
        _side: f32,
    ) -> Option<(BeltSatchel, YouthAttire, u8)> {
        None
    }
}

/// `ThermalHearth` decorator for `BlockMedium::BakingOven` (`~250 °C` wood-fired culinary baking).
#[derive(Clone, Copy, Debug)]
pub struct CulinaryOven<H> {
    origin: H,
}

impl<H> CulinaryOven<H> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: H) -> Self {
        Self { origin }
    }
}

impl<H: ThermalHearth> ThermalHearth for CulinaryOven<H> {
    fn fire_charge(
        &self,
        hands: (HeldRelic, HeldRelic),
        medium: HearthMedium,
        satchel: BeltSatchel,
        attire: YouthAttire,
        side: f32,
    ) -> Option<(BeltSatchel, YouthAttire, u8)> {
        let (active, _) = hands;
        let blocked_ore = matches!(
            active,
            HeldRelic::CopperOre | HeldRelic::IronOre | HeldRelic::GoldOre | HeldRelic::CrustBlock
        );
        if medium.oxygen_rich()
            && !blocked_ore
            && satchel.resource_tally(HeldRelic::BoarMeat, attire) >= 1
            && satchel.resource_tally(HeldRelic::DeadwoodBough, attire) >= 1
        {
            let (ns, na) = satchel.consume_and_equip(
                [(HeldRelic::BoarMeat, 1), (HeldRelic::DeadwoodBough, 1)],
                HeldRelic::RoastedBoar,
                1,
                side,
                attire,
            );
            return Some((ns, na, 100));
        }
        self.origin.fire_charge(hands, medium, satchel, attire, side)
    }
}
