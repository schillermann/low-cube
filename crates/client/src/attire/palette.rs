use glam::Vec3;

/// Natural skin tone swatches for the avatar hands, forearms, neck, and face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComplexionSwatch {
    Birch,
    Meadow,
    Amber,
    Umber,
}

impl ComplexionSwatch {
    pub fn next(self) -> Self {
        match self {
            Self::Birch => Self::Meadow,
            Self::Meadow => Self::Amber,
            Self::Amber => Self::Umber,
            Self::Umber => Self::Birch,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 4 {
            0 => Self::Birch,
            1 => Self::Meadow,
            2 => Self::Amber,
            _ => Self::Umber,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::Birch => 0,
            Self::Meadow => 1,
            Self::Amber => 2,
            Self::Umber => 3,
        }
    }

    pub fn skin(self) -> Vec3 {
        match self {
            Self::Birch => Vec3::new(0.92, 0.76, 0.64),
            Self::Meadow => Vec3::new(0.84, 0.66, 0.52),
            Self::Amber => Vec3::new(0.72, 0.53, 0.39),
            Self::Umber => Vec3::new(0.48, 0.33, 0.24),
        }
    }
}

/// Medieval natural dye swatches for tunics, jerkins, hose, and hoods.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PigmentSwatch {
    UnbleachedLinen,
    ForestMoss,
    MadderCrimson,
    WoadIndigo,
    OchreGold,
    SaddleHide,
    CharcoalSlate,
    RussetBark,
}

impl PigmentSwatch {
    pub fn next(self) -> Self {
        match self {
            Self::UnbleachedLinen => Self::ForestMoss,
            Self::ForestMoss => Self::MadderCrimson,
            Self::MadderCrimson => Self::WoadIndigo,
            Self::WoadIndigo => Self::OchreGold,
            Self::OchreGold => Self::SaddleHide,
            Self::SaddleHide => Self::CharcoalSlate,
            Self::CharcoalSlate => Self::RussetBark,
            Self::RussetBark => Self::UnbleachedLinen,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 8 {
            0 => Self::UnbleachedLinen,
            1 => Self::ForestMoss,
            2 => Self::MadderCrimson,
            3 => Self::WoadIndigo,
            4 => Self::OchreGold,
            5 => Self::SaddleHide,
            6 => Self::CharcoalSlate,
            _ => Self::RussetBark,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::UnbleachedLinen => 0,
            Self::ForestMoss => 1,
            Self::MadderCrimson => 2,
            Self::WoadIndigo => 3,
            Self::OchreGold => 4,
            Self::SaddleHide => 5,
            Self::CharcoalSlate => 6,
            Self::RussetBark => 7,
        }
    }

    pub fn cloth(self) -> Vec3 {
        match self {
            Self::UnbleachedLinen => Vec3::new(0.82, 0.77, 0.68),
            Self::ForestMoss => Vec3::new(0.24, 0.42, 0.25),
            Self::MadderCrimson => Vec3::new(0.62, 0.20, 0.18),
            Self::WoadIndigo => Vec3::new(0.22, 0.34, 0.54),
            Self::OchreGold => Vec3::new(0.76, 0.58, 0.24),
            Self::SaddleHide => Vec3::new(0.52, 0.34, 0.20),
            Self::CharcoalSlate => Vec3::new(0.30, 0.32, 0.35),
            Self::RussetBark => Vec3::new(0.44, 0.26, 0.18),
        }
    }

    pub fn trim(self) -> Vec3 {
        self.cloth() * 0.72
    }

    pub fn chausses(self) -> Vec3 {
        match self {
            Self::UnbleachedLinen | Self::OchreGold => Vec3::new(0.62, 0.58, 0.50),
            Self::ForestMoss | Self::SaddleHide => Vec3::new(0.42, 0.45, 0.38),
            _ => Vec3::new(0.36, 0.38, 0.42),
        }
    }
}

/// Youthful stature build variations scaling shoulder and limb proportions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YouthPhysique {
    SlenderSquire,
    AgileScout,
    SturdyApprentice,
}

impl YouthPhysique {
    pub fn next(self) -> Self {
        match self {
            Self::SlenderSquire => Self::AgileScout,
            Self::AgileScout => Self::SturdyApprentice,
            Self::SturdyApprentice => Self::SlenderSquire,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 3 {
            0 => Self::SlenderSquire,
            1 => Self::AgileScout,
            _ => Self::SturdyApprentice,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::SlenderSquire => 0,
            Self::AgileScout => 1,
            Self::SturdyApprentice => 2,
        }
    }

    /// Scales a lateral anatomical measure according to this youthful physique ("Tell, Don't Ask").
    pub fn scale(self, measure: f32) -> f32 {
        let factor = match self {
            Self::SlenderSquire => 0.94,
            Self::AgileScout => 1.0,
            Self::SturdyApprentice => 1.08,
        };
        measure * factor
    }
}
