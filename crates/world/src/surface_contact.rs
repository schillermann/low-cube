use glam::Vec3;

/// Polymorphic planetary surface contact outcome avoiding dummy fallback coordinates in vacuum or gas giants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurfaceContact {
    Vacuum,
    GasEnvelope { drag: f32 },
    SolidCrust { ground: Vec3, normal: Vec3 },
}

impl SurfaceContact {
    /// Indicates whether this probe reached a landable solid planetary, lunar, or asteroidal crust.
    pub const fn landable(self) -> bool {
        matches!(self, Self::SolidCrust { .. })
    }

    /// Indicates whether this probe entered a non-solid gas giant cloud deck.
    pub const fn gaseous(self) -> bool {
        matches!(self, Self::GasEnvelope { .. })
    }

    /// Shifts a solid crust contact point by `offset` while leaving vacuum and gas envelopes unchanged.
    pub fn translated(self, offset: Vec3) -> Self {
        match self {
            Self::Vacuum => Self::Vacuum,
            Self::GasEnvelope { drag } => Self::GasEnvelope { drag },
            Self::SolidCrust { ground, normal } => Self::SolidCrust {
                ground: ground + offset,
                normal,
            },
        }
    }

    /// Dispatches crust `(ground, normal)` to `on_crust` or returns `fallback` in vacuum and gas envelopes.
    pub fn resolve_crust<R>(self, fallback: R, on_crust: impl FnOnce(Vec3, Vec3) -> R) -> R {
        match self {
            Self::Vacuum | Self::GasEnvelope { .. } => fallback,
            Self::SolidCrust { ground, normal } => on_crust(ground, normal),
        }
    }
}
