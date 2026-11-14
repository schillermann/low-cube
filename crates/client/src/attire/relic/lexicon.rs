use world::{BlockMedium, SurfaceFind};
use super::{HeldRelic, MartialStyle};

/// Domain lexicon and property catalog for `HeldRelic` items.
pub struct RelicLexicon {
    relic: HeldRelic,
}

impl RelicLexicon {
    pub const TRAY_ORDER: [HeldRelic; HeldRelic::COUNT - 1] = [
        HeldRelic::GoldOre,
        HeldRelic::IronOre,
        HeldRelic::CopperOre,
        HeldRelic::CabinSeat,
        HeldRelic::PedalCluster,
        HeldRelic::CrossAxle,
        HeldRelic::CarWheel,
        HeldRelic::SteeringWheel,
        HeldRelic::AircraftEngine,
        HeldRelic::RocketEngine,
        HeldRelic::ExcavatedEarth,
        HeldRelic::CrustBlock,
        HeldRelic::QuarriedStone,
        HeldRelic::TimberBlock,
        HeldRelic::PlankBlock,
        HeldRelic::BoarMeat,
        HeldRelic::DeadwoodBough,
        HeldRelic::FlintStone,
        HeldRelic::FieldCamera,
        HeldRelic::CantileverCrane,
        HeldRelic::IronSpade,
        HeldRelic::WoodmanAxe,
        HeldRelic::KnightlySword,
        HeldRelic::WoodenMallet,
        HeldRelic::TimberTorch,
    ];

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic) -> Self {
        Self { relic }
    }

    pub const fn stackable_in_tray(&self) -> bool {
        !matches!(
            self.relic,
            HeldRelic::EmptyPalm
                | HeldRelic::FieldCamera
                | HeldRelic::CantileverCrane
                | HeldRelic::IronSpade
                | HeldRelic::WoodmanAxe
                | HeldRelic::KnightlySword
                | HeldRelic::WoodenMallet
                | HeldRelic::TimberTorch
        )
    }

    pub const fn title(&self) -> &'static str {
        match self.relic {
            HeldRelic::EmptyPalm => "EMPTY SATCHEL",
            HeldRelic::QuarriedStone => "STONE BLOCK",
            HeldRelic::TimberTorch => "TIMBER TORCH",
            HeldRelic::WoodenMallet => "WOODEN MALLET",
            HeldRelic::WoodmanAxe => "WOODMAN AXE",
            HeldRelic::KnightlySword => "KNIGHT SWORD",
            HeldRelic::IronSpade => "IRON SPADE",
            HeldRelic::ExcavatedEarth => "EARTH BLOCK",
            HeldRelic::DeadwoodBough => "DEADWOOD",
            HeldRelic::FlintStone => "FLINT STONE",
            HeldRelic::TimberBlock => "WOOD BLOCK",
            HeldRelic::PlankBlock => "WOOD PLANKS",
            HeldRelic::CrustBlock => "CRUST BLOCK",
            HeldRelic::CantileverCrane => "CRANE LIFT",
            HeldRelic::SteeringWheel => "STEERING WHEEL",
            HeldRelic::AircraftEngine => "PLANE ENGINE",
            HeldRelic::RocketEngine => "ROCKET ENGINE",
            HeldRelic::CabinSeat => "SEAT BLOCK",
            HeldRelic::CarWheel => "CAR WHEEL",
            HeldRelic::FieldCamera => "FIELD CAMERA",
            HeldRelic::PedalCluster => "PEDAL BLOCK",
            HeldRelic::CrossAxle => "CROSS AXLE",
            HeldRelic::CopperOre => "COPPER ORE",
            HeldRelic::IronOre => "IRON ORE",
            HeldRelic::GoldOre => "GOLD ORE",
            HeldRelic::BoarMeat => "BOAR MEAT",
        }
    }

    pub fn parse(&self, token: &str) -> Option<HeldRelic> {
        let key: String = token
            .trim()
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match key.as_str() {
            "camera" | "fieldcamera" => Some(HeldRelic::FieldCamera),
            "stoneblock" | "quarriedstone" | "stone" => Some(HeldRelic::QuarriedStone),
            "timbertorch" | "torch" => Some(HeldRelic::TimberTorch),
            "woodenmallet" | "mallet" => Some(HeldRelic::WoodenMallet),
            "woodmanaxe" | "axe" => Some(HeldRelic::WoodmanAxe),
            "knightlysword" | "knightsword" | "sword" => Some(HeldRelic::KnightlySword),
            "ironspade" | "spade" => Some(HeldRelic::IronSpade),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(HeldRelic::ExcavatedEarth),
            "deadwood" | "deadwoodbough" => Some(HeldRelic::DeadwoodBough),
            "flintstone" | "flint" => Some(HeldRelic::FlintStone),
            "woodblock" | "timberblock" | "timber" => Some(HeldRelic::TimberBlock),
            "woodplanks" | "plankblock" | "planks" => Some(HeldRelic::PlankBlock),
            "crustblock" | "crust" => Some(HeldRelic::CrustBlock),
            "cranelift" | "cantilevercrane" | "crane" => Some(HeldRelic::CantileverCrane),
            "steeringwheel" | "steering" => Some(HeldRelic::SteeringWheel),
            "planeengine" | "aircraftengine" => Some(HeldRelic::AircraftEngine),
            "rocketengine" | "rocket" => Some(HeldRelic::RocketEngine),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(HeldRelic::CabinSeat),
            "carwheel" | "wheel" => Some(HeldRelic::CarWheel),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(HeldRelic::PedalCluster),
            "crossaxle" | "axle" | "axleblock" | "querachse" => Some(HeldRelic::CrossAxle),
            "copperore" | "copper" | "kupfer" => Some(HeldRelic::CopperOre),
            "ironore" | "iron" | "eisen" => Some(HeldRelic::IronOre),
            "goldore" | "gold" => Some(HeldRelic::GoldOre),
            "boarmeat" | "meat" | "venison" | "fleisch" | "wildschwein" => Some(HeldRelic::BoarMeat),
            _ => None,
        }
    }

    pub const fn block_medium(&self) -> BlockMedium {
        match self.relic {
            HeldRelic::CrustBlock => BlockMedium::CrustRock,
            HeldRelic::QuarriedStone => BlockMedium::QuarriedStone,
            HeldRelic::TimberBlock => BlockMedium::RawTimber,
            HeldRelic::PlankBlock => BlockMedium::PlanedPlanks,
            HeldRelic::CantileverCrane => BlockMedium::CantileverRig,
            HeldRelic::SteeringWheel => BlockMedium::SteeringWheel,
            HeldRelic::AircraftEngine => BlockMedium::AircraftEngine,
            HeldRelic::RocketEngine => BlockMedium::RocketEngine,
            HeldRelic::CabinSeat => BlockMedium::CabinSeat,
            HeldRelic::CarWheel => BlockMedium::CarWheel,
            HeldRelic::FieldCamera => BlockMedium::FieldCamera,
            HeldRelic::PedalCluster => BlockMedium::PedalCluster,
            HeldRelic::CrossAxle => BlockMedium::CrossAxle,
            HeldRelic::CopperOre => BlockMedium::CopperOre,
            HeldRelic::IronOre => BlockMedium::IronOre,
            HeldRelic::GoldOre => BlockMedium::GoldOre,
            _ => BlockMedium::TurfEarth,
        }
    }

    pub const fn martial_style(&self) -> MartialStyle {
        match self.relic {
            HeldRelic::EmptyPalm => MartialStyle::Unarmed,
            HeldRelic::IronSpade => MartialStyle::SpadeDig,
            HeldRelic::WoodenMallet | HeldRelic::WoodmanAxe => MartialStyle::LumberjackCleave,
            HeldRelic::TimberTorch | HeldRelic::KnightlySword => MartialStyle::SwordplaySlash,
            _ => MartialStyle::PalmCradle,
        }
    }

    pub const fn surface_find(&self) -> SurfaceFind {
        match self.relic {
            HeldRelic::EmptyPalm => SurfaceFind::EmptyHanded,
            HeldRelic::DeadwoodBough => SurfaceFind::DeadwoodBough,
            HeldRelic::FlintStone => SurfaceFind::FlintStone,
            HeldRelic::TimberTorch => SurfaceFind::TimberTorch,
            HeldRelic::WoodenMallet => SurfaceFind::WoodenMallet,
            HeldRelic::WoodmanAxe => SurfaceFind::WoodmanAxe,
            HeldRelic::KnightlySword => SurfaceFind::KnightlySword,
            HeldRelic::IronSpade => SurfaceFind::IronSpade,
            HeldRelic::BoarMeat => SurfaceFind::BoarMeat,
            _ => SurfaceFind::LooseBlock(self.block_medium()),
        }
    }
}
