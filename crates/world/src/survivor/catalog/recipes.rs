pub(super) struct RecipeLedger {
    recipe_idx: u8,
}

impl RecipeLedger {
    pub(super) const fn new(recipe_idx: u8) -> Self {
        Self { recipe_idx }
    }

    pub(super) const fn vacuum_smelt(self) -> bool {
        self.recipe_idx == 103
    }

    pub(super) const fn spec(self) -> Option<(u8, u32, [(u8, u32); 2])> {
        match self.recipe_idx {
            0 => Some((6, 4, [(5, 1), (0, 0)])),
            1 => Some((8, 1, [(6, 2), (1, 1)])),
            2 => Some((21, 1, [(16, 1), (1, 1)])),
            3 => Some((12, 1, [(6, 1), (0, 0)])),
            4 => Some((15, 1, [(17, 1), (16, 1)])),
            5 => Some((13, 4, [(17, 1), (6, 1)])),
            6 => Some((9, 1, [(16, 1), (6, 1)])),
            7 => Some((14, 1, [(16, 2), (18, 1)])),
            8 => Some((10, 1, [(17, 2), (16, 2)])),
            9 => Some((11, 1, [(18, 2), (16, 2)])),
            10 => Some((19, 2, [(17, 3), (18, 1)])),
            11 => Some((22, 1, [(18, 2), (17, 3)])),
            12 => Some((20, 2, [(16, 2), (18, 1)])),
            13 => Some((24, 4, [(29, 2), (17, 1)])),
            14 => Some((25, 4, [(18, 1), (1, 2)])),
            15 => Some((27, 1, [(17, 2), (16, 1)])),
            16 => Some((28, 1, [(18, 2), (16, 1)])),
            17 => Some((2, 1, [(29, 1), (16, 1)])),
            18 => Some((3, 1, [(16, 2), (18, 1)])),
            103 => Some((29, 2, [(7, 1), (17, 1)])),
            _ => None,
        }
    }

    pub(super) fn parse_token(token: &str) -> Option<u8> {
        let key: String = token
            .trim()
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match key.as_str() {
            "stoneblock" | "quarriedstone" | "stone" => Some(1),
            "rigwrench" | "assemblytool" | "evawrench" | "wrench" | "montagewerkzeug" => Some(2),
            "suitradio" | "evaradio" | "radio" | "funkgeraet" | "transceiver" => Some(3),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(4),
            "woodblock" | "timberblock" | "timber" => Some(5),
            "woodplanks" | "plankblock" | "planks" => Some(6),
            "crustblock" | "crust" | "crustrock" => Some(7),
            "cranelift" | "cantilevercrane" | "crane" => Some(8),
            "steeringwheel" | "steering" => Some(9),
            "planeengine" | "aircraftengine" => Some(10),
            "flightconsole" | "console" | "steuerkonsole" => Some(11),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(12),
            "carwheel" | "wheel" => Some(13),
            "camera" | "fieldcamera" => Some(14),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(15),
            "copperore" | "copper" | "kupfer" => Some(16),
            "ironore" | "iron" | "eisen" => Some(17),
            "goldore" | "gold" | "golderz" => Some(18),
            "rocketnozzle" | "nozzle" | "thruster" | "triebwerk" | "rocket" => Some(19),
            "rocketcamera" | "hullcamera" | "spacecamera" | "raketenkamera" | "nosecamera" => Some(20),
            "blueprint" | "blueprintblock" | "vehicleblueprint" | "bauplan" => Some(21),
            "fusiontorch" | "fusiondrive" | "epstein" | "fusionsantrieb" => Some(22),
            "spawnstone" | "spawnblock" | "spawnpoint" | "arrivalstone" => Some(23),
            "titaniumhull" | "titanium" | "titaniumalloy" | "titan" | "titanhuelle" => Some(24),
            "carbonsilicate" | "carbonlace" | "silicatelace" | "cermet" | "keramik" => Some(25),
            "vitrifiedbasalt" | "fusedbasalt" | "scorchbasalt" | "schmelzbasalt" => Some(26),
            "blackboxpod" | "blackbox" | "flightblackbox" | "flugschreiber" => Some(27),
            "laserbeacon" | "opticalbeacon" | "leuchtfeuer" | "beacon" => Some(28),
            "titaniumbillet" | "spacetitanium" | "titanrohling" | "titanbarren" => Some(29),
            _ => None,
        }
    }
}
