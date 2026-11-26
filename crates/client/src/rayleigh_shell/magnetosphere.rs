use glam::Vec3;

/// Focused contract for planetary magnetic dipole emission bands.
pub trait MagnetosphereEnvelope: Copy {
    fn unmagnetized(&self) -> bool;
    fn transfer_bands<R>(&self, sink: impl FnOnce([f32; 4], [f32; 4]) -> R) -> R;
}

/// Polymorphic planetary magnetosphere state (`Unmagnetized` Null Object vs. `PolarDipole`, `<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Magnetosphere {
    Unmagnetized,
    PolarDipole {
        lower_tint: Vec3,
        upper_tint: Vec3,
        dynamics: (f32, f32),
    },
}

impl Magnetosphere {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn polar_dipole(lower_tint: Vec3, upper_tint: Vec3, dynamics: (f32, f32)) -> Self {
        Self::PolarDipole { lower_tint, upper_tint, dynamics }
    }

    /// Null-object constructor for airless worlds and unmagnetized atmospheres (`Luna`, `Mercury`, `Venus`).
    pub const fn void() -> Self {
        Self::Unmagnetized
    }

    /// Terrestrial `Arctic (-X)` oxygen-nitrogen dipole (`557.7 nm` emerald base to `630.0 nm` violet-crimson crown).
    pub const fn terrestrial_arctic() -> Self {
        Self::polar_dipole(
            Vec3::new(0.06, 0.98, 0.46),
            Vec3::new(0.68, 0.16, 0.90),
            (1.0, 0.74),
        )
    }

    /// Saturn-induced `Titan` methane-nitrogen magnetosphere (deep indigo-cyan base to amber-magenta crown).
    pub const fn saturnian_methane() -> Self {
        Self::polar_dipole(
            Vec3::new(0.16, 0.38, 0.96),
            Vec3::new(0.90, 0.44, 0.72),
            (0.78, 0.56),
        )
    }

    /// Martian localized crustal magnetic wisps (subtle cyan-ultraviolet upper-air glow).
    pub const fn crustal_wisps() -> Self {
        Self::polar_dipole(
            Vec3::new(0.14, 0.78, 0.88),
            Vec3::new(0.46, 0.24, 0.92),
            (0.36, 0.35),
        )
    }

    /// Decodes uniform pair `(lower_and_strength, upper_and_flare)` back into `Magnetosphere`.
    pub fn from_bands(lower: [f32; 4], upper: [f32; 4]) -> Self {
        if lower[3] <= 1e-4 {
            Self::void()
        } else {
            Self::polar_dipole(
                Vec3::new(lower[0], lower[1], lower[2]),
                Vec3::new(upper[0], upper[1], upper[2]),
                (lower[3], upper[3]),
            )
        }
    }
}

impl MagnetosphereEnvelope for Magnetosphere {
    fn unmagnetized(&self) -> bool {
        matches!(self, Self::Unmagnetized)
    }

    fn transfer_bands<R>(&self, sink: impl FnOnce([f32; 4], [f32; 4]) -> R) -> R {
        match *self {
            Self::Unmagnetized => sink([0.0; 4], [0.0; 4]),
            Self::PolarDipole { lower_tint, upper_tint, dynamics } => sink(
                [lower_tint.x, lower_tint.y, lower_tint.z, dynamics.0],
                [upper_tint.x, upper_tint.y, upper_tint.z, dynamics.1],
            ),
        }
    }
}
