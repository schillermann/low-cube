use crate::attire::{
    ComplexionSwatch, CrewIdentity, CrewPhysique, EvaAttire, HeldHands, HelmStyle, MagBootStyle,
    PartProportion, PigmentSwatch, PlssHarness, SuitEnsemble, SuitGauntlet, SuitShell,
};

/// Space-Age crew archetypes with distinct EVA suit silhouettes, helmets, PLSS packs, and stature builds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarArchetype {
    Pilot,
    Specialist,
    Geologist,
    Machinist,
    Vanguard,
    Envoy,
}

impl AvatarArchetype {
    pub fn all() -> [Self; 6] {
        [
            Self::Pilot,
            Self::Specialist,
            Self::Geologist,
            Self::Machinist,
            Self::Vanguard,
            Self::Envoy,
        ]
    }

    pub fn gender_title(self, feminine: bool) -> &'static str {
        match (self, feminine) {
            (Self::Pilot, false) => "COMMAND PILOT",
            (Self::Pilot, true) => "FLIGHT CAPTAIN",
            (Self::Specialist, false) => "EVA SPECIALIST",
            (Self::Specialist, true) => "MISSION SPECIALIST",
            (Self::Geologist, false) => "FIELD GEOLOGIST",
            (Self::Geologist, true) => "ASTROGEOLOGIST",
            (Self::Machinist, false) => "RIG MACHINIST",
            (Self::Machinist, true) => "FLIGHT ENGINEER",
            (Self::Vanguard, false) => "DECK VANGUARD",
            (Self::Vanguard, true) => "ORBITAL VANGUARD",
            (Self::Envoy, false) => "STATION ENVOY",
            (Self::Envoy, true) => "COMMS ENVOY",
        }
    }

    pub fn attire_for(self, feminine: bool) -> EvaAttire {
        if feminine { self.female_attire() } else { self.attire() }
    }

    fn proportion(c: (u8, u8), t: (u8, u8), a: (u8, u8), l: (u8, u8), feminine: bool) -> CrewPhysique {
        CrewPhysique::with_feminine(
            PartProportion::new(c.0, c.1),
            PartProportion::new(t.0, t.1),
            PartProportion::new(a.0, a.1),
            PartProportion::new(l.0, l.1),
            feminine,
        )
    }

    pub fn female_attire(self) -> EvaAttire {
        let (suit, identity) = match self {
            Self::Pilot => (
                SuitEnsemble::new(SuitShell::ArmoredRig, SuitGauntlet::TitaniumGauntlets, PlssHarness::JetpackHarness, MagBootStyle::ArmoredSabatons),
                CrewIdentity::new(HelmStyle::SolarVisor, PigmentSwatch::SolarGold, ComplexionSwatch::Porcelain, Self::proportion((4, 3), (4, 3), (4, 2), (5, 3), true)),
            ),
            Self::Specialist => (
                SuitEnsemble::new(SuitShell::ThermalShell, SuitGauntlet::TactileGloves, PlssHarness::SamplePouchRig, MagBootStyle::DeckSlipons),
                CrewIdentity::new(HelmStyle::SurveyDome, PigmentSwatch::OrbitalOlive, ComplexionSwatch::Meadow, Self::proportion((4, 3), (4, 2), (4, 2), (5, 3), true)),
            ),
            Self::Geologist => (
                SuitEnsemble::new(SuitShell::SurveySuit, SuitGauntlet::SealCuffs, PlssHarness::PlssBackpack, MagBootStyle::TreadedGaiters),
                CrewIdentity::new(HelmStyle::CommsCanopy, PigmentSwatch::MartianRust, ComplexionSwatch::Birch, Self::proportion((4, 3), (4, 3), (4, 2), (4, 3), true)),
            ),
            Self::Machinist => (
                SuitEnsemble::new(SuitShell::TechCoverall, SuitGauntlet::TelemetrySleeves, PlssHarness::UmbilicalBelt, MagBootStyle::DeckSlipons),
                CrewIdentity::new(HelmStyle::OpticGoggles, PigmentSwatch::SignalCrimson, ComplexionSwatch::Amber, Self::proportion((4, 3), (4, 3), (4, 2), (4, 3), true)),
            ),
            Self::Vanguard => (
                SuitEnsemble::new(SuitShell::CommandSuit, SuitGauntlet::InsulatedWraps, PlssHarness::BeaconBelt, MagBootStyle::ClampBoots),
                CrewIdentity::new(HelmStyle::ArmoredCowl, PigmentSwatch::DeepCobalt, ComplexionSwatch::Bronze, Self::proportion((4, 3), (5, 3), (4, 3), (5, 3), true)),
            ),
            Self::Envoy => (
                SuitEnsemble::new(SuitShell::FlightSuit, SuitGauntlet::TelemetrySleeves, PlssHarness::UmbilicalBelt, MagBootStyle::MagLockBoots),
                CrewIdentity::new(HelmStyle::ClearBubble, PigmentSwatch::LunarWhite, ComplexionSwatch::Porcelain, Self::proportion((4, 2), (4, 2), (4, 2), (5, 2), true)),
            ),
        };
        EvaAttire::new(suit, identity, HeldHands::empty())
    }

    pub fn attire(self) -> EvaAttire {
        let (suit, identity) = match self {
            Self::Pilot => (
                SuitEnsemble::new(SuitShell::ArmoredRig, SuitGauntlet::TitaniumGauntlets, PlssHarness::JetpackHarness, MagBootStyle::ArmoredSabatons),
                CrewIdentity::new(HelmStyle::SolarVisor, PigmentSwatch::SignalCrimson, ComplexionSwatch::Porcelain, Self::proportion((4, 5), (5, 6), (5, 6), (5, 5), false)),
            ),
            Self::Specialist => (
                SuitEnsemble::new(SuitShell::ThermalShell, SuitGauntlet::TactileGloves, PlssHarness::SamplePouchRig, MagBootStyle::DeckSlipons),
                CrewIdentity::new(HelmStyle::SurveyDome, PigmentSwatch::OrbitalOlive, ComplexionSwatch::Meadow, Self::proportion((4, 4), (4, 3), (5, 4), (6, 4), false)),
            ),
            Self::Geologist => (
                SuitEnsemble::new(SuitShell::SurveySuit, SuitGauntlet::SealCuffs, PlssHarness::PlssBackpack, MagBootStyle::TreadedGaiters),
                CrewIdentity::new(HelmStyle::CommsCanopy, PigmentSwatch::MartianRust, ComplexionSwatch::Amber, Self::proportion((4, 6), (5, 7), (5, 7), (4, 6), false)),
            ),
            Self::Machinist => (
                SuitEnsemble::new(SuitShell::TechCoverall, SuitGauntlet::WeldMitts, PlssHarness::ToolRigBelt, MagBootStyle::ThermalBoots),
                CrewIdentity::new(HelmStyle::OpticGoggles, PigmentSwatch::CarbonSlate, ComplexionSwatch::Umber, Self::proportion((4, 6), (5, 8), (5, 8), (4, 7), false)),
            ),
            Self::Vanguard => (
                SuitEnsemble::new(SuitShell::CommandSuit, SuitGauntlet::InsulatedWraps, PlssHarness::BeaconBelt, MagBootStyle::ClampBoots),
                CrewIdentity::new(HelmStyle::ArmoredCowl, PigmentSwatch::SolarGold, ComplexionSwatch::Bronze, Self::proportion((4, 6), (4, 7), (4, 6), (3, 7), false)),
            ),
            Self::Envoy => (
                SuitEnsemble::new(SuitShell::FlightSuit, SuitGauntlet::TelemetrySleeves, PlssHarness::UmbilicalBelt, MagBootStyle::MagLockBoots),
                CrewIdentity::new(HelmStyle::ClearBubble, PigmentSwatch::LunarWhite, ComplexionSwatch::Birch, Self::proportion((4, 4), (5, 5), (4, 5), (5, 4), false)),
            ),
        };
        EvaAttire::new(suit, identity, HeldHands::empty())
    }
}
