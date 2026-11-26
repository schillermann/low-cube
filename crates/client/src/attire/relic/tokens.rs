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
            "timbertorch" | "torch" => Some(HeldRelic::TimberTorch),
            "woodenmallet" | "mallet" => Some(HeldRelic::WoodenMallet),
            "woodmanaxe" | "axe" => Some(HeldRelic::WoodmanAxe),
            "knightlysword" | "knightsword" | "sword" => Some(HeldRelic::KnightlySword),
            "woodenspade" | "woodspade" | "holzspaten" => Some(HeldRelic::WoodenSpade),
            "copperspade" | "kupferspaten" => Some(HeldRelic::CopperSpade),
            "ironspade" | "spade" | "eisenspaten" => Some(HeldRelic::IronSpade),
            "titaniumspade" | "titanspaten" => Some(HeldRelic::TitaniumSpade),
            "flintpickaxe" | "flintpick" | "stonepickaxe" | "pickaxe" | "hacke" | "spitzhacke" | "feuersteinhacke" => {
                Some(HeldRelic::FlintPickaxe)
            }
            "copperpickaxe" | "copperpick" | "kupferhacke" | "kupferspitzhacke" => Some(HeldRelic::CopperPickaxe),
            "ironpickaxe" | "ironpick" | "eisenhacke" | "eisenspitzhacke" => Some(HeldRelic::IronPickaxe),
            "titaniumpickaxe" | "titaniumpick" | "titanhacke" | "titanspitzhacke" => Some(HeldRelic::TitaniumPickaxe),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(HeldRelic::ExcavatedEarth),
            "deadwood" | "deadwoodbough" => Some(HeldRelic::DeadwoodBough),
            "flintstone" | "flint" => Some(HeldRelic::FlintStone),
            "woodblock" | "timberblock" | "timber" => Some(HeldRelic::TimberBlock),
            "woodplanks" | "plankblock" | "planks" => Some(HeldRelic::PlankBlock),
            "crustblock" | "crust" => Some(HeldRelic::CrustBlock),
            "cranelift" | "cantilevercrane" | "crane" => Some(HeldRelic::CantileverCrane),
            "steeringwheel" | "steering" => Some(HeldRelic::SteeringWheel),
            "planeengine" | "aircraftengine" => Some(HeldRelic::AircraftEngine),
            "flightconsole" | "console" | "steuerkonsole" => Some(HeldRelic::FlightConsole),
            "fusiontorch" | "fusiondrive" | "epstein" | "fusionsantrieb" => Some(HeldRelic::FusionTorch),
            "rocketnozzle" | "nozzle" | "thruster" | "triebwerk" | "rocket" => Some(HeldRelic::RocketNozzle),
            "rocketcamera" | "hullcamera" | "spacecamera" | "raketenkamera" | "nosecamera" => Some(HeldRelic::RocketCamera),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(HeldRelic::CabinSeat),
            "carwheel" | "wheel" => Some(HeldRelic::CarWheel),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(HeldRelic::PedalCluster),
            "copperore" | "copper" | "kupfer" => Some(HeldRelic::CopperOre),
            "ironore" | "iron" | "eisen" => Some(HeldRelic::IronOre),
            "goldore" | "gold" | "golderz" => Some(HeldRelic::GoldOre),
            "boarmeat" | "meat" | "venison" | "fleisch" | "wildschwein" => Some(HeldRelic::BoarMeat),
            "blueprint" | "blueprintblock" | "vehicleblueprint" | "bauplan" => Some(HeldRelic::VehicleBlueprint),
            "spawnstone" | "spawnblock" | "spawnpoint" | "arrivalstone" => Some(HeldRelic::SpawnStone),
            "titaniumhull" | "titanium" | "titaniumalloy" | "titan" | "titanhuelle" => Some(HeldRelic::TitaniumHull),
            "carbonsilicate" | "carbonlace" | "silicatelace" | "cermet" | "keramik" => Some(HeldRelic::CarbonSilicate),
            "vitrifiedbasalt" | "fusedbasalt" | "scorchbasalt" | "schmelzbasalt" => Some(HeldRelic::VitrifiedBasalt),
            "blackboxpod" | "blackbox" | "flightblackbox" | "flugschreiber" => Some(HeldRelic::BlackBoxPod),
            "laserbeacon" | "opticalbeacon" | "leuchtfeuer" | "beacon" => Some(HeldRelic::LaserBeacon),
            "bakingoven" | "breadoven" | "backofen" | "oven" => Some(HeldRelic::BakingOven),
            "bloomeryforge" | "smeltingforge" | "bloomery" | "schmelzesse" | "schmelzofen" | "forge" => {
                Some(HeldRelic::BloomeryForge)
            }
            "smithyanvil" | "anvil" | "amboss" => Some(HeldRelic::SmithyAnvil),
            "roastedboar" | "roastboar" | "braten" | "wildschweinbraten" => Some(HeldRelic::RoastedBoar),
            "copperbloom" | "kupferluppe" => Some(HeldRelic::CopperBloom),
            "ironbloom" | "eisenluppe" | "roheisen" => Some(HeldRelic::IronBloom),
            "titaniumbillet" | "spacetitanium" | "titanrohling" | "titanbarren" => Some(HeldRelic::TitaniumBillet),
            _ => None,
        }
    }
}
