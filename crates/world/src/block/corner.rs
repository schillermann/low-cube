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
            Self::NorthEast => (u + v - 1.0).max(0.0),
            Self::NorthWest => (v - u).max(0.0),
            Self::SouthEast => (u - v).max(0.0),
            Self::SouthWest => (1.0 - u - v).max(0.0),
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

    pub fn slope_elevation(&self, u: f32, v: f32) -> f32 {
        match self {
            Self::NorthEast => (u + v) * 0.25,
            Self::NorthWest => (1.0 - u + v) * 0.25,
            Self::SouthEast => (u + 1.0 - v) * 0.25,
            Self::SouthWest => (2.0 - u - v) * 0.25,
        }
    }

    pub fn corner_heights(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::NorthEast => (0.0, 0.25, 0.5, 0.25),
            Self::NorthWest => (0.25, 0.0, 0.25, 0.5),
            Self::SouthEast => (0.25, 0.5, 0.25, 0.0),
            Self::SouthWest => (0.5, 0.25, 0.0, 0.25),
        }
    }
}
