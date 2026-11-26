use world::{BlockMedium, PlanetaryWorld};
use crate::attire::{ArtisanTome, BeltSatchel, HeldRelic, YouthAttire};
use super::{ColdStone, CulinaryOven, HearthMedium, ThermalHearth};

/// `ThermalHearth` decorator for atmospheric `BlockMedium::BloomeryForge` (`~1,300 °C` ore reduction).
#[derive(Clone, Copy, Debug)]
pub struct AtmosphericBloomery<H> {
    origin: H,
}

impl<H> AtmosphericBloomery<H> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: H) -> Self {
        Self { origin }
    }

    fn try_smelt(
        ore: HeldRelic,
        bloom: HeldRelic,
        recipe_code: u8,
        satchel: BeltSatchel,
        attire: YouthAttire,
        side: f32,
    ) -> Option<(BeltSatchel, YouthAttire, u8)> {
        if satchel.resource_tally(ore, attire) >= 1
            && satchel.resource_tally(HeldRelic::DeadwoodBough, attire) >= 1
        {
            let (ns, na) = satchel.consume_and_equip(
                [(ore, 1), (HeldRelic::DeadwoodBough, 1)],
                bloom,
                1,
                side,
                attire,
            );
            Some((ns, na, recipe_code))
        } else {
            None
        }
    }
}

impl<H: ThermalHearth> ThermalHearth for AtmosphericBloomery<H> {
    fn fire_charge(
        &self,
        hands: (HeldRelic, HeldRelic),
        medium: HearthMedium,
        satchel: BeltSatchel,
        attire: YouthAttire,
        side: f32,
    ) -> Option<(BeltSatchel, YouthAttire, u8)> {
        let (active, offhand) = hands;
        if medium.oxygen_rich() && !active.edible_provision() && active != HeldRelic::CrustBlock {
            let prefer_copper = active == HeldRelic::CopperOre
                || (active != HeldRelic::IronOre && offhand == HeldRelic::CopperOre);
            let outcome = if prefer_copper {
                Self::try_smelt(HeldRelic::CopperOre, HeldRelic::CopperBloom, 101, satchel, attire, side)
                    .or_else(|| Self::try_smelt(HeldRelic::IronOre, HeldRelic::IronBloom, 102, satchel, attire, side))
            } else {
                Self::try_smelt(HeldRelic::IronOre, HeldRelic::IronBloom, 102, satchel, attire, side)
                    .or_else(|| Self::try_smelt(HeldRelic::CopperOre, HeldRelic::CopperBloom, 101, satchel, attire, side))
            };
            if outcome.is_some() {
                return outcome;
            }
        }
        self.origin.fire_charge(hands, medium, satchel, attire, side)
    }
}

/// `ThermalHearth` decorator for vacuum `BlockMedium::BloomeryForge` (**Space-Forged** oxygen-free titanium reduction).
#[derive(Clone, Copy, Debug)]
pub struct VacuumForge<H> {
    origin: H,
}

impl<H> VacuumForge<H> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: H) -> Self {
        Self { origin }
    }
}

impl<H: ThermalHearth> ThermalHearth for VacuumForge<H> {
    fn fire_charge(
        &self,
        hands: (HeldRelic, HeldRelic),
        medium: HearthMedium,
        satchel: BeltSatchel,
        attire: YouthAttire,
        side: f32,
    ) -> Option<(BeltSatchel, YouthAttire, u8)> {
        let (active, _) = hands;
        if medium.airless_vacuum()
            && !active.edible_provision()
            && active != HeldRelic::CopperOre
            && satchel.resource_tally(HeldRelic::CrustBlock, attire) >= 1
            && satchel.resource_tally(HeldRelic::IronOre, attire) >= 1
        {
            let (ns, na) = satchel.consume_and_equip(
                [(HeldRelic::CrustBlock, 1), (HeldRelic::IronOre, 1)],
                HeldRelic::TitaniumBillet,
                2,
                side,
                attire,
            );
            return Some((ns, na, 103));
        }
        self.origin.fire_charge(hands, medium, satchel, attire, side)
    }
}

/// Placed workstation contact resolver dispatching `BakingOven`, `BloomeryForge`, and `SmithyAnvil` (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct WorkstationStation {
    medium: BlockMedium,
    ambient: HearthMedium,
}

impl WorkstationStation {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(medium: BlockMedium, ambient: HearthMedium) -> Self {
        Self { medium, ambient }
    }

    pub const fn claims_hand(medium: BlockMedium, held: HeldRelic) -> bool {
        if held.pickaxe_head() {
            return false;
        }
        match medium {
            BlockMedium::SmithyAnvil => !matches!(
                held,
                HeldRelic::BakingOven | HeldRelic::BloomeryForge | HeldRelic::SmithyAnvil
            ),
            BlockMedium::BakingOven | BlockMedium::BloomeryForge => matches!(
                held,
                HeldRelic::EmptyPalm
                    | HeldRelic::BoarMeat
                    | HeldRelic::DeadwoodBough
                    | HeldRelic::WoodenMallet
                    | HeldRelic::TimberTorch
                    | HeldRelic::CopperOre
                    | HeldRelic::IronOre
                    | HeldRelic::GoldOre
                    | HeldRelic::CrustBlock
                    | HeldRelic::CopperBloom
                    | HeldRelic::IronBloom
                    | HeldRelic::TitaniumBillet
            ),
            _ => false,
        }
    }

    pub fn interact(
        self,
        side: f32,
        satchel: BeltSatchel,
        tome: ArtisanTome,
        attire: YouthAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> Option<(BeltSatchel, ArtisanTome, YouthAttire, bool)> {
        let hands = attire.hand_pair(side);
        if !Self::claims_hand(self.medium, hands.0) {
            return None;
        }
        let reconciled = satchel.reconcile_attire(attire);
        match self.medium {
            BlockMedium::SmithyAnvil => {
                let opened = tome.open_at_anvil(&reconciled, attire);
                Some((reconciled.with_open(false), opened, attire, true))
            }
            BlockMedium::BakingOven => {
                let oven = CulinaryOven::new(ColdStone::new());
                if let Some((ns, na, recipe)) = oven.fire_charge(hands, self.ambient, reconciled, attire, side) {
                    let noted = tome.noted_recipe(recipe).drain_crafted_notes(world);
                    Some((ns, noted, na, true))
                } else {
                    Some((reconciled, tome, attire, false))
                }
            }
            BlockMedium::BloomeryForge => {
                let forge = VacuumForge::new(AtmosphericBloomery::new(ColdStone::new()));
                if let Some((ns, na, recipe)) = forge.fire_charge(hands, self.ambient, reconciled, attire, side) {
                    let noted = tome.noted_recipe(recipe).drain_crafted_notes(world);
                    Some((ns, noted, na, true))
                } else {
                    Some((reconciled, tome, attire, false))
                }
            }
            _ => None,
        }
    }
}
