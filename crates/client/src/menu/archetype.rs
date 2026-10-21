use crate::attire::{
    ComplexionSwatch, CrownStyle, FootwearStyle, GirdleStyle, HeldRelic, PartProportion,
    PigmentSwatch, TunicStyle, VambraceStyle, YouthAttire, YouthPhysique,
};

/// Distinct Polygon-style character body archetypes with unique silhouettes, headgear, back gear, and anatomy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarArchetype {
    Knight,
    Ranger,
    Woodsman,
    Blacksmith,
    Sentinel,
    Noble,
}

impl AvatarArchetype {
    pub fn all() -> [Self; 6] {
        [
            Self::Knight,
            Self::Ranger,
            Self::Woodsman,
            Self::Blacksmith,
            Self::Sentinel,
            Self::Noble,
        ]
    }

    pub fn gender_title(self, feminine: bool) -> &'static str {
        if feminine {
            match self {
                Self::Knight => "VALKYRIE",
                Self::Ranger => "HUNTRESS",
                Self::Woodsman => "HERBALIST",
                Self::Blacksmith => "WEAVER",
                Self::Sentinel => "SHIELDMAID",
                Self::Noble => "LADY",
            }
        } else {
            match self {
                Self::Knight => "KNIGHT",
                Self::Ranger => "RANGER",
                Self::Woodsman => "WOODSMAN",
                Self::Blacksmith => "BLACKSMITH",
                Self::Sentinel => "SENTINEL",
                Self::Noble => "NOBLE",
            }
        }
    }

    pub fn attire_for(self, feminine: bool) -> YouthAttire {
        if feminine { self.female_attire() } else { self.attire() }
    }

    pub fn female_attire(self) -> YouthAttire {
        let prop = |c: (u8, u8), t: (u8, u8), a: (u8, u8), l: (u8, u8)| {
            YouthPhysique::with_feminine(PartProportion::new(c.0, c.1), PartProportion::new(t.0, t.1), PartProportion::new(a.0, a.1), PartProportion::new(l.0, l.1), true)
        };
        match self {
            Self::Knight => YouthAttire::new(TunicStyle::PlateCuirass, VambraceStyle::PlateGauntlets, GirdleStyle::KnightlyBelt, FootwearStyle::PlateGreaves, CrownStyle::VisoredHelm, HeldRelic::EmptyPalm, HeldRelic::EmptyPalm, PigmentSwatch::OchreGold, ComplexionSwatch::Porcelain, prop((4, 3), (4, 3), (4, 2), (5, 3))),
            Self::Ranger => YouthAttire::new(TunicStyle::PeasantShirt, VambraceStyle::RawhideWraps, GirdleStyle::PouchedGirdle, FootwearStyle::Turnshoes, CrownStyle::MedievalGugel, HeldRelic::EmptyPalm, HeldRelic::EmptyPalm, PigmentSwatch::ForestMoss, ComplexionSwatch::Meadow, prop((4, 3), (4, 2), (4, 2), (5, 3))),
            Self::Woodsman => YouthAttire::new(TunicStyle::QuiltedJerkin, VambraceStyle::BareForearms, GirdleStyle::UnbeltedWaist, FootwearStyle::LacedBoots, CrownStyle::MerchantCap, HeldRelic::EmptyPalm, HeldRelic::EmptyPalm, PigmentSwatch::RussetBark, ComplexionSwatch::Birch, prop((4, 3), (4, 3), (4, 2), (4, 3))),
            Self::Blacksmith => YouthAttire::new(TunicStyle::ForgeApron, VambraceStyle::PuffedSleeves, GirdleStyle::KnottedCord, FootwearStyle::Turnshoes, CrownStyle::SmithGoggles, HeldRelic::EmptyPalm, HeldRelic::EmptyPalm, PigmentSwatch::MadderCrimson, ComplexionSwatch::Amber, prop((4, 3), (4, 3), (4, 2), (4, 3))),
            Self::Sentinel => YouthAttire::new(TunicStyle::HeraldicTabard, VambraceStyle::StuddedVambraces, GirdleStyle::GarrisonBelt, FootwearStyle::GarrisonBoots, CrownStyle::KettleHelm, HeldRelic::EmptyPalm, HeldRelic::EmptyPalm, PigmentSwatch::WoadIndigo, ComplexionSwatch::Bronze, prop((4, 3), (5, 3), (4, 3), (5, 3))),
            Self::Noble => YouthAttire::new(TunicStyle::LinenDoublet, VambraceStyle::PuffedSleeves, GirdleStyle::KnottedCord, FootwearStyle::CuffedBoots, CrownStyle::SquireCrop, HeldRelic::EmptyPalm, HeldRelic::EmptyPalm, PigmentSwatch::MadderCrimson, ComplexionSwatch::Porcelain, prop((4, 2), (4, 2), (4, 2), (5, 2))),
        }
    }

    pub fn attire(self) -> YouthAttire {
        match self {
            Self::Knight => YouthAttire::new(
                TunicStyle::PlateCuirass,
                VambraceStyle::PlateGauntlets,
                GirdleStyle::KnightlyBelt,
                FootwearStyle::PlateGreaves,
                CrownStyle::VisoredHelm,
                HeldRelic::EmptyPalm,
                HeldRelic::EmptyPalm,
                PigmentSwatch::MadderCrimson,
                ComplexionSwatch::Porcelain,
                YouthPhysique::new(
                    PartProportion::new(4, 5),
                    PartProportion::new(5, 6),
                    PartProportion::new(5, 6),
                    PartProportion::new(5, 5),
                ),
            ),
            Self::Ranger => YouthAttire::new(
                TunicStyle::PeasantShirt,
                VambraceStyle::RawhideWraps,
                GirdleStyle::PouchedGirdle,
                FootwearStyle::Turnshoes,
                CrownStyle::MedievalGugel,
                HeldRelic::EmptyPalm,
                HeldRelic::EmptyPalm,
                PigmentSwatch::ForestMoss,
                ComplexionSwatch::Meadow,
                YouthPhysique::new(
                    PartProportion::new(4, 4),
                    PartProportion::new(4, 3),
                    PartProportion::new(5, 4),
                    PartProportion::new(6, 4),
                ),
            ),
            Self::Woodsman => YouthAttire::new(
                TunicStyle::QuiltedJerkin,
                VambraceStyle::BareForearms,
                GirdleStyle::UnbeltedWaist,
                FootwearStyle::LacedBoots,
                CrownStyle::MerchantCap,
                HeldRelic::EmptyPalm,
                HeldRelic::EmptyPalm,
                PigmentSwatch::RussetBark,
                ComplexionSwatch::Amber,
                YouthPhysique::new(
                    PartProportion::new(4, 6),
                    PartProportion::new(5, 7),
                    PartProportion::new(5, 7),
                    PartProportion::new(4, 6),
                ),
            ),
            Self::Blacksmith => YouthAttire::new(
                TunicStyle::ForgeApron,
                VambraceStyle::ForgeGloves,
                GirdleStyle::ForgeBelt,
                FootwearStyle::ForgeBoots,
                CrownStyle::SmithGoggles,
                HeldRelic::EmptyPalm,
                HeldRelic::EmptyPalm,
                PigmentSwatch::CharcoalSlate,
                ComplexionSwatch::Umber,
                YouthPhysique::new(
                    PartProportion::new(4, 6),
                    PartProportion::new(5, 8),
                    PartProportion::new(5, 8),
                    PartProportion::new(4, 7),
                ),
            ),
            Self::Sentinel => YouthAttire::new(
                TunicStyle::HeraldicTabard,
                VambraceStyle::StuddedVambraces,
                GirdleStyle::GarrisonBelt,
                FootwearStyle::GarrisonBoots,
                CrownStyle::KettleHelm,
                HeldRelic::EmptyPalm,
                HeldRelic::EmptyPalm,
                PigmentSwatch::OchreGold,
                ComplexionSwatch::Bronze,
                YouthPhysique::new(
                    PartProportion::new(4, 6),
                    PartProportion::new(4, 7),
                    PartProportion::new(4, 6),
                    PartProportion::new(3, 7),
                ),
            ),
            Self::Noble => YouthAttire::new(
                TunicStyle::LinenDoublet,
                VambraceStyle::PuffedSleeves,
                GirdleStyle::KnottedCord,
                FootwearStyle::CuffedBoots,
                CrownStyle::SquireCrop,
                HeldRelic::EmptyPalm,
                HeldRelic::EmptyPalm,
                PigmentSwatch::WoadIndigo,
                ComplexionSwatch::Birch,
                YouthPhysique::new(
                    PartProportion::new(4, 4),
                    PartProportion::new(5, 5),
                    PartProportion::new(4, 5),
                    PartProportion::new(5, 4),
                ),
            ),
        }
    }
}
