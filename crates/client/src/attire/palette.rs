use glam::Vec3;

/// Natural skin tone swatches for the crew avatar face and brow visible inside clear EVA visors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComplexionSwatch {
    Birch,
    Meadow,
    Amber,
    Umber,
    Porcelain,
    Olive,
    Bronze,
    Ebony,
}

impl ComplexionSwatch {
    pub fn all() -> [Self; 8] {
        [
            Self::Birch,
            Self::Meadow,
            Self::Amber,
            Self::Umber,
            Self::Porcelain,
            Self::Olive,
            Self::Bronze,
            Self::Ebony,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Birch => "BIRCH",
            Self::Meadow => "MEADOW",
            Self::Amber => "AMBER",
            Self::Umber => "UMBER",
            Self::Porcelain => "PORCELAIN",
            Self::Olive => "OLIVE",
            Self::Bronze => "BRONZE",
            Self::Ebony => "EBONY",
        }
    }

    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 8 {
            0 => Self::Birch,
            1 => Self::Meadow,
            2 => Self::Amber,
            3 => Self::Umber,
            4 => Self::Porcelain,
            5 => Self::Olive,
            6 => Self::Bronze,
            _ => Self::Ebony,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::Birch => 0,
            Self::Meadow => 1,
            Self::Amber => 2,
            Self::Umber => 3,
            Self::Porcelain => 4,
            Self::Olive => 5,
            Self::Bronze => 6,
            Self::Ebony => 7,
        }
    }

    pub fn skin(self) -> Vec3 {
        match self {
            Self::Birch => Vec3::new(0.92, 0.76, 0.64),
            Self::Meadow => Vec3::new(0.84, 0.66, 0.52),
            Self::Amber => Vec3::new(0.72, 0.53, 0.39),
            Self::Umber => Vec3::new(0.48, 0.33, 0.24),
            Self::Porcelain => Vec3::new(0.96, 0.85, 0.76),
            Self::Olive => Vec3::new(0.78, 0.63, 0.46),
            Self::Bronze => Vec3::new(0.61, 0.42, 0.28),
            Self::Ebony => Vec3::new(0.32, 0.21, 0.15),
        }
    }
}

/// Space-Age EVA pressure-suit outer beta-cloth and anodized alloy pigment swatches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PigmentSwatch {
    LunarWhite,
    OrbitalOlive,
    SignalCrimson,
    DeepCobalt,
    SolarGold,
    RegolithTan,
    CarbonSlate,
    MartianRust,
}

impl PigmentSwatch {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 8 {
            0 => Self::LunarWhite,
            1 => Self::OrbitalOlive,
            2 => Self::SignalCrimson,
            3 => Self::DeepCobalt,
            4 => Self::SolarGold,
            5 => Self::RegolithTan,
            6 => Self::CarbonSlate,
            _ => Self::MartianRust,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::LunarWhite => 0,
            Self::OrbitalOlive => 1,
            Self::SignalCrimson => 2,
            Self::DeepCobalt => 3,
            Self::SolarGold => 4,
            Self::RegolithTan => 5,
            Self::CarbonSlate => 6,
            Self::MartianRust => 7,
        }
    }

    pub fn cloth(self) -> Vec3 {
        match self {
            Self::LunarWhite => Vec3::new(0.86, 0.88, 0.92),
            Self::OrbitalOlive => Vec3::new(0.32, 0.44, 0.34),
            Self::SignalCrimson => Vec3::new(0.74, 0.22, 0.18),
            Self::DeepCobalt => Vec3::new(0.20, 0.38, 0.64),
            Self::SolarGold => Vec3::new(0.84, 0.64, 0.22),
            Self::RegolithTan => Vec3::new(0.62, 0.54, 0.44),
            Self::CarbonSlate => Vec3::new(0.26, 0.30, 0.36),
            Self::MartianRust => Vec3::new(0.64, 0.30, 0.20),
        }
    }

    pub fn trim(self) -> Vec3 {
        match self {
            Self::LunarWhite => Vec3::new(0.22, 0.56, 0.84),
            Self::SignalCrimson => Vec3::new(0.88, 0.84, 0.78),
            _ => self.cloth() * 0.72,
        }
    }

    pub fn trousers(self) -> Vec3 {
        match self {
            Self::LunarWhite | Self::SolarGold => Vec3::new(0.68, 0.72, 0.78),
            Self::OrbitalOlive | Self::RegolithTan => Vec3::new(0.40, 0.44, 0.42),
            _ => Vec3::new(0.32, 0.36, 0.42),
        }
    }
}
