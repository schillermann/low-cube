mod traits;

use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, CraftBlueprint, CraftIdentity, CraftPose,
    ExposedStratum, GravityField, GridFleet, MiningEdge, MoldArchetype, PlanetaryWorld, RawAtom,
    SurfaceFind, VehicleAssembly, VehicleAxes, VehiclePart, WedgeHeading, WorldEarthworks,
};

const SURFACE_PARK_CLEARANCE: f32 = 4.5;

/// `PlanetaryWorld` decorator simulating free, rotating, burning, and docked `GridBody`s in space (`2` private fields).
pub struct OrbitalFleetWorld<W> {
    origin: W,
    fleet: GridFleet,
}

impl<W> OrbitalFleetWorld<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W, fleet: GridFleet) -> Self {
        Self { origin, fleet }
    }

    pub const fn wrapping(origin: W) -> Self {
        Self::new(origin, GridFleet::untouched())
    }

    pub const fn fleet(&self) -> &GridFleet {
        &self.fleet
    }

    pub fn fleet_mut(&mut self) -> &mut GridFleet {
        &mut self.fleet
    }

    pub(super) const fn inner(&self) -> &W {
        &self.origin
    }

    pub(super) fn inner_mut(&mut self) -> &mut W {
        &mut self.origin
    }

    pub(super) fn step_fleet_six_dof(&mut self, dt: f32) -> bool
    where
        W: GravityField,
    {
        let origin = &self.origin;
        self.fleet.step_six_dof(dt, |pos| origin.apply_pull(pos, dt))
    }
}

impl<W: PlanetaryWorld> OrbitalFleetWorld<W> {
    fn above_surface(&self, pos: Vec3) -> bool {
        let ground = self.origin.constrain_to_surface(pos, 0.0);
        if (pos - ground).length_squared() < 1e-6 {
            return self.origin.apply_pull(pos, 1.0).length_squared() < 0.25;
        }
        let up = self.origin.surface_up_at(pos);
        (pos - ground).dot(up) > SURFACE_PARK_CLEARANCE
    }

    fn spawn_orbital_flange(&mut self, eye: Vec3, direction: Vec3, medium: BlockMedium) -> bool {
        let dir = direction.normalize_or_zero();
        let spawn_origin = eye + if dir.length_squared() > 1e-4 { dir * 2.0 } else { Vec3::Z * 2.0 };
        let seed_med = if medium == BlockMedium::CantileverRig {
            BlockMedium::DockingRing
        } else {
            medium
        };
        let atom = RawAtom::molded(seed_med, MoldArchetype::FullCube, WedgeHeading::South);
        let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
        let assembly = VehicleAssembly::new(spawn_origin, axes, Vec3::Z, vec![VehiclePart::new((0, 0), 0.0, atom)]);
        self.fleet.insert_orbital(assembly, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
        true
    }
}

impl<W: GravityField> GravityField for OrbitalFleetWorld<W> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 { self.origin.pull_direction_at(position) }
    fn surface_up_at(&self, position: Vec3) -> Vec3 { self.origin.surface_up_at(position) }
    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 { self.origin.apply_pull(position, dt) }
}

impl<W: PlanetaryWorld> WorldEarthworks for OrbitalFleetWorld<W> {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.fleet
            .harvest_on_aimed(eye, direction)
            .or_else(|| self.origin.harvest_surface(eye, direction))
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.origin.loosen_surface(eye, direction, stage)
    }

    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool {
        self.origin.surface_yields_to(eye, direction, edge)
    }

    fn aimed_surface_medium(&self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.origin.aimed_surface_medium(eye, direction)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.origin.strike_timber(eye, direction)
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.origin.forage_surface(eye, direction)
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        self.origin.cast_surface_find(eye, direction, find)
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        if self.fleet.place_molded_on_aimed(eye, direction, mold) {
            return true;
        }
        if self.above_surface(avatar_pos)
            && matches!(mold.medium(), BlockMedium::DockingRing | BlockMedium::CantileverRig)
        {
            return self.spawn_orbital_flange(eye, direction, mold.medium());
        }
        self.origin.place_molded_surface_block(eye, direction, avatar_pos, mold)
    }

    fn place_scaled_surface_module(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, spec: crate::ModuleSpec) -> bool {
        if self.fleet.place_scaled_on_aimed(eye, direction, spec) {
            return true;
        }
        if self.above_surface(avatar_pos) && spec.medium() == BlockMedium::DockingRing {
            return self.spawn_orbital_flange(eye, direction, BlockMedium::DockingRing);
        }
        self.origin.place_scaled_surface_module(eye, direction, avatar_pos, spec)
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.origin.adjust_rig(eye, direction, delta_steps, rotate_quarter)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool { self.origin.lower_rig(eye, direction) }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        self.origin.build_rig_blueprint(eye, direction, blueprint)
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.fleet
            .board_orbital(eye, direction, holding_block)
            .or_else(|| self.origin.board_vehicle(eye, direction, holding_block))
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        if self.above_surface(assembly.origin()) {
            let quiet = VehicleAssembly::from_blueprint(
                assembly.blueprint().clone(),
                assembly.pose().with_motion(0.0, 0.0),
            );
            self.fleet.insert_orbital(quiet, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
            true
        } else {
            self.origin.park_vehicle(assembly)
        }
    }

    fn park_moving_vehicle(
        &mut self,
        assembly: VehicleAssembly,
        linear_vel: Vec3,
        angular_vel: Vec3,
        proper_accel: Vec3,
    ) -> bool {
        let airborne = self.above_surface(assembly.origin())
            || linear_vel.length_squared() > 0.05
            || angular_vel.length_squared() > 1e-5
            || proper_accel.length_squared() > 0.05;
        if airborne {
            self.fleet.insert_orbital(assembly, linear_vel, angular_vel, proper_accel);
            true
        } else {
            self.origin.park_vehicle(assembly)
        }
    }

    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool { self.origin.register_craft_blueprint(blueprint) }
    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.origin.claim_vehicle_helm(craft, pilot_slot, eye, direction)
    }
    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        self.origin.park_craft_by_id(craft, pose)
    }
    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.origin.disclose_stratum(face_index, stratum);
    }
    fn offworld_frame(&self, probe: Vec3) -> Option<(crate::BodyId, Vec3)> { self.origin.offworld_frame(probe) }
    fn body_center(&self, body: crate::BodyId) -> Option<Vec3> { self.origin.body_center(body) }
    fn harvest_offworld(&mut self, body: crate::BodyId, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.origin.harvest_offworld(body, eye, direction)
    }
    fn enact_offworld(&mut self, body: crate::BodyId, fact: &crate::WorldFact) -> bool {
        self.origin.enact_offworld(body, fact)
    }
    fn capture_sector_snapshot(&self, face: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        self.origin.capture_sector_snapshot(face, gsu, gsv)
    }
    fn apply_sector_snapshot(&mut self, snapshot: &crate::SectorSnapshot) -> bool {
        self.origin.apply_sector_snapshot(snapshot)
    }
    fn collect_sector_snapshots(&self, out: &mut Vec<crate::SectorSnapshot>) {
        self.origin.collect_sector_snapshots(out);
    }
    fn capture_offworld_snapshot(&mut self, body: crate::BodyId, face: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        self.origin.capture_offworld_snapshot(body, face, gsu, gsv)
    }
    fn collect_offworld_snapshots(&mut self, out: &mut Vec<(crate::BodyId, crate::SectorSnapshot)>) {
        self.origin.collect_offworld_snapshots(out);
    }
}
