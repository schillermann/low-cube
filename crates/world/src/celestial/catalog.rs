use glam::Vec3;

/// Compact celestial body identifier addressing the Sun, 8 planets, dwarf planets, moons, and belt asteroids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BodyId {
    code: u16,
}

impl BodyId {
    pub const SOL: Self = Self::new(0);
    pub const MERCURY: Self = Self::new(1);
    pub const VENUS: Self = Self::new(2);
    pub const EARTH: Self = Self::new(3);
    pub const LUNA: Self = Self::new(4);
    pub const MARS: Self = Self::new(5);
    pub const PHOBOS: Self = Self::new(6);
    pub const DEIMOS: Self = Self::new(7);
    pub const CERES: Self = Self::new(8);
    pub const JUPITER: Self = Self::new(9);
    pub const IO: Self = Self::new(10);
    pub const EUROPA: Self = Self::new(11);
    pub const GANYMEDE: Self = Self::new(12);
    pub const CALLISTO: Self = Self::new(13);
    pub const SATURN: Self = Self::new(14);
    pub const ENCELADUS: Self = Self::new(15);
    pub const TITAN: Self = Self::new(16);
    pub const URANUS: Self = Self::new(17);
    pub const TITANIA: Self = Self::new(18);
    pub const NEPTUNE: Self = Self::new(19);
    pub const TRITON: Self = Self::new(20);

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: u16) -> Self {
        Self { code }
    }

    pub const fn belt_rock(sector_hash: u16) -> Self {
        Self::new(0x8000 | (sector_hash & 0x7FFF))
    }

    pub const fn raw_code(self) -> u16 {
        self.code
    }

    pub const fn belt_slot(self) -> Option<u16> {
        if (self.code & 0x8000) != 0 {
            Some(self.code & 0x7FFF)
        } else {
            None
        }
    }
}

/// Polymorphic physical nature of a celestial cube without runtime type-checking flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceNature {
    SolarStar,
    Homeworld,
    RockyWorld,
    GasGiant { ringed: bool },
}

impl SurfaceNature {
    pub const fn landable_crust(self) -> bool {
        matches!(self, Self::Homeworld | Self::RockyWorld)
    }

    pub const fn ringed_giant(self) -> bool {
        matches!(self, Self::GasGiant { ringed: true })
    }
}

/// Keplerian circular-inclined orbital elements (`AU` around `SOL`, parent-diameters around planets).
#[derive(Clone, Copy, Debug)]
pub struct OrbitElements {
    parent: BodyId,
    orbit_units: f64,
    tilt_and_phase: (f32, f32),
}

impl OrbitElements {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(parent: BodyId, orbit_units: f64, tilt_and_phase: (f32, f32)) -> Self {
        Self { parent, orbit_units, tilt_and_phase }
    }

    pub const fn parent(self) -> BodyId { self.parent }
    pub const fn orbit_units(self) -> f64 { self.orbit_units }
    pub const fn inclination_rad(self) -> f32 { self.tilt_and_phase.0 }
    pub const fn phase_rad(self) -> f32 { self.tilt_and_phase.1 }
}

/// Physical proportions, surface gravity, nature, and primary/secondary cube tones of a celestial body.
#[derive(Clone, Copy, Debug)]
pub struct BodyPhysique {
    diameter_ratio: f32,
    surface_g: f32,
    nature: SurfaceNature,
    palette: (Vec3, Vec3),
}

impl BodyPhysique {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        diameter_ratio: f32,
        surface_g: f32,
        nature: SurfaceNature,
        palette: (Vec3, Vec3),
    ) -> Self {
        Self { diameter_ratio, surface_g, nature, palette }
    }

    pub const fn diameter_ratio(self) -> f32 { self.diameter_ratio }
    pub const fn surface_g(self) -> f32 { self.surface_g }
    pub const fn nature(self) -> SurfaceNature { self.nature }
    pub const fn palette(self) -> (Vec3, Vec3) { self.palette }
}

/// Immutable specification of one major solar-system body (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct OrbitalSpec {
    identity: (BodyId, &'static str),
    orbit: OrbitElements,
    physique: BodyPhysique,
}

impl OrbitalSpec {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        identity: (BodyId, &'static str),
        orbit: OrbitElements,
        physique: BodyPhysique,
    ) -> Self {
        Self { identity, orbit, physique }
    }

