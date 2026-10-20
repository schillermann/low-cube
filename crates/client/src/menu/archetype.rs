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

    pub fn title(self) -> &'static str {
        match self {
            Self::Knight => "KNIGHT",
            Self::Ranger => "RANGER",
            Self::Woodsman => "WOODSMAN",
            Self::Blacksmith => "BLACKSMITH",
            Self::Sentinel => "SENTINEL",
            Self::Noble => "NOBLE",
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
