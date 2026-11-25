mod cubes;
mod medium_shape;
mod mining_edge;

use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};

pub use self::medium_shape::MediumShape;
pub use self::mining_edge::MiningEdge;

/// Material medium governing the sculpted 3D surface of a molded block in hand and world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockMedium {
    TurfEarth,
    CrustRock,
    QuarriedStone,
    RawTimber,
    PlanedPlanks,
    Sand,
    CantileverRig,
    SteeringWheel,
    AircraftEngine,
    FlightConsole,
    CabinSeat,
    CarWheel,
    FieldCamera,
    PedalCluster,
    CopperOre,
    IronOre,
    GoldOre,
    RocketNozzle,
    RocketCamera,
    VehicleBlueprint,
    FusionTorch,
    SpawnStone,
    TitaniumHull,
    CarbonSilicate,
    VitrifiedBasalt,
    BlackBoxPod,
}

impl BlockMedium {
    /// Compact 1-byte material identifier for world chronicles and multiplayer disclosures.
    pub const fn code(self) -> u8 {
        match self {
            Self::TurfEarth => 0,
            Self::CrustRock => 1,
            Self::QuarriedStone => 2,
            Self::RawTimber => 3,
            Self::PlanedPlanks => 4,
            Self::Sand => 5,
            Self::CantileverRig => 6,
            Self::SteeringWheel => 7,
            Self::AircraftEngine => 8,
            Self::FlightConsole => 9,
            Self::CabinSeat => 10,
            Self::CarWheel => 11,
            Self::FieldCamera => 12,
            Self::PedalCluster => 13,
            Self::CopperOre => 14,
            Self::IronOre => 15,
            Self::GoldOre => 16,
            Self::RocketNozzle => 17,
            Self::RocketCamera => 18,
            Self::VehicleBlueprint => 19,
            Self::FusionTorch => 20,
            Self::SpawnStone => 21,
            Self::TitaniumHull => 22,
            Self::CarbonSilicate => 23,
            Self::VitrifiedBasalt => 24,
            Self::BlackBoxPod => 25,
        }
    }

