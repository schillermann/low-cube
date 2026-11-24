use glam::{Quat, Vec3};
use world::{
    BlockMedium, BlockMold, BodyId, CleaveOutcome, CraftBlueprint, CraftIdentity, CraftPose,
    ExposedStratum, PlanetaryWorld, PresenceKinematics, SurfaceFind, VehicleAssembly,
    WorldEarthworks, WorldFact,
};

/// Active admin-placed `SpawnStone` coordinates distributing joining guests across multiple arrival plinths.
pub struct SpawnBeacons {
    points: Vec<(Vec3, Vec3)>,
    cursor: usize,
}

impl SpawnBeacons {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(points: Vec<(Vec3, Vec3)>, cursor: usize) -> Self {
        Self { points, cursor }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new(), 0)
    }

    pub fn refresh_from(&mut self, world: &dyn PlanetaryWorld) {
        let mut raw = Vec::new();
        world.collect_spawn_points(&mut raw);
        self.points.clear();
        for pos in raw {
            let up = world.surface_up_at(pos).normalize_or(Vec3::Y);
            self.points.push((pos, up));
        }
    }

    pub fn allocate_kinematics(&mut self, fallback_half: f32) -> (PresenceKinematics, Option<Vec3>) {
        if self.points.is_empty() {
            return (
                PresenceKinematics::new(
                    Vec3::new(0.0, fallback_half, 0.0),
                    Quat::IDENTITY,
                    Quat::IDENTITY,
                    (0.0, 0.0),
                ),
                None,
            );
        }
        let idx = self.cursor % self.points.len();
        self.cursor = self.cursor.wrapping_add(1);
        let (pos, up) = self.points[idx];
        let surface_rot = if up.dot(Vec3::Y) < -0.999 {
            Quat::from_rotation_x(std::f32::consts::PI)
        } else {
            Quat::from_rotation_arc(Vec3::Y, up)
        };
        (
            PresenceKinematics::new(pos + up, surface_rot, Quat::IDENTITY, (0.0, 0.0)),
            Some(pos + up),
        )
    }
}

/// Author-stamped guest `WorldFact` binding `HelmClaimed` petitions to the authenticated companion slot.
pub struct GuestFactStamp {
    slot: u32,
    fact: WorldFact,
}

impl GuestFactStamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(slot: u32, fact: WorldFact) -> Self {
        Self { slot, fact }
    }

    pub fn stamped(self) -> WorldFact {
        match self.fact {
            WorldFact::HelmClaimed { craft, eye, direction, .. } => {
                WorldFact::HelmClaimed { craft, pilot_slot: self.slot, eye, direction }
            }
            WorldFact::OffWorld { body, fact } => {
                let inner = match *fact {
                    WorldFact::HelmClaimed { craft, eye, direction, .. } => {
                        WorldFact::HelmClaimed { craft, pilot_slot: self.slot, eye, direction }
                    }
                    rest => rest,
                };
                WorldFact::OffWorld { body, fact: Box::new(inner) }
            }
            rest => rest,
        }
    }
}

/// `WorldEarthworks` decorator forbidding non-admin guests from placing, loosening, or harvesting `SpawnStone` blocks.
pub struct AdminBlockWard<W> {
    origin: W,
}

impl<W> AdminBlockWard<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W) -> Self {
        Self { origin }
    }
}

impl<W: WorldEarthworks> AdminBlockWard<W> {
    fn aimed_admin_block(&mut self, eye: Vec3, direction: Vec3) -> bool {
        if self
            .origin
            .loosen_surface(eye, direction, 1)
            .is_some_and(BlockMedium::admin_restricted)
        {
            let _ = self.origin.loosen_surface(eye, direction, 0);
            true
        } else {
            false
        }
    }
}

impl<W: WorldEarthworks> WorldEarthworks for AdminBlockWard<W> {
    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let loosened = self.origin.loosen_surface(eye, direction, stage)?;
        if loosened.admin_restricted() {
            let _ = self.origin.loosen_surface(eye, direction, 0);
            return None;
        }
        Some(loosened)
    }

    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        if self.aimed_admin_block(eye, direction) {
            return None;
        }
        self.origin.harvest_surface(eye, direction)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.origin.strike_timber(eye, direction)
    }

    fn strike_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        self.origin.strike_boar(eye, direction, power, max_reach)
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.origin.forage_surface(eye, direction)
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        self.origin.cast_surface_find(eye, direction, find)
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        if mold.medium().admin_restricted() {
            return false;
        }
        self.origin.place_molded_surface_block(eye, direction, avatar_pos, mold)
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.origin.adjust_rig(eye, direction, delta_steps, rotate_quarter)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.origin.lower_rig(eye, direction)
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.origin.board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        self.origin.park_vehicle(assembly)
    }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        self.origin.build_rig_blueprint(eye, direction, blueprint)
    }

    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.origin.register_craft_blueprint(blueprint)
    }

    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.origin.claim_vehicle_helm(craft, pilot_slot, eye, direction)
    }

    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        self.origin.park_craft_by_id(craft, pose)
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.origin.disclose_stratum(face_index, stratum);
    }

    fn offworld_frame(&self, probe: Vec3) -> Option<(BodyId, Vec3)> {
        self.origin.offworld_frame(probe)
    }

    fn body_center(&self, body: BodyId) -> Option<Vec3> {
        self.origin.body_center(body)
    }

    fn enact_offworld(&mut self, body: BodyId, fact: &WorldFact) -> bool {
        match *fact {
            WorldFact::MoldPlaced { mold, .. } if mold.medium().admin_restricted() => false,
            _ => self.origin.enact_offworld(body, fact),
        }
    }
}
