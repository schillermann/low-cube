use glam::Vec3;

/// Four-way 90-degree horizontal rotation relative to the placement sight direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardinalTurn {
    Forward,
    Right,
    Backward,
    Left,
}

impl CardinalTurn {
    pub const fn turn_clockwise(self) -> Self {
        match self {
            Self::Forward => Self::Right,
            Self::Right => Self::Backward,
            Self::Backward => Self::Left,
            Self::Left => Self::Forward,
        }
    }

    pub const fn turn_counter_clockwise(self) -> Self {
        match self {
            Self::Forward => Self::Left,
            Self::Right => Self::Forward,
            Self::Backward => Self::Right,
            Self::Left => Self::Backward,
        }
    }

    /// Rotates a 3D look direction around the surface normal in 90-degree increments.
    pub fn orient_vector(self, direction: Vec3, surface_up: Vec3) -> Vec3 {
        let flat = direction - surface_up * direction.dot(surface_up);
        let fwd = if flat.length_squared() > 1e-6 { flat.normalize() } else { Vec3::NEG_Z };
        let right = fwd.cross(surface_up).normalize_or_zero();
        match self {
            Self::Forward => fwd,
            Self::Right => right,
            Self::Backward => -fwd,
            Self::Left => -right,
        }
    }
}

/// Complete catalog of all 16 sculptable voxel form archetypes available for palm molding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoldArchetype {
    FullCube,
    HalfSlab,
    RampWedge,
    SteppedStair,
    PyramidApex,
    ConvexRidge,
    ConcaveValley,
    DiagonalRamp,
    GentleLowerWedge,
    GentleUpperWedge,
    GentleLowerRidge,
    GentleUpperRidge,
    GentleLowerValley,
    GentleUpperValley,
    ConvexStairCorner,
    ConcaveStairCorner,
}

impl MoldArchetype {
    pub const fn from_code(code: u8) -> Self {
        match code % 16 {
            0 => Self::FullCube,
            1 => Self::HalfSlab,
            2 => Self::RampWedge,
            3 => Self::SteppedStair,
            4 => Self::PyramidApex,
            5 => Self::ConvexRidge,
            6 => Self::ConcaveValley,
            7 => Self::DiagonalRamp,
            8 => Self::GentleLowerWedge,
            9 => Self::GentleUpperWedge,
            10 => Self::GentleLowerRidge,
            11 => Self::GentleUpperRidge,
            12 => Self::GentleLowerValley,
            13 => Self::GentleUpperValley,
            14 => Self::ConvexStairCorner,
            _ => Self::ConcaveStairCorner,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::FullCube => 0,
            Self::HalfSlab => 1,
            Self::RampWedge => 2,
            Self::SteppedStair => 3,
            Self::PyramidApex => 4,
            Self::ConvexRidge => 5,
            Self::ConcaveValley => 6,
            Self::DiagonalRamp => 7,
            Self::GentleLowerWedge => 8,
            Self::GentleUpperWedge => 9,
            Self::GentleLowerRidge => 10,
            Self::GentleUpperRidge => 11,
            Self::GentleLowerValley => 12,
            Self::GentleUpperValley => 13,
            Self::ConvexStairCorner => 14,
            Self::ConcaveStairCorner => 15,
        }
    }

    pub const fn next(self) -> Self { Self::from_code(self.code() + 1) }
    pub const fn prev(self) -> Self { Self::from_code(self.code() + 15) }
    pub const fn full_height_flat(self) -> bool { matches!(self, Self::FullCube) }
    pub const fn flat_square_top(self) -> bool { matches!(self, Self::FullCube | Self::HalfSlab) }

    pub const fn next_flat_square_top(self) -> Self {
        match self {
            Self::FullCube => Self::HalfSlab,
            _ => Self::FullCube,
        }
    }

    pub const fn next_camera_turn(self) -> Self { Self::from_code((self.code() + 1) % 8) }
    pub const fn prev_camera_turn(self) -> Self { Self::from_code((self.code() + 7) % 8) }

    pub const fn placed_camera_yaw(self) -> f32 {
        match self.code() % 8 {
            0 => 0.0,
            1 => -std::f32::consts::FRAC_PI_4,
            2 => -std::f32::consts::FRAC_PI_2,
            3 => -3.0 * std::f32::consts::FRAC_PI_4,
            4 => std::f32::consts::PI,
            5 => 3.0 * std::f32::consts::FRAC_PI_4,
            6 => std::f32::consts::FRAC_PI_2,
            _ => std::f32::consts::FRAC_PI_4,
        }
    }
}