    pub const fn id(self) -> BodyId { self.identity.0 }
    pub const fn name(self) -> &'static str { self.identity.1 }
    pub const fn orbit(self) -> OrbitElements { self.orbit }
    pub const fn physique(self) -> BodyPhysique { self.physique }
}

pub const MAJOR_BODY_COUNT: usize = 21;

/// Authoritative catalog of the cubic Sun, all 8 planets, Ceres, and 11 major moons.
pub struct SolarCatalog;

impl SolarCatalog {
    pub const fn new() -> Self { Self }

    pub fn entries(&self) -> &'static [OrbitalSpec; MAJOR_BODY_COUNT] {
        &CATALOG
    }

    pub fn spec_for(&self, id: BodyId) -> Option<OrbitalSpec> {
        CATALOG.iter().copied().find(|s| s.id() == id)
    }
}

const fn v3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

static CATALOG: [OrbitalSpec; MAJOR_BODY_COUNT] = [
    OrbitalSpec::new(
        (BodyId::SOL, "SOL"),
        OrbitElements::new(BodyId::SOL, 0.0, (0.0, 0.0)),
        BodyPhysique::new(109.2, 274.0, SurfaceNature::SolarStar, (v3(1.0, 0.92, 0.55), v3(1.0, 0.48, 0.12))),
    ),
    OrbitalSpec::new(
        (BodyId::MERCURY, "MERCURY"),
        OrbitElements::new(BodyId::SOL, 0.387, (0.12, 0.85)),
        BodyPhysique::new(0.383, 3.70, SurfaceNature::RockyWorld, (v3(0.58, 0.56, 0.54), v3(0.36, 0.34, 0.33))),
    ),
    OrbitalSpec::new(
        (BodyId::VENUS, "VENUS"),
        OrbitElements::new(BodyId::SOL, 0.723, (0.06, 2.10)),
        BodyPhysique::new(0.949, 8.87, SurfaceNature::RockyWorld, (v3(0.92, 0.78, 0.46), v3(0.72, 0.52, 0.24))),
    ),
    OrbitalSpec::new(
        (BodyId::EARTH, "EARTH"),
        OrbitElements::new(BodyId::SOL, 1.000, (0.41, -1.20)),
        BodyPhysique::new(1.000, 9.81, SurfaceNature::Homeworld, (v3(0.22, 0.56, 0.88), v3(0.26, 0.64, 0.28))),
    ),
    OrbitalSpec::new(
        (BodyId::LUNA, "LUNA"),
        OrbitElements::new(BodyId::EARTH, 30.1, (0.09, 1.25)),
        BodyPhysique::new(0.273, 1.62, SurfaceNature::RockyWorld, (v3(0.72, 0.74, 0.78), v3(0.42, 0.44, 0.49))),
    ),
    OrbitalSpec::new(
        (BodyId::MARS, "MARS"),
        OrbitElements::new(BodyId::SOL, 1.524, (0.03, 0.52)),
        BodyPhysique::new(0.532, 3.72, SurfaceNature::RockyWorld, (v3(0.82, 0.36, 0.20), v3(0.48, 0.20, 0.12))),
    ),
    OrbitalSpec::new(
        (BodyId::PHOBOS, "PHOBOS"),
        OrbitElements::new(BodyId::MARS, 2.8, (0.02, 0.40)),
        BodyPhysique::new(0.090, 0.55, SurfaceNature::RockyWorld, (v3(0.46, 0.40, 0.36), v3(0.28, 0.24, 0.22))),
    ),
    OrbitalSpec::new(
        (BodyId::DEIMOS, "DEIMOS"),
        OrbitElements::new(BodyId::MARS, 6.9, (0.04, 3.60)),
        BodyPhysique::new(0.075, 0.42, SurfaceNature::RockyWorld, (v3(0.52, 0.46, 0.40), v3(0.32, 0.28, 0.25))),
    ),
    OrbitalSpec::new(
        (BodyId::CERES, "CERES"),
        OrbitElements::new(BodyId::SOL, 2.768, (0.18, 1.95)),
        BodyPhysique::new(0.140, 0.85, SurfaceNature::RockyWorld, (v3(0.50, 0.50, 0.52), v3(0.30, 0.31, 0.34))),
    ),
    OrbitalSpec::new(
        (BodyId::JUPITER, "JUPITER"),
        OrbitElements::new(BodyId::SOL, 5.203, (0.02, 1.15)),
        BodyPhysique::new(10.97, 24.79, SurfaceNature::GasGiant { ringed: false }, (v3(0.86, 0.68, 0.48), v3(0.74, 0.38, 0.22))),
    ),
    OrbitalSpec::new(
        (BodyId::IO, "IO"),
        OrbitElements::new(BodyId::JUPITER, 3.2, (0.01, 0.30)),
        BodyPhysique::new(0.286, 1.80, SurfaceNature::RockyWorld, (v3(0.95, 0.82, 0.24), v3(0.82, 0.28, 0.12))),
    ),
    OrbitalSpec::new(
        (BodyId::EUROPA, "EUROPA"),
        OrbitElements::new(BodyId::JUPITER, 5.1, (0.02, 1.80)),
        BodyPhysique::new(0.245, 1.31, SurfaceNature::RockyWorld, (v3(0.84, 0.92, 0.98), v3(0.52, 0.68, 0.82))),
    ),
    OrbitalSpec::new(
        (BodyId::GANYMEDE, "GANYMEDE"),
        OrbitElements::new(BodyId::JUPITER, 8.2, (0.01, 3.10)),
        BodyPhysique::new(0.413, 1.43, SurfaceNature::RockyWorld, (v3(0.62, 0.60, 0.58), v3(0.38, 0.36, 0.35))),
    ),
    OrbitalSpec::new(
        (BodyId::CALLISTO, "CALLISTO"),
        OrbitElements::new(BodyId::JUPITER, 13.8, (0.01, 4.70)),
        BodyPhysique::new(0.378, 1.24, SurfaceNature::RockyWorld, (v3(0.42, 0.40, 0.44), v3(0.24, 0.23, 0.27))),
    ),
    OrbitalSpec::new(
        (BodyId::SATURN, "SATURN"),
        OrbitElements::new(BodyId::SOL, 9.537, (0.04, 2.55)),
        BodyPhysique::new(9.14, 10.44, SurfaceNature::GasGiant { ringed: true }, (v3(0.90, 0.80, 0.56), v3(0.72, 0.62, 0.42))),
    ),
    OrbitalSpec::new(
        (BodyId::ENCELADUS, "ENCELADUS"),
        OrbitElements::new(BodyId::SATURN, 3.4, (0.01, 0.90)),
        BodyPhysique::new(0.120, 0.65, SurfaceNature::RockyWorld, (v3(0.92, 0.96, 1.00), v3(0.62, 0.78, 0.92))),
    ),
    OrbitalSpec::new(
        (BodyId::TITAN, "TITAN"),
        OrbitElements::new(BodyId::SATURN, 10.2, (0.02, 2.75)),
        BodyPhysique::new(0.404, 1.35, SurfaceNature::RockyWorld, (v3(0.90, 0.62, 0.24), v3(0.58, 0.36, 0.14))),
    ),
    OrbitalSpec::new(
        (BodyId::URANUS, "URANUS"),
        OrbitElements::new(BodyId::SOL, 19.19, (0.01, 4.05)),
        BodyPhysique::new(3.98, 8.69, SurfaceNature::GasGiant { ringed: false }, (v3(0.56, 0.88, 0.92), v3(0.36, 0.68, 0.76))),
    ),
    OrbitalSpec::new(
        (BodyId::TITANIA, "TITANIA"),
        OrbitElements::new(BodyId::URANUS, 8.6, (0.05, 1.40)),
        BodyPhysique::new(0.180, 0.78, SurfaceNature::RockyWorld, (v3(0.74, 0.78, 0.82), v3(0.45, 0.50, 0.56))),
    ),
    OrbitalSpec::new(
        (BodyId::NEPTUNE, "NEPTUNE"),
        OrbitElements::new(BodyId::SOL, 30.07, (0.03, 5.15)),
        BodyPhysique::new(3.86, 11.15, SurfaceNature::GasGiant { ringed: false }, (v3(0.22, 0.42, 0.92), v3(0.14, 0.26, 0.68))),
    ),
    OrbitalSpec::new(
        (BodyId::TRITON, "TRITON"),
        OrbitElements::new(BodyId::NEPTUNE, 7.4, (-0.28, 2.20)),
        BodyPhysique::new(0.212, 0.78, SurfaceNature::RockyWorld, (v3(0.82, 0.86, 0.92), v3(0.52, 0.56, 0.66))),
    ),
];
