pub(super) struct RecipeLedger {
    recipe_idx: u8,
}

impl RecipeLedger {
    pub(super) const fn new(recipe_idx: u8) -> Self {
        Self { recipe_idx }
    }

    pub(super) const fn vacuum_hearth(self) -> bool {
        self.recipe_idx == 103
    }

    pub(super) const fn air_hearth(self) -> bool {
        matches!(self.recipe_idx, 100..=102)
    }

    pub(super) const fn spec(self) -> Option<(u8, u32, [(u8, u32); 2])> {
        match self.recipe_idx {
            0 => Some((4, 1, [(11, 1), (47, 2)])),
            1 => Some((11, 4, [(10, 1), (0, 0)])),
            2 => Some((13, 1, [(11, 2), (1, 1)])),
            3 => Some((27, 1, [(11, 1), (9, 1)])),
            4 => Some((3, 1, [(8, 2), (0, 0)])),
            5 => Some((6, 1, [(11, 1), (47, 2)])),
            6 => Some((5, 1, [(11, 1), (47, 3)])),
            7 => Some((2, 1, [(8, 1), (9, 1)])),
            8 => Some((1, 1, [(9, 2), (0, 0)])),
            9 => Some((17, 1, [(11, 1), (0, 0)])),
            10 => Some((20, 1, [(11, 1), (9, 1)])),
            11 => Some((18, 4, [(11, 1), (9, 1)])),
            12 => Some((14, 1, [(21, 1), (11, 1)])),
            13 => Some((19, 1, [(21, 2), (23, 1)])),
            14 => Some((15, 1, [(22, 2), (21, 2)])),
            15 => Some((16, 1, [(23, 2), (21, 2)])),
            16 => Some((25, 2, [(22, 3), (23, 1)])),
            17 => Some((28, 1, [(23, 2), (22, 3)])),
            18 => Some((26, 2, [(21, 2), (23, 1)])),
            19 => Some((30, 1, [(8, 1), (9, 3)])),
            20 => Some((31, 1, [(11, 1), (47, 2)])),
            21 => Some((32, 4, [(48, 2), (47, 1)])),
            22 => Some((33, 4, [(23, 1), (9, 2)])),
            23 => Some((35, 1, [(22, 2), (21, 1)])),
            24 => Some((36, 1, [(23, 2), (21, 1)])),
            25 => Some((37, 1, [(8, 2), (0, 0)])),
            26 => Some((38, 1, [(11, 1), (46, 2)])),
            27 => Some((39, 1, [(11, 1), (48, 1)])),
            28 => Some((40, 1, [(11, 1), (46, 2)])),
            29 => Some((41, 1, [(11, 1), (48, 2)])),
            30 => Some((42, 1, [(1, 2), (8, 1)])),
            31 => Some((43, 1, [(1, 3), (2, 1)])),
            32 => Some((44, 1, [(1, 2), (10, 1)])),
            100 => Some((45, 1, [(24, 1), (8, 1)])),
            101 => Some((46, 1, [(21, 1), (8, 1)])),
            102 => Some((47, 1, [(22, 1), (8, 1)])),
            103 => Some((48, 2, [(12, 1), (22, 1)])),
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
            "camera" | "fieldcamera" => Some(19),
            "stoneblock" | "quarriedstone" | "stone" => Some(1),
            "timbertorch" | "torch" => Some(2),
            "woodenmallet" | "mallet" => Some(3),
            "woodmanaxe" | "axe" => Some(4),
            "knightlysword" | "knightsword" | "sword" => Some(5),
            "ironspade" | "spade" | "eisenspaten" => Some(6),
            "woodenspade" | "woodspade" | "holzspaten" => Some(37),
            "copperspade" | "kupferspaten" => Some(38),
            "titaniumspade" | "titanspaten" => Some(39),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(7),
            "deadwood" | "deadwoodbough" => Some(8),
            "flintstone" | "flint" => Some(9),
            "woodblock" | "timberblock" | "timber" => Some(10),
            "woodplanks" | "plankblock" | "planks" => Some(11),
            "crustblock" | "crust" => Some(12),
            "cranelift" | "cantilevercrane" | "crane" => Some(13),
            "steeringwheel" | "steering" => Some(14),
            "planeengine" | "aircraftengine" => Some(15),
            "flightconsole" | "console" | "steuerkonsole" => Some(16),
            "fusiontorch" | "fusiondrive" | "epstein" | "fusionsantrieb" => Some(28),
            "rocketnozzle" | "nozzle" | "thruster" | "triebwerk" | "rocket" => Some(25),
            "rocketcamera" | "hullcamera" | "spacecamera" | "raketenkamera" | "nosecamera" => Some(26),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(17),
            "carwheel" | "wheel" => Some(18),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(20),
            "copperore" | "copper" | "kupfer" => Some(21),
            "ironore" | "iron" | "eisen" => Some(22),
            "goldore" | "gold" | "golderz" => Some(23),
            "boarmeat" | "meat" | "venison" | "fleisch" | "wildschwein" => Some(24),
            "blueprint" | "blueprintblock" | "vehicleblueprint" | "bauplan" => Some(27),
            "spawnstone" | "spawnblock" | "spawnpoint" | "arrivalstone" => Some(29),
            "flintpickaxe" | "flintpick" | "stonepickaxe" | "pickaxe" | "hacke" | "spitzhacke" | "feuersteinhacke" => {
                Some(30)
            }
            "copperpickaxe" | "copperpick" | "kupferhacke" | "kupferspitzhacke" => Some(40),
            "ironpickaxe" | "ironpick" | "eisenhacke" | "eisenspitzhacke" => Some(31),
            "titaniumpickaxe" | "titaniumpick" | "titanhacke" | "titanspitzhacke" => Some(41),
            "titaniumhull" | "titanium" | "titaniumalloy" | "titan" | "titanhuelle" => Some(32),
            "carbonsilicate" | "carbonlace" | "silicatelace" | "cermet" | "keramik" => Some(33),
            "vitrifiedbasalt" | "fusedbasalt" | "scorchbasalt" | "schmelzbasalt" => Some(34),
            "blackboxpod" | "blackbox" | "flightblackbox" | "flugschreiber" => Some(35),
            "laserbeacon" | "opticalbeacon" | "leuchtfeuer" | "beacon" => Some(36),
            "bakingoven" | "breadoven" | "backofen" | "oven" => Some(42),
            "bloomeryforge" | "smeltingforge" | "bloomery" | "schmelzesse" | "schmelzofen" | "forge" => Some(43),
            "smithyanvil" | "anvil" | "amboss" => Some(44),
            "roastedboar" | "roastboar" | "braten" | "wildschweinbraten" => Some(45),
            "copperbloom" | "kupferluppe" => Some(46),
            "ironbloom" | "eisenluppe" | "roheisen" => Some(47),
            "titaniumbillet" | "spacetitanium" | "titanrohling" | "titanbarren" => Some(48),
            _ => None,
        }
    }
}
