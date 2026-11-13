use super::HeldRelic;

/// Domain lexicon translating `HeldRelic` items to display titles and CLI/profile tokens.
pub struct RelicLexicon {
    relic: HeldRelic,
}

impl RelicLexicon {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic) -> Self {
        Self { relic }
    }

    pub const fn title(&self) -> &'static str {
        match self.relic {
            HeldRelic::EmptyPalm => "EMPTY SATCHEL",
            HeldRelic::QuarriedStone => "STONE BLOCK",
            HeldRelic::TimberTorch => "TIMBER TORCH",
            HeldRelic::WoodenMallet => "WOODEN MALLET",
            HeldRelic::WoodmanAxe => "WOODMAN AXE",
            HeldRelic::KnightlySword => "KNIGHT SWORD",
            HeldRelic::IronSpade => "IRON SPADE",
            HeldRelic::ExcavatedEarth => "EARTH BLOCK",
            HeldRelic::DeadwoodBough => "DEADWOOD",
            HeldRelic::FlintStone => "FLINT STONE",
            HeldRelic::TimberBlock => "WOOD BLOCK",
            HeldRelic::PlankBlock => "WOOD PLANKS",
            HeldRelic::CrustBlock => "CRUST BLOCK",
            HeldRelic::CantileverCrane => "CRANE LIFT",
            HeldRelic::SteeringWheel => "STEERING WHEEL",
            HeldRelic::AircraftEngine => "PLANE ENGINE",
            HeldRelic::RocketEngine => "ROCKET ENGINE",
            HeldRelic::CabinSeat => "SEAT BLOCK",
            HeldRelic::CarWheel => "CAR WHEEL",
            HeldRelic::FieldCamera => "FIELD CAMERA",
            HeldRelic::PedalCluster => "PEDAL BLOCK",
            HeldRelic::CrossAxle => "CROSS AXLE",
            HeldRelic::CopperOre => "COPPER ORE",
            HeldRelic::IronOre => "IRON ORE",
            HeldRelic::GoldOre => "GOLD ORE",
        }
    }

    pub fn parse(&self, token: &str) -> Option<HeldRelic> {
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
            "ironspade" | "spade" => Some(HeldRelic::IronSpade),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(HeldRelic::ExcavatedEarth),
            "deadwood" | "deadwoodbough" => Some(HeldRelic::DeadwoodBough),
            "flintstone" | "flint" => Some(HeldRelic::FlintStone),
            "woodblock" | "timberblock" | "timber" => Some(HeldRelic::TimberBlock),
            "woodplanks" | "plankblock" | "planks" => Some(HeldRelic::PlankBlock),
            "crustblock" | "crust" => Some(HeldRelic::CrustBlock),
            "cranelift" | "cantilevercrane" | "crane" => Some(HeldRelic::CantileverCrane),
            "steeringwheel" | "steering" => Some(HeldRelic::SteeringWheel),
            "planeengine" | "aircraftengine" => Some(HeldRelic::AircraftEngine),
            "rocketengine" | "rocket" => Some(HeldRelic::RocketEngine),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(HeldRelic::CabinSeat),
            "carwheel" | "wheel" => Some(HeldRelic::CarWheel),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(HeldRelic::PedalCluster),
            "crossaxle" | "axle" | "axleblock" | "querachse" => Some(HeldRelic::CrossAxle),
            "copperore" | "copper" | "kupfer" => Some(HeldRelic::CopperOre),
            "ironore" | "iron" | "eisen" => Some(HeldRelic::IronOre),
            "goldore" | "gold" => Some(HeldRelic::GoldOre),
            _ => None,
        }
    }
}
