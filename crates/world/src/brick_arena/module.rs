use crate::block::{BlockMedium, CardinalTurn};
use super::module_spec::ModuleSpan;

pub const MAX_MODULE_AXIS_CELLS: u8 = 16;

/// Fixed Space-Age multi-block station & spacecraft modules (`> 1` `0.5 m` block).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MachineKind {
    ArcFurnace,        // 2×2×3 (Lichtbogen-Schmelze)
    AssemblyBench,     // 2×1×2 (Fabrikator)
    ElectrolysisPlant, // 2×2×3 (Elektrolyse-Raffinerie)
    AirlockChamber,    // 2×2×4 (Luftschleuse)
    PilotSeat,         // 2×2×3 (Pilotensitz)
    PhosphorConsole,   // 2×2×2 (Phosphor-Flugkonsole)
    DockingRing,       // 2×2×1 (Andockring)
    CrewBerth,         // 2×4×2 (Spawn-Koje)
}

impl MachineKind {
    pub const fn all() -> [Self; 8] {
        [
            Self::ArcFurnace,
            Self::AssemblyBench,
            Self::ElectrolysisPlant,
            Self::AirlockChamber,
            Self::PilotSeat,
            Self::PhosphorConsole,
            Self::DockingRing,
            Self::CrewBerth,
        ]
    }

    /// Unrotated `(width_u, depth_v, height_w)` footprint in `0.5 m` grid cells (`> 1` cell).
    pub const fn canonical_dims(self) -> (u8, u8, u8) {
        match self {
            Self::ArcFurnace => (2, 2, 3),
            Self::AssemblyBench => (2, 1, 2),
            Self::ElectrolysisPlant => (2, 2, 3),
            Self::AirlockChamber => (2, 2, 4),
            Self::PilotSeat => (2, 2, 3),
            Self::PhosphorConsole => (2, 2, 2),
            Self::DockingRing => (2, 2, 1),
            Self::CrewBerth => (2, 4, 2),
        }
    }

    pub const fn footprint(self, turn: CardinalTurn) -> ModuleSpan {
        let (b, t, h) = self.canonical_dims();
        if (turn.code() & 1) == 1 {
            ModuleSpan::clamped(t, b, h)
        } else {
            ModuleSpan::clamped(b, t, h)
        }
    }

    pub const fn medium(self) -> BlockMedium {
        match self {
            Self::ArcFurnace => BlockMedium::ArcFurnace,
            Self::AssemblyBench => BlockMedium::AssemblyBench,
            Self::ElectrolysisPlant => BlockMedium::ElectrolysisPlant,
            Self::AirlockChamber => BlockMedium::AirlockChamber,
            Self::PilotSeat => BlockMedium::PilotSeat,
            Self::PhosphorConsole => BlockMedium::PhosphorConsole,
            Self::DockingRing => BlockMedium::DockingRing,
            Self::CrewBerth => BlockMedium::CrewBerth,
        }
    }

    pub const fn cell_mass_kg(self) -> f64 {
        match self {
            Self::ArcFurnace => 195.0,
            Self::AssemblyBench => 130.0,
            Self::ElectrolysisPlant => 165.0,
            Self::AirlockChamber => 145.0,
            Self::PilotSeat => 75.0,
            Self::PhosphorConsole => 90.0,
            Self::DockingRing => 210.0,
            Self::CrewBerth => 65.0,
        }
    }
}

/// Grid-scalable Space-Age propulsion, propellant storage, and thermal rejection modules (`B × T × H`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScalableKind {
    PropulsionBell,
    PropellantTank,
    ThermalFin,
}

impl ScalableKind {
    pub const fn all() -> [Self; 3] {
        [Self::PropulsionBell, Self::PropellantTank, Self::ThermalFin]
    }

    pub const fn initial_span(self, turn: CardinalTurn) -> ModuleSpan {
        let (b, t, h) = match self {
            Self::PropulsionBell => (2, 2, 2),
            Self::PropellantTank => (2, 2, 3),
            Self::ThermalFin => (3, 1, 2),
        };
        if (turn.code() & 1) == 1 {
            ModuleSpan::clamped(t, b, h)
        } else {
            ModuleSpan::clamped(b, t, h)
        }
    }

