/// Cardinal diagonal heading defining corner orientation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CornerHeading {
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl CornerHeading {
    pub fn ridge_elevation(&self, u: f32, v: f32) -> f32 {
        match self {
            Self::NorthEast => u.min(v),
            Self::NorthWest => (1.0 - u).min(v),
            Self::SouthEast => u.min(1.0 - v),
            Self::SouthWest => (1.0 - u).min(1.0 - v),
        }
    }

    pub fn valley_elevation(&self, u: f32, v: f32) -> f32 {
        match self {
            Self::NorthEast => u.max(v),
            Self::NorthWest => (1.0 - u).max(v),
            Self::SouthEast => u.max(1.0 - v),
            Self::SouthWest => (1.0 - u).max(1.0 - v),
        }
    }
}
