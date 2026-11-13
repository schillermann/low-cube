mod mesh;
mod plinth;

use world::{BlockMedium, BlockMold};
pub use super::rhythm::MartialStyle;
pub use mesh::RelicFigure;

/// Object held in the avatar's right or left hand visible in first-person and multiplayer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldRelic {
    EmptyPalm,
    QuarriedStone,
    TimberTorch,
    WoodenMallet,
    WoodmanAxe,
    KnightlySword,
    IronSpade,
    ExcavatedEarth,
    DeadwoodBough,
    FlintStone,
    TimberBlock,
    PlankBlock,
    CrustBlock,
    CantileverCrane,
    SteeringWheel,
    AircraftEngine,
    RocketEngine,
    CabinSeat,
    CarWheel,
    FieldCamera,
    PedalCluster,
    CrossAxle,
}

impl HeldRelic {
    #[cfg(test)]
    pub fn next(self) -> Self {
        Self::from_code((self.code() + 1) % 22)
    }

    pub const fn from_code(code: u8) -> Self {
        match code % 22 {
            0 => Self::EmptyPalm,
            1 => Self::QuarriedStone,
            2 => Self::TimberTorch,
            3 => Self::WoodenMallet,
            4 => Self::WoodmanAxe,
            5 => Self::KnightlySword,
            6 => Self::IronSpade,
            7 => Self::ExcavatedEarth,
            8 => Self::DeadwoodBough,
            9 => Self::FlintStone,
            10 => Self::TimberBlock,
            11 => Self::PlankBlock,
            12 => Self::CrustBlock,
            13 => Self::CantileverCrane,
            14 => Self::SteeringWheel,
            15 => Self::AircraftEngine,
            16 => Self::RocketEngine,
            17 => Self::CabinSeat,
            18 => Self::CarWheel,
            19 => Self::FieldCamera,
            20 => Self::PedalCluster,
            _ => Self::CrossAxle,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::EmptyPalm => 0,
            Self::QuarriedStone => 1,
            Self::TimberTorch => 2,
            Self::WoodenMallet => 3,
            Self::WoodmanAxe => 4,
            Self::KnightlySword => 5,
            Self::IronSpade => 6,
            Self::ExcavatedEarth => 7,
            Self::DeadwoodBough => 8,
            Self::FlintStone => 9,
            Self::TimberBlock => 10,
            Self::PlankBlock => 11,
            Self::CrustBlock => 12,
            Self::CantileverCrane => 13,
            Self::SteeringWheel => 14,
            Self::AircraftEngine => 15,
            Self::RocketEngine => 16,
            Self::CabinSeat => 17,
            Self::CarWheel => 18,
            Self::FieldCamera => 19,
            Self::PedalCluster => 20,
            Self::CrossAxle => 21,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::EmptyPalm => "EMPTY SATCHEL",
            Self::QuarriedStone => "STONE BLOCK",
            Self::TimberTorch => "TIMBER TORCH",
            Self::WoodenMallet => "WOODEN MALLET",
            Self::WoodmanAxe => "WOODMAN AXE",
            Self::KnightlySword => "KNIGHT SWORD",
            Self::IronSpade => "IRON SPADE",
            Self::ExcavatedEarth => "EARTH BLOCK",
            Self::DeadwoodBough => "DEADWOOD",
            Self::FlintStone => "FLINT STONE",
            Self::TimberBlock => "WOOD BLOCK",
            Self::PlankBlock => "WOOD PLANKS",
            Self::CrustBlock => "CRUST BLOCK",
            Self::CantileverCrane => "CRANE LIFT",
            Self::SteeringWheel => "STEERING WHEEL",
            Self::AircraftEngine => "PLANE ENGINE",
            Self::RocketEngine => "ROCKET ENGINE",
            Self::CabinSeat => "SEAT BLOCK",
            Self::CarWheel => "CAR WHEEL",
            Self::FieldCamera => "FIELD CAMERA",
            Self::PedalCluster => "PEDAL BLOCK",
            Self::CrossAxle => "CROSS AXLE",
        }
    }

    pub fn parse_token(&self, token: &str) -> Option<Self> {
        let key: String = token
            .trim()
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match key.as_str() {
            "camera" | "fieldcamera" => Some(Self::FieldCamera),
            "stoneblock" | "quarriedstone" | "stone" => Some(Self::QuarriedStone),
            "timbertorch" | "torch" => Some(Self::TimberTorch),
            "woodenmallet" | "mallet" => Some(Self::WoodenMallet),
            "woodmanaxe" | "axe" => Some(Self::WoodmanAxe),
            "knightlysword" | "knightsword" | "sword" => Some(Self::KnightlySword),
            "ironspade" | "spade" => Some(Self::IronSpade),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(Self::ExcavatedEarth),
            "deadwood" | "deadwoodbough" => Some(Self::DeadwoodBough),
            "flintstone" | "flint" => Some(Self::FlintStone),
            "woodblock" | "timberblock" | "timber" => Some(Self::TimberBlock),
            "woodplanks" | "plankblock" | "planks" => Some(Self::PlankBlock),
            "crustblock" | "crust" => Some(Self::CrustBlock),
            "cranelift" | "cantilevercrane" | "crane" => Some(Self::CantileverCrane),
            "steeringwheel" | "steering" => Some(Self::SteeringWheel),
            "planeengine" | "aircraftengine" => Some(Self::AircraftEngine),
            "rocketengine" | "rocket" => Some(Self::RocketEngine),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(Self::CabinSeat),
            "carwheel" | "wheel" => Some(Self::CarWheel),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(Self::PedalCluster),
            "crossaxle" | "axle" | "axleblock" | "querachse" => Some(Self::CrossAxle),
            _ => None,
        }
    }