    /// Reconstructs a `BlockMedium` from its compact 1-byte chronicle identifier.
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::CrustRock,
            2 => Self::QuarriedStone,
            3 => Self::RawTimber,
            4 => Self::PlanedPlanks,
            5 => Self::Sand,
            6 => Self::CantileverRig,
            7 => Self::SteeringWheel,
            8 => Self::AircraftEngine,
            9 => Self::FlightConsole,
            10 => Self::CabinSeat,
            11 => Self::CarWheel,
            12 => Self::FieldCamera,
            13 => Self::PedalCluster,
            14 => Self::CopperOre,
            15 => Self::IronOre,
            16 => Self::GoldOre,
            17 => Self::RocketNozzle,
            18 => Self::RocketCamera,
            19 => Self::VehicleBlueprint,
            20 => Self::FusionTorch,
            21 => Self::SpawnStone,
            22 => Self::TitaniumHull,
            23 => Self::CarbonSilicate,
            24 => Self::VitrifiedBasalt,
            25 => Self::BlackBoxPod,
            _ => Self::TurfEarth,
        }
    }

    /// Indicates whether placement and removal of this block medium are restricted to the server admin.
    pub const fn admin_restricted(self) -> bool {
        matches!(self, Self::SpawnStone)
    }

    /// Indicates whether this medium is an Expanse-grade aerospace hull alloy or composite armor plate.
    pub const fn aerospace_hull(self) -> bool {
        matches!(self, Self::TitaniumHull | Self::CarbonSilicate)
    }

    /// Indicates whether this medium provides ceramic carbon-silicate thermal entry and impact shielding.
    pub const fn thermal_shielded(self) -> bool {
        matches!(self, Self::CarbonSilicate)
    }

    /// Indicates whether this medium is a stratified subterranean mineral ore deposit.
    pub const fn mineral_ore(self) -> bool {
        matches!(self, Self::CopperOre | Self::IronOre | Self::GoldOre)
    }

    /// Indicates whether this block medium settles downward under planetary gravity when unsupported below.
    pub const fn settles_under_gravity(self) -> bool {
        matches!(self, Self::Sand)
    }

    /// Indicates whether this block medium is a vehicle engine or steering control.
    pub const fn vehicle_engine(self) -> bool {
        matches!(
            self,
            Self::SteeringWheel | Self::AircraftEngine | Self::FlightConsole | Self::RocketNozzle | Self::FusionTorch
        )
    }

    /// Indicates whether this block medium is a sculpted mechanical vehicle component.
    pub const fn mechanical_part(self) -> bool {
        self.vehicle_engine()
            || matches!(
                self,
                Self::CabinSeat | Self::CarWheel | Self::PedalCluster | Self::RocketCamera
            )
    }

    /// Indicates whether this medium is a portable item picked up in a single click without excavation stages.
    pub const fn instant_pickup(self) -> bool {
        matches!(self, Self::FieldCamera | Self::BlackBoxPod)
    }

    /// Indicates whether this medium is physical spacecraft residue (plasma-fused basalt or crash black-box pod).
    pub const fn flight_residue(self) -> bool {
        matches!(self, Self::VitrifiedBasalt | Self::BlackBoxPod)
    }

    /// Indicates whether this medium yields to the given `MiningEdge` during excavation.
    pub const fn yields_to(self, edge: MiningEdge) -> bool {
        match self {
            Self::QuarriedStone | Self::VitrifiedBasalt | Self::CopperOre | Self::IronOre => edge.pierces_stone(),
            Self::CrustRock | Self::GoldOre => edge.pierces_crust(),
            _ => true,
        }
    }

    pub(super) const fn sculpted_tones(self) -> (Vec3, Vec3) {
        match self {
            Self::RawTimber => (Vec3::new(0.85, 0.70, 0.46), Vec3::new(0.32, 0.20, 0.11)),
            Self::PlanedPlanks => (Vec3::new(0.84, 0.67, 0.41), Vec3::new(0.74, 0.56, 0.33)),
            Self::QuarriedStone => (Vec3::new(0.62, 0.64, 0.66), Vec3::new(0.50, 0.51, 0.53)),
            Self::CopperOre => (Vec3::new(0.86, 0.46, 0.22), Vec3::new(0.36, 0.35, 0.33)),
            Self::IronOre => (Vec3::new(0.72, 0.75, 0.80), Vec3::new(0.30, 0.30, 0.33)),
            Self::GoldOre => (Vec3::new(0.98, 0.80, 0.22), Vec3::new(0.25, 0.23, 0.27)),
            Self::Sand => (Vec3::new(0.88, 0.78, 0.52), Vec3::new(0.78, 0.68, 0.44)),
            Self::TitaniumHull | Self::CarbonSilicate => super::AerospacePlating::sculpted_tones(self),
            _ => (Vec3::new(0.35, 0.31, 0.28), Vec3::new(0.31, 0.27, 0.25)),
        }
    }

    /// Primary and secondary debris particle colors emitted when striking this block medium.
    pub const fn debris_tones(self) -> (Vec3, Vec3) {
        match self {
            Self::TurfEarth => (Vec3::new(0.44, 0.31, 0.19), Vec3::new(0.32, 0.22, 0.13)),
            Self::CrustRock => (Vec3::new(0.35, 0.31, 0.28), Vec3::new(0.25, 0.21, 0.20)),
            Self::QuarriedStone => (Vec3::new(0.62, 0.64, 0.66), Vec3::new(0.46, 0.48, 0.50)),
            Self::RawTimber | Self::PlanedPlanks | Self::CantileverRig => (Vec3::new(0.82, 0.66, 0.42), Vec3::new(0.58, 0.42, 0.24)),
            Self::Sand => (Vec3::new(0.88, 0.78, 0.52), Vec3::new(0.74, 0.64, 0.40)),
            Self::SteeringWheel | Self::FieldCamera => (Vec3::new(0.94, 0.76, 0.24), Vec3::new(0.20, 0.22, 0.27)),
            Self::AircraftEngine => (Vec3::new(0.22, 0.88, 0.98), Vec3::new(0.54, 0.62, 0.72)),
            Self::FlightConsole => (Vec3::new(0.16, 0.94, 0.78), Vec3::new(0.90, 0.72, 0.24)),
            Self::RocketNozzle => (Vec3::new(0.99, 0.38, 0.12), Vec3::new(0.22, 0.24, 0.30)),
            Self::RocketCamera => (Vec3::new(0.24, 0.88, 0.98), Vec3::new(0.84, 0.88, 0.93)),
            Self::CabinSeat => (Vec3::new(0.72, 0.24, 0.16), Vec3::new(0.36, 0.20, 0.13)),
            Self::CarWheel => (Vec3::new(0.18, 0.20, 0.22), Vec3::new(0.68, 0.72, 0.78)),
            Self::PedalCluster => (Vec3::new(0.88, 0.22, 0.18), Vec3::new(0.24, 0.84, 0.36)),
            Self::CopperOre => (Vec3::new(0.86, 0.46, 0.22), Vec3::new(0.18, 0.68, 0.52)),
            Self::IronOre => (Vec3::new(0.72, 0.75, 0.80), Vec3::new(0.64, 0.30, 0.20)),
            Self::GoldOre => (Vec3::new(0.98, 0.80, 0.22), Vec3::new(0.25, 0.23, 0.27)),
            Self::VehicleBlueprint => (Vec3::new(0.18, 0.52, 0.92), Vec3::new(0.88, 0.72, 0.26)),
            Self::FusionTorch => (Vec3::new(0.28, 0.86, 1.00), Vec3::new(0.32, 0.36, 0.44)),
            Self::SpawnStone => (Vec3::new(0.22, 0.94, 0.98), Vec3::new(0.92, 0.76, 0.28)),
            Self::TitaniumHull => (Vec3::new(0.64, 0.70, 0.78), Vec3::new(0.34, 0.38, 0.45)),
            Self::CarbonSilicate => (Vec3::new(0.24, 0.80, 0.98), Vec3::new(0.14, 0.18, 0.26)),
            Self::VitrifiedBasalt => (Vec3::new(0.14, 0.16, 0.22), Vec3::new(0.96, 0.34, 0.08)),
            Self::BlackBoxPod => (Vec3::new(0.98, 0.36, 0.05), Vec3::new(0.20, 0.23, 0.28)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_box(
        u0: f32, u1: f32, v0: f32, v1: f32, h0: f32, h1: f32,
        frame: &SurfaceFrame, wall_col: Vec3, top_col: Vec3, bottom_cap: bool, target: &mut dyn MeshTarget,
    ) {
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        let [b0, b1, b2, b3] = [
            frame.point_at(u0, v0, h0), frame.point_at(u1, v0, h0),
            frame.point_at(u1, v1, h0), frame.point_at(u0, v1, h0),
        ];
        let [p0, p1, p2, p3] = [
            frame.point_at(u0, v0, h1), frame.point_at(u1, v0, h1),
            frame.point_at(u1, v1, h1), frame.point_at(u0, v1, h1),
        ];
        let top_glow = if crate::MeadowCrown::new(frame.normal(), 0).verdant(top_col) { 0.02 } else { 0.0 };
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, top_col, top_glow));
        if bottom_cap {
            target.push_quad(Quad::new(b0, b3, b2, b1, -n, wall_col));
        }
        target.push_quad(Quad::new(b0, b1, p1, p0, -b, wall_col));
        target.push_quad(Quad::new(p3, p2, b2, b3, b, wall_col));
        target.push_quad(Quad::new(b3, b0, p0, p3, -t, wall_col));
        target.push_quad(Quad::new(b1, b2, p2, p1, t, wall_col));
    }
}
