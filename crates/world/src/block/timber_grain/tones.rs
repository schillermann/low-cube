use glam::Vec3;
use super::BlockMedium;

pub(super) struct MediumPalette {
    medium: BlockMedium,
}

impl MediumPalette {
    pub(super) const fn new(medium: BlockMedium) -> Self {
        Self { medium }
    }

    pub(super) const fn sculpted_tones(self) -> (Vec3, Vec3) {
        match self.medium {
            BlockMedium::RawTimber => (Vec3::new(0.85, 0.70, 0.46), Vec3::new(0.32, 0.20, 0.11)),
            BlockMedium::PlanedPlanks => (Vec3::new(0.84, 0.67, 0.41), Vec3::new(0.74, 0.56, 0.33)),
            BlockMedium::QuarriedStone => (Vec3::new(0.62, 0.64, 0.66), Vec3::new(0.50, 0.51, 0.53)),
            BlockMedium::CopperOre => (Vec3::new(0.86, 0.46, 0.22), Vec3::new(0.36, 0.35, 0.33)),
            BlockMedium::IronOre => (Vec3::new(0.72, 0.75, 0.80), Vec3::new(0.30, 0.30, 0.33)),
            BlockMedium::GoldOre => (Vec3::new(0.98, 0.80, 0.22), Vec3::new(0.25, 0.23, 0.27)),
            BlockMedium::Sand => (Vec3::new(0.88, 0.78, 0.52), Vec3::new(0.78, 0.68, 0.44)),
            BlockMedium::TitaniumHull | BlockMedium::CarbonSilicate => {
                super::super::AerospacePlating::sculpted_tones(self.medium)
            }
            _ => (Vec3::new(0.35, 0.31, 0.28), Vec3::new(0.31, 0.27, 0.25)),
        }
    }

    pub(super) const fn debris_tones(self) -> (Vec3, Vec3) {
        match self.medium {
            BlockMedium::TurfEarth => (Vec3::new(0.44, 0.31, 0.19), Vec3::new(0.32, 0.22, 0.13)),
            BlockMedium::CrustRock => (Vec3::new(0.35, 0.31, 0.28), Vec3::new(0.25, 0.21, 0.20)),
            BlockMedium::QuarriedStone => (Vec3::new(0.62, 0.64, 0.66), Vec3::new(0.46, 0.48, 0.50)),
            BlockMedium::RawTimber | BlockMedium::PlanedPlanks | BlockMedium::CantileverRig => {
                (Vec3::new(0.82, 0.66, 0.42), Vec3::new(0.58, 0.42, 0.24))
            }
            BlockMedium::Sand => (Vec3::new(0.88, 0.78, 0.52), Vec3::new(0.74, 0.64, 0.40)),
            BlockMedium::SteeringWheel | BlockMedium::FieldCamera => {
                (Vec3::new(0.94, 0.76, 0.24), Vec3::new(0.20, 0.22, 0.27))
            }
            BlockMedium::AircraftEngine => (Vec3::new(0.22, 0.88, 0.98), Vec3::new(0.54, 0.62, 0.72)),
            BlockMedium::FlightConsole => (Vec3::new(0.16, 0.94, 0.78), Vec3::new(0.90, 0.72, 0.24)),
            BlockMedium::RocketNozzle => (Vec3::new(0.99, 0.38, 0.12), Vec3::new(0.22, 0.24, 0.30)),
            BlockMedium::RocketCamera => (Vec3::new(0.24, 0.88, 0.98), Vec3::new(0.84, 0.88, 0.93)),
            BlockMedium::CabinSeat => (Vec3::new(0.72, 0.24, 0.16), Vec3::new(0.36, 0.20, 0.13)),
            BlockMedium::CarWheel => (Vec3::new(0.18, 0.20, 0.22), Vec3::new(0.68, 0.72, 0.78)),
            BlockMedium::PedalCluster => (Vec3::new(0.88, 0.22, 0.18), Vec3::new(0.24, 0.84, 0.36)),
            BlockMedium::CopperOre => (Vec3::new(0.86, 0.46, 0.22), Vec3::new(0.18, 0.68, 0.52)),
            BlockMedium::IronOre => (Vec3::new(0.72, 0.75, 0.80), Vec3::new(0.64, 0.30, 0.20)),
            BlockMedium::GoldOre => (Vec3::new(0.98, 0.80, 0.22), Vec3::new(0.25, 0.23, 0.27)),
            BlockMedium::VehicleBlueprint => (Vec3::new(0.18, 0.52, 0.92), Vec3::new(0.88, 0.72, 0.26)),
            BlockMedium::FusionTorch => (Vec3::new(0.28, 0.86, 1.00), Vec3::new(0.32, 0.36, 0.44)),
            BlockMedium::SpawnStone => (Vec3::new(0.22, 0.94, 0.98), Vec3::new(0.92, 0.76, 0.28)),
            BlockMedium::TitaniumHull => (Vec3::new(0.64, 0.70, 0.78), Vec3::new(0.34, 0.38, 0.45)),
            BlockMedium::CarbonSilicate => (Vec3::new(0.24, 0.80, 0.98), Vec3::new(0.14, 0.18, 0.26)),
            BlockMedium::VitrifiedBasalt => (Vec3::new(0.14, 0.16, 0.22), Vec3::new(0.96, 0.34, 0.08)),
            BlockMedium::BlackBoxPod => (Vec3::new(0.98, 0.36, 0.05), Vec3::new(0.20, 0.23, 0.28)),
            BlockMedium::LaserBeacon => (Vec3::new(0.18, 1.00, 0.62), Vec3::new(0.92, 0.74, 0.24)),
            BlockMedium::OriginStone => (Vec3::new(0.32, 0.96, 0.68), Vec3::new(0.95, 0.80, 0.32)),
            BlockMedium::ArcFurnace | BlockMedium::ThermalFin | BlockMedium::PropulsionBell => {
                (Vec3::new(0.98, 0.56, 0.16), Vec3::new(0.24, 0.28, 0.34))
            }
            BlockMedium::AssemblyBench | BlockMedium::ElectrolysisPlant | BlockMedium::PropellantTank => {
                (Vec3::new(0.24, 0.88, 0.98), Vec3::new(0.68, 0.74, 0.82))
            }
            BlockMedium::AirlockChamber | BlockMedium::DockingRing | BlockMedium::CrewBerth | BlockMedium::PilotSeat | BlockMedium::PhosphorConsole => {
                (Vec3::new(0.66, 0.72, 0.80), Vec3::new(0.18, 0.94, 0.68))
            }
        }
    }
}