    pub const fn gripped(self) -> bool {
        !matches!(self, Self::EmptyPalm)
    }

    pub const fn free_palm(self) -> bool {
        matches!(self, Self::EmptyPalm)
    }

    pub const fn field_camera(self) -> bool {
        matches!(self, Self::FieldCamera)
    }

    pub const fn rotatable_in_palm(self) -> bool {
        self.placeable_block() || self.field_camera()
    }

    pub const fn turnable_in_palm(self) -> bool {
        self.placeable_block() && self.block_medium().mechanical_part()
    }

    pub const fn blocks_perch(self) -> bool {
        self.placeable_block() && !self.field_camera()
    }

    pub const fn spade_blade(self) -> bool {
        matches!(self, Self::IronSpade)
    }

    pub const fn axe_blade(self) -> bool {
        matches!(self, Self::WoodmanAxe | Self::WoodenMallet)
    }

    pub const fn placeable_block(self) -> bool {
        matches!(
            self,
            Self::ExcavatedEarth
                | Self::QuarriedStone
                | Self::TimberBlock
                | Self::PlankBlock
                | Self::CrustBlock
                | Self::CantileverCrane
                | Self::SteeringWheel
                | Self::AircraftEngine
                | Self::RocketEngine
                | Self::CabinSeat
                | Self::CarWheel
                | Self::FieldCamera
                | Self::PedalCluster
                | Self::CrossAxle
        )
    }

    pub const fn block_medium(self) -> BlockMedium {
        match self {
            Self::CrustBlock => BlockMedium::CrustRock,
            Self::QuarriedStone => BlockMedium::QuarriedStone,
            Self::TimberBlock => BlockMedium::RawTimber,
            Self::PlankBlock => BlockMedium::PlanedPlanks,
            Self::CantileverCrane => BlockMedium::CantileverRig,
            Self::SteeringWheel => BlockMedium::SteeringWheel,
            Self::AircraftEngine => BlockMedium::AircraftEngine,
            Self::RocketEngine => BlockMedium::RocketEngine,
            Self::CabinSeat => BlockMedium::CabinSeat,
            Self::CarWheel => BlockMedium::CarWheel,
            Self::FieldCamera => BlockMedium::FieldCamera,
            Self::PedalCluster => BlockMedium::PedalCluster,
            Self::CrossAxle => BlockMedium::CrossAxle,
            _ => BlockMedium::TurfEarth,
        }
    }

    pub const fn from_medium(medium: BlockMedium) -> Self {
        match medium {
            BlockMedium::TurfEarth | BlockMedium::Sand => Self::ExcavatedEarth,
            BlockMedium::CrustRock => Self::CrustBlock,
            BlockMedium::QuarriedStone => Self::QuarriedStone,
            BlockMedium::RawTimber => Self::TimberBlock,
            BlockMedium::PlanedPlanks => Self::PlankBlock,
            BlockMedium::CantileverRig => Self::CantileverCrane,
            BlockMedium::SteeringWheel => Self::SteeringWheel,
            BlockMedium::AircraftEngine => Self::AircraftEngine,
            BlockMedium::RocketEngine => Self::RocketEngine,
            BlockMedium::CabinSeat => Self::CabinSeat,
            BlockMedium::CarWheel => Self::CarWheel,
            BlockMedium::FieldCamera => Self::FieldCamera,
            BlockMedium::PedalCluster => Self::PedalCluster,
            BlockMedium::CrossAxle => Self::CrossAxle,
        }
    }

    pub const fn martial_style(self) -> MartialStyle {
        match self {
            Self::EmptyPalm => MartialStyle::Unarmed,
            Self::ExcavatedEarth
            | Self::CrustBlock
            | Self::QuarriedStone
            | Self::TimberBlock
            | Self::PlankBlock
            | Self::CantileverCrane
            | Self::SteeringWheel
            | Self::AircraftEngine
            | Self::RocketEngine
            | Self::CabinSeat
            | Self::CarWheel
            | Self::DeadwoodBough
            | Self::FlintStone
            | Self::FieldCamera
            | Self::PedalCluster
            | Self::CrossAxle => MartialStyle::PalmCradle,
            Self::IronSpade => MartialStyle::SpadeDig,
            Self::WoodenMallet | Self::WoodmanAxe => MartialStyle::LumberjackCleave,
            Self::TimberTorch | Self::KnightlySword => MartialStyle::SwordplaySlash,
        }
    }

    pub const fn figure(self, mold: BlockMold) -> RelicFigure {
        RelicFigure::new(self, mold)
    }
}
