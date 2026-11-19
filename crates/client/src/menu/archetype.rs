use crate::attire::{
    ComplexionSwatch, CrownStyle, FootwearStyle, GarmentEnsemble, GirdleStyle, HeldHands,
    IdentityTraits, PartProportion, PigmentSwatch, TunicStyle, VambraceStyle, YouthAttire,
    YouthPhysique,
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

    fn proportion(c: (u8, u8), t: (u8, u8), a: (u8, u8), l: (u8, u8), feminine: bool) -> YouthPhysique {
        YouthPhysique::with_feminine(
            PartProportion::new(c.0, c.1),
            PartProportion::new(t.0, t.1),
            PartProportion::new(a.0, a.1),
            PartProportion::new(l.0, l.1),
            feminine,
        )
    }

    pub fn female_attire(self) -> YouthAttire {
        let (garments, identity) = match self {
            Self::Knight => (
                GarmentEnsemble::new(TunicStyle::PlateCuirass, VambraceStyle::PlateGauntlets, GirdleStyle::KnightlyBelt, FootwearStyle::PlateGreaves),
                IdentityTraits::new(CrownStyle::VisoredHelm, PigmentSwatch::OchreGold, ComplexionSwatch::Porcelain, Self::proportion((4, 3), (4, 3), (4, 2), (5, 3), true)),
            ),
            Self::Ranger => (
                GarmentEnsemble::new(TunicStyle::PeasantShirt, VambraceStyle::RawhideWraps, GirdleStyle::PouchedGirdle, FootwearStyle::Turnshoes),
                IdentityTraits::new(CrownStyle::MedievalGugel, PigmentSwatch::ForestMoss, ComplexionSwatch::Meadow, Self::proportion((4, 3), (4, 2), (4, 2), (5, 3), true)),
            ),
            Self::Woodsman => (
                GarmentEnsemble::new(TunicStyle::QuiltedJerkin, VambraceStyle::BareForearms, GirdleStyle::UnbeltedWaist, FootwearStyle::LacedBoots),
                IdentityTraits::new(CrownStyle::MerchantCap, PigmentSwatch::RussetBark, ComplexionSwatch::Birch, Self::proportion((4, 3), (4, 3), (4, 2), (4, 3), true)),
            ),
            Self::Blacksmith => (
                GarmentEnsemble::new(TunicStyle::ForgeApron, VambraceStyle::PuffedSleeves, GirdleStyle::KnottedCord, FootwearStyle::Turnshoes),
                IdentityTraits::new(CrownStyle::SmithGoggles, PigmentSwatch::MadderCrimson, ComplexionSwatch::Amber, Self::proportion((4, 3), (4, 3), (4, 2), (4, 3), true)),
            ),
            Self::Sentinel => (
                GarmentEnsemble::new(TunicStyle::HeraldicTabard, VambraceStyle::StuddedVambraces, GirdleStyle::GarrisonBelt, FootwearStyle::GarrisonBoots),
                IdentityTraits::new(CrownStyle::KettleHelm, PigmentSwatch::WoadIndigo, ComplexionSwatch::Bronze, Self::proportion((4, 3), (5, 3), (4, 3), (5, 3), true)),
            ),
            Self::Noble => (
                GarmentEnsemble::new(TunicStyle::LinenDoublet, VambraceStyle::PuffedSleeves, GirdleStyle::KnottedCord, FootwearStyle::CuffedBoots),
                IdentityTraits::new(CrownStyle::SquireCrop, PigmentSwatch::MadderCrimson, ComplexionSwatch::Porcelain, Self::proportion((4, 2), (4, 2), (4, 2), (5, 2), true)),
            ),
        };
        YouthAttire::new(garments, identity, HeldHands::empty())
    }

    pub fn attire(self) -> YouthAttire {
        let (garments, identity) = match self {
            Self::Knight => (
                GarmentEnsemble::new(TunicStyle::PlateCuirass, VambraceStyle::PlateGauntlets, GirdleStyle::KnightlyBelt, FootwearStyle::PlateGreaves),
                IdentityTraits::new(CrownStyle::VisoredHelm, PigmentSwatch::MadderCrimson, ComplexionSwatch::Porcelain, Self::proportion((4, 5), (5, 6), (5, 6), (5, 5), false)),
            ),
            Self::Ranger => (
                GarmentEnsemble::new(TunicStyle::PeasantShirt, VambraceStyle::RawhideWraps, GirdleStyle::PouchedGirdle, FootwearStyle::Turnshoes),
                IdentityTraits::new(CrownStyle::MedievalGugel, PigmentSwatch::ForestMoss, ComplexionSwatch::Meadow, Self::proportion((4, 4), (4, 3), (5, 4), (6, 4), false)),
            ),
            Self::Woodsman => (
                GarmentEnsemble::new(TunicStyle::QuiltedJerkin, VambraceStyle::BareForearms, GirdleStyle::UnbeltedWaist, FootwearStyle::LacedBoots),
                IdentityTraits::new(CrownStyle::MerchantCap, PigmentSwatch::RussetBark, ComplexionSwatch::Amber, Self::proportion((4, 6), (5, 7), (5, 7), (4, 6), false)),
            ),
            Self::Blacksmith => (
                GarmentEnsemble::new(TunicStyle::ForgeApron, VambraceStyle::ForgeGloves, GirdleStyle::ForgeBelt, FootwearStyle::ForgeBoots),
                IdentityTraits::new(CrownStyle::SmithGoggles, PigmentSwatch::CharcoalSlate, ComplexionSwatch::Umber, Self::proportion((4, 6), (5, 8), (5, 8), (4, 7), false)),
            ),
            Self::Sentinel => (
                GarmentEnsemble::new(TunicStyle::HeraldicTabard, VambraceStyle::StuddedVambraces, GirdleStyle::GarrisonBelt, FootwearStyle::GarrisonBoots),
                IdentityTraits::new(CrownStyle::KettleHelm, PigmentSwatch::OchreGold, ComplexionSwatch::Bronze, Self::proportion((4, 6), (4, 7), (4, 6), (3, 7), false)),
            ),
            Self::Noble => (
                GarmentEnsemble::new(TunicStyle::LinenDoublet, VambraceStyle::PuffedSleeves, GirdleStyle::KnottedCord, FootwearStyle::CuffedBoots),
                IdentityTraits::new(CrownStyle::SquireCrop, PigmentSwatch::WoadIndigo, ComplexionSwatch::Birch, Self::proportion((4, 4), (5, 5), (4, 5), (5, 4), false)),
            ),
        };
        YouthAttire::new(garments, identity, HeldHands::empty())
    }
}