    pub const fn medium(self) -> BlockMedium {
        match self {
            Self::PropulsionBell => BlockMedium::PropulsionBell,
            Self::PropellantTank => BlockMedium::PropellantTank,
            Self::ThermalFin => BlockMedium::ThermalFin,
        }
    }

    pub const fn cell_mass_kg(self) -> f64 {
        match self {
            Self::PropulsionBell => 180.0,
            Self::PropellantTank => 125.0,
            Self::ThermalFin => 55.0,
        }
    }
}

/// Polymorphic Space-Age multi-block module classification (`Fixed` machine vs `B × T × H` `Scalable` module).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpaceModule {
    Machine(MachineKind),
    Scalable(ScalableKind),
}

impl SpaceModule {
    pub const fn from_medium(medium: BlockMedium) -> Option<Self> {
        match medium {
            BlockMedium::ArcFurnace => Some(Self::Machine(MachineKind::ArcFurnace)),
            BlockMedium::AssemblyBench => Some(Self::Machine(MachineKind::AssemblyBench)),
            BlockMedium::ElectrolysisPlant => Some(Self::Machine(MachineKind::ElectrolysisPlant)),
            BlockMedium::AirlockChamber => Some(Self::Machine(MachineKind::AirlockChamber)),
            BlockMedium::PilotSeat => Some(Self::Machine(MachineKind::PilotSeat)),
            BlockMedium::PhosphorConsole => Some(Self::Machine(MachineKind::PhosphorConsole)),
            BlockMedium::DockingRing => Some(Self::Machine(MachineKind::DockingRing)),
            BlockMedium::CrewBerth => Some(Self::Machine(MachineKind::CrewBerth)),
            BlockMedium::PropulsionBell => Some(Self::Scalable(ScalableKind::PropulsionBell)),
            BlockMedium::PropellantTank => Some(Self::Scalable(ScalableKind::PropellantTank)),
            BlockMedium::ThermalFin => Some(Self::Scalable(ScalableKind::ThermalFin)),
            _ => None,
        }
    }

    pub const fn medium(self) -> BlockMedium {
        match self {
            Self::Machine(k) => k.medium(),
            Self::Scalable(k) => k.medium(),
        }
    }

    pub const fn cell_mass_kg(self) -> f64 {
        match self {
            Self::Machine(k) => k.cell_mass_kg(),
            Self::Scalable(k) => k.cell_mass_kg(),
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::Machine(MachineKind::ArcFurnace) => 1,
            Self::Machine(MachineKind::AssemblyBench) => 2,
            Self::Machine(MachineKind::ElectrolysisPlant) => 3,
            Self::Machine(MachineKind::AirlockChamber) => 4,
            Self::Machine(MachineKind::PilotSeat) => 5,
            Self::Machine(MachineKind::PhosphorConsole) => 6,
            Self::Machine(MachineKind::DockingRing) => 7,
            Self::Machine(MachineKind::CrewBerth) => 8,
            Self::Scalable(ScalableKind::PropulsionBell) => 9,
            Self::Scalable(ScalableKind::PropellantTank) => 10,
            Self::Scalable(ScalableKind::ThermalFin) => 11,
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Machine(MachineKind::ArcFurnace)),
            2 => Some(Self::Machine(MachineKind::AssemblyBench)),
            3 => Some(Self::Machine(MachineKind::ElectrolysisPlant)),
            4 => Some(Self::Machine(MachineKind::AirlockChamber)),
            5 => Some(Self::Machine(MachineKind::PilotSeat)),
            6 => Some(Self::Machine(MachineKind::PhosphorConsole)),
            7 => Some(Self::Machine(MachineKind::DockingRing)),
            8 => Some(Self::Machine(MachineKind::CrewBerth)),
            9 => Some(Self::Scalable(ScalableKind::PropulsionBell)),
            10 => Some(Self::Scalable(ScalableKind::PropellantTank)),
            11 => Some(Self::Scalable(ScalableKind::ThermalFin)),
            _ => None,
        }
    }
}
