use super::{HeldRelic, RelicLexicon};

pub(super) trait RelicTokenLexicon {
    fn parse(&self, token: &str) -> Option<HeldRelic>;
}

impl RelicTokenLexicon for RelicLexicon {
    fn parse(&self, token: &str) -> Option<HeldRelic> {
        let key: String = token
            .trim()
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match key.as_str() {
            "camera" | "fieldcamera" => Some(HeldRelic::FieldCamera),
            "stoneblock" | "quarriedstone" | "stone" => Some(HeldRelic::QuarriedStone),
            "rigwrench" | "wrench" | "montagewerkzeug" | "multitool" => Some(HeldRelic::RigWrench),
            "suitradio" | "radio" | "transceiver" | "funkgeraet" => Some(HeldRelic::SuitRadio),
            "earthblock" | "excavatedearth" | "earth" | "dirt" | "regolith" => Some(HeldRelic::ExcavatedEarth),
            "woodblock" | "timberblock" | "timber" => Some(HeldRelic::TimberBlock),
            "woodplanks" | "plankblock" | "planks" => Some(HeldRelic::PlankBlock),
            "crustblock" | "crust" => Some(HeldRelic::CrustBlock),
            "cranelift" | "cantilevercrane" | "crane" => Some(HeldRelic::CantileverCrane),
            "steeringwheel" | "steering" => Some(HeldRelic::SteeringWheel),
            "planeengine" | "aircraftengine" => Some(HeldRelic::AircraftEngine),
            "flightconsole" | "console" | "steuerkonsole" => Some(HeldRelic::FlightConsole),
            "fusiontorch" | "fusiondrive" | "epstein" | "fusionsantrieb" => Some(HeldRelic::FusionTorch),
            "rocketnozzle" | "nozzle" | "triebwerk" | "rocket" => Some(HeldRelic::RocketNozzle),
            "rocketcamera" | "hullcamera" | "spacecamera" | "raketenkamera" | "nosecamera" => Some(HeldRelic::RocketCamera),
            "seatblock" | "cabinseat" | "seat" => Some(HeldRelic::CabinSeat),
            "carwheel" | "wheel" => Some(HeldRelic::CarWheel),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(HeldRelic::PedalCluster),
            "copperore" | "copper" | "kupfer" => Some(HeldRelic::CopperOre),
            "ironore" | "iron" | "eisen" => Some(HeldRelic::IronOre),
            "goldore" | "gold" | "golderz" => Some(HeldRelic::GoldOre),
            "blueprint" | "blueprintblock" | "vehicleblueprint" | "bauplan" => Some(HeldRelic::VehicleBlueprint),
            "spawnstone" | "spawnblock" | "spawnpoint" | "arrivalstone" => Some(HeldRelic::SpawnStone),
            "titaniumhull" | "titanium" | "titaniumalloy" | "titan" | "titanhuelle" => Some(HeldRelic::TitaniumHull),
            "carbonsilicate" | "carbonlace" | "silicatelace" | "cermet" | "keramik" => Some(HeldRelic::CarbonSilicate),
            "vitrifiedbasalt" | "fusedbasalt" | "scorchbasalt" | "schmelzbasalt" => Some(HeldRelic::VitrifiedBasalt),
            "blackboxpod" | "blackbox" | "flightblackbox" | "flugschreiber" => Some(HeldRelic::BlackBoxPod),
            "laserbeacon" | "opticalbeacon" | "leuchtfeuer" | "beacon" => Some(HeldRelic::LaserBeacon),
            "titaniumbillet" | "spacetitanium" | "titanrohling" | "titanbarren" => Some(HeldRelic::TitaniumBillet),
            "arcfurnace" | "lichtbogenschmelze" | "furnace" => Some(HeldRelic::ArcFurnace),
            "assemblybench" | "fabrikator" | "bench" => Some(HeldRelic::AssemblyBench),
            "electrolysisplant" | "elektrolyse" | "electrolysis" => Some(HeldRelic::ElectrolysisPlant),
            "airlockchamber" | "airlock" | "luftschleuse" => Some(HeldRelic::AirlockChamber),
            "pilotseat" | "pilotensitz" => Some(HeldRelic::PilotSeat),
            "phosphorconsole" | "phosphorflugkonsole" => Some(HeldRelic::PhosphorConsole),
            "dockingring" | "andockring" => Some(HeldRelic::DockingRing),
            "crewberth" | "spawnkoje" | "koje" | "berth" => Some(HeldRelic::CrewBerth),
            "propulsionbell" | "scalablethruster" | "skalierbarerthruster" => Some(HeldRelic::PropulsionBell),
            "propellanttank" | "scalabletank" | "treibstofftank" => Some(HeldRelic::PropellantTank),
            "thermalfin" | "scalableradiator" | "kuehlfinne" => Some(HeldRelic::ThermalFin),
            _ => None,
        }
    }
}
