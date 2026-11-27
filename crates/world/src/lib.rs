use glam::Vec3;

mod arrival_sites;
mod biome;
mod block;
mod brick_arena;
mod celestial;
mod chronicle;
mod contracts;
mod cosmic_pos;
mod frame;
mod grid_body;
mod mesh_target;
mod noise;
mod null_face;
mod planet;
mod presence;
mod quad;
mod relief;
mod seed;
mod sight_cone;
mod surface_contact;
mod survivor;
mod topology;
mod tree;
mod weather;
mod wire_vector;

#[cfg(test)]
mod celestial_tests;
#[cfg(test)]
mod continental_cosmos_tests;
#[cfg(test)]
mod gravity_tests;
#[cfg(test)]
mod grid_body_tests;
#[cfg(test)]
mod mesh_target_tests;
#[cfg(test)]
mod offworld_eviction_tests;
#[cfg(test)]
mod quad_sphere_tests;

pub use arrival_sites::{ArrivalSites, MediumArrivalSites, PrioritizedArrivalSites};
pub use biome::{
    BarrenZone, BiomedQuadSphere, ForestedZone, HollowRelief, LivingRelief, OriginPlinthRelief,
    ReliefMold, ZoneSurface,
};
pub use celestial::{
    ActiveVicinity, AlienWorld, ApexSpin, AsteroidBelt, AtlasMode, AwakenedBody, BeltAndReach,
    BeltRock, BinaryGravity, BinaryPlanet, BodyId, BodyKinematics, BodyPhysique, BodySnapshot,
    Brad64, CelestialAtlas, CelestialSystem, CosmicFrame, EllipticShape, FrameHandoff,
    GravityBubble, HeliosphereWorld, KeplerAnomaly, LunarCraterRelief, LunarWorld, OrbitElements,
    OrbitalFleetWorld, OrbitalLocus, OrbitalPlane, OrbitalSpec, PlanetarySpin, RadialWell,
    SignalKind, SkySignal, SolarCatalog, SolarEphemeris, SolarProportions, SurfaceNature,
    TranslatedWorld, VicinityCache, DEEP_SPACE_SECTOR_SPAN, MAJOR_BODY_COUNT,
    OFFWORLD_IDLE_TTL_SECS,
};
pub use block::{
    AttitudeNozzleMesh, BlackBoxPodMesh, BlockMedium, BlockMold,
    BlueprintBlockMesh, CabinSeatMesh, CardinalTurn, ConsoleOrrery, CornerHeading, CuboidShape,
    DiagonalSlope, FieldCameraTripod, FlightConsoleMesh, FootwellPlinth,
    FusionTorchMesh, GentleRidge, GentleValley, GentleWedge, GridCell, LaserBeaconMesh, MediumShape,
    MineralSeam, MiningEdge, MoldArchetype, NozzleAxis, OriginPlinthMesh, PalmMolds,
    PedalClusterMesh, PyramidPeak, RampWedge, RidgeCorner, RocketNozzleMesh,
    SpawnPlinthMesh, StairCorner, StairFlight, TurfSod, ValleyCorner, VitrifiedBasaltMesh,
    VoxelShape, WalnutFloorPan, WedgeHeading, WheelProfile,
};
pub use brick_arena::{
    AnchorOffset, AtomKind, AtomSurface, Brick8, BrickArena, BrickCell, BrickCoord, BrickStructure,
    BrickVolume, CellCoord, DismantledSpan, GranularFlag, InertialBricks, LocalSlot, MachineKind,
    ModuleSculpture, ModuleSpan, ModuleSpec, NicheConfinement, RawAtom, ScalableKind,
    SpaceModule, SpanHologram, SteinerMass, SymmetricInertia, MAX_MODULE_AXIS_CELLS,
};
pub use chronicle::{
    RigSnapshot, SectorColumnState, SectorSnapshot, SnapshotBlock, WorldFact,
    NATURAL_OVERHANG_STAMP, SNAPSHOT_SECTOR_CELLS,
};
pub use contracts::{
    CompanionPortal, PlanetaryWorld, TerrainMesh, VesselDeck, WorldEarthworks,
};
pub use cosmic_pos::CosmicPos;
pub use frame::{FrameShape, SurfaceFrame};
pub use grid_body::{
    BodyMooring, BodyMotion, CentrifugalRing, CoriolisDeflection, CrustMason, CrustSurvey,
    DeckDynamics, DockedLink, DockingClamp, DockingPort, FlangeHoist, FlangeMason, FlangeSurvey,
    FleetDockyard, GridBody, GridFleet, HullMason, InertialForce, MagBootGrip, SurfaceFleetView,
    ThrustGravity, ZeroGravityField,
};
pub use mesh_target::{
    CountedMeshTarget, CrateredMeshTarget, MeadowCrown, NaturalMeadowTarget, TurfMeshTarget,
};
pub use noise::{CoherentNoise, DomainWarp, FractalHarmonics, SeededFrequency, SpatialFrequency};
pub use null_face::NullFace;
pub use planet::{ModifiedSlices, PatchCoord, QuadSphere, ZoneShards};
pub use presence::{
    Callsign, CompanionPresence, DrivenCraft, PilotCraft,
    PresenceAttire, PresenceKinematics, PresenceRhythm, ProximityUtterance,
    VesselBerth, MAX_CALLSIGN_BYTES, MAX_UTTERANCE_BYTES,
};
pub use seed::WorldSeed;
pub use sight_cone::SightCone;
pub use surface_contact::SurfaceContact;
pub use survivor::{
    RelicCatalog, SatchelManifest, SurvivorRecord, SurvivorWelcome, RELIC_SLOT_COUNT,
};
pub use relief::{
    AttitudeGuidance, BlueprintCatalog, CellRole, ClipmapLod, CockpitAxes, CraftBlueprint,
    CraftIdentity, CraftPose, CrustVault, DeckFrame, ExposedStratum, FacetedReliefFace, HelmLocks,
    HollowVeins, HullOptic, HullWireframe, JetTone, LivingVeins, MeadowRing, MeadowStamp,
    MountDomain, MountSite, OpticMount, PilotOrder, PlanetHeightClipmap, PlannedMount,
    ReliefElevation, ReliefFacet, ReliefPalette, RigidImpulse, SeatOccupancy, SubterraneanVeins,
    SurfaceSightRay, TerracedRelief, VehicleAssembly, VehicleAxes, VehiclePart, VesselSink,
    MAX_EXHAUST_JETS, MAX_WIRE_EDGES, MEADOW_RING_SLOTS,
};
pub use topology::{
    ClimateZone, CoordinateFold, FacePerimeter, FoldedPoint, OpenBoundary, PerimeterCrossing,
    PerimeterEdge, PlanetaryTopology, SeamBoundary, SeamFold, ShellBoundary, Topology,
};
pub use tree::{
    Arboreal, ArborealFacet, ArcticPine, ClearedPlot, CleaveOutcome, ConicalSkirt,
    DesertSucculent, FloorLitter, ForestedFace, FullCubeGround, HewnTimberTarget,
    JungleEmergent, MeridianGround, PlantingGround, PolyhedralCrown, PrismTrunk, SectorForest,
    SectorGrove, SeededGround, SolidArboreal, SpawnSanctuaryGround, StandingTree, SteppeAcacia,
    SurfaceFind, SurfaceGround, TemperateOak, TimberDepository, TimberYield, TreeSpecies,
    VolcanicSpire, WoodlandDepository, WoodlandGround, ZonedForest,
};
pub use weather::{
    BiomeMoisture, ClimateHumidity, CloudStratum, CloudVeil, CloudVolume, ConvergenceCell,
    LocalDownpour, MoistureField, PlanarNoise, PlanetaryWeather, SmoothScalarField, StratumBand,
    TroposphereDeck, TroposphereField, WeatherSample,
};
pub use quad::Quad;

/// Interface for objects accepting generated mesh primitives.
pub trait MeshTarget {
    fn push_quad(&mut self, quad: Quad);
}

/// Interface representing a planetary gravitational field.
pub trait GravityField {
    fn pull_direction_at(&self, position: Vec3) -> Vec3;
    fn surface_up_at(&self, position: Vec3) -> Vec3;
    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        self.pull_direction_at(position) * (9.81 * dt)
    }
}

/// Interface representing an individual shell of a `QuadSphere` planetary body.
pub trait Face: Send + Sync {
    fn matches(&self, point: Vec3) -> bool;
    fn allocated_brick_bytes(&self) -> usize { 0 }
    fn surface_up(&self) -> Vec3;
    fn surface_up_at(&self, _position: Vec3) -> Vec3 { self.surface_up() }
    fn pull_direction(&self) -> Vec3 { -self.surface_up() }
    fn constrain(&self, point: Vec3, distance: f32) -> Vec3;
    fn emit_surface(&self, target: &mut dyn MeshTarget);
    fn emit_terrain(&self, target: &mut dyn MeshTarget) { self.emit_surface(target); }
    fn emit_flora(&self, _target: &mut dyn MeshTarget) {}
    fn emit_meadow_around(&self, _center: Vec3, _view_dir: Vec3, _target: &mut dyn MeshTarget) {}
    fn terrain_patch_count(&self) -> usize { 0 }
    fn emit_terrain_patch(&self, _patch_slot: usize, _target: &mut dyn MeshTarget) {}
    fn pending_terrain_patches(&self) -> bool { false }
    fn drain_terrain_patch(&mut self, _target: &mut dyn MeshTarget) -> Option<usize> { None }
    fn clear_terrain_patches(&mut self) {
        struct NullTarget;
        impl MeshTarget for NullTarget {
            fn push_quad(&mut self, _quad: Quad) {}
        }
        let mut sink = NullTarget;
        while self.drain_terrain_patch(&mut sink).is_some() {}
    }
    fn meadow_center_sector(&self, _center: Vec3) -> (i32, i32) { (0, 0) }
    fn meadow_sector_revision(&self, _gsu: i32, _gsv: i32) -> u32 { 0 }
    fn emit_meadow_sector(&self, _gsu: i32, _gsv: i32, _lod: u8, _target: &mut dyn MeshTarget) {}
    fn canopy_drag(&self, _point: Vec3) -> f32 { 0.0 }
    fn excavate(&mut self, eye: Vec3, direction: Vec3) -> bool { self.harvest(eye, direction).is_some() }
    fn harvest(&mut self, _eye: Vec3, _direction: Vec3) -> Option<BlockMedium> { None }
    fn loosen(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> Option<BlockMedium> { None }
    fn surface_yields_to(&self, _eye: Vec3, _direction: Vec3, _edge: MiningEdge) -> Option<bool> { None }
    fn aimed_surface_medium(&self, _eye: Vec3, _direction: Vec3) -> Option<BlockMedium> { None }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome { CleaveOutcome::Missed }
    fn forage_timber(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind { SurfaceFind::EmptyHanded }
    fn cast_litter(&mut self, _eye: Vec3, _direction: Vec3, _find: SurfaceFind) -> bool { false }
    fn plan_molded_block(&self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> Option<PlannedMount> { None }
    fn place_molded_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool { false }
    fn place_scaled_module(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _spec: ModuleSpec) -> bool { false }
    fn adjust_rig(&mut self, _eye: Vec3, _direction: Vec3, _delta_steps: i32, _rotate_quarter: bool) -> bool { false }
    fn lower_rig(&mut self, _eye: Vec3, _direction: Vec3) -> bool { false }
    fn board_vehicle(&mut self, _eye: Vec3, _direction: Vec3, _holding_block: bool) -> Option<VehicleAssembly> { None }
    fn park_vehicle(&mut self, _assembly: VehicleAssembly) -> bool { false }
    fn capture_vehicle_blueprint(&self, _eye: Vec3, _direction: Vec3) -> Option<CraftBlueprint> { None }
    fn aimed_empty_rig(&self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3) -> bool { false }
    fn build_rig_blueprint(&mut self, _eye: Vec3, _direction: Vec3, _blueprint: &CraftBlueprint) -> bool { false }
    fn present_vessels(&self, _sink: &mut dyn VesselSink) {}
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { None }
    fn collect_medium_sites(&self, _medium: BlockMedium, _out: &mut Vec<(Vec3, Vec3)>) {}
    fn collect_spawn_points(&self, out: &mut Vec<Vec3>) {
        let mut sites = Vec::new();
        self.collect_medium_sites(BlockMedium::SpawnStone, &mut sites);
        out.extend(sites.into_iter().map(|(p, _)| p));
    }
    fn step_surface(&mut self, _dt: f32) -> bool { false }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.step_surface(dt) }
    fn step_flora(&mut self, _dt: f32) -> bool { false }
    fn active_simulation(&self) -> bool { false }
    fn lethal_at(&self, _point: Vec3) -> bool { false }
    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 { point }
    fn accept_disclosure(&mut self, _stratum: ExposedStratum) {}
    fn drain_disclosures(&mut self, _out: &mut Vec<ExposedStratum>) {}
    fn capture_sector_snapshot(&self, face_idx: u8, gsu: i32, gsv: i32) -> SectorSnapshot {
        SectorSnapshot::new(face_idx, (gsu, gsv), 0, SectorColumnState::untouched())
    }
    fn apply_sector_snapshot(&mut self, _snapshot: &SectorSnapshot) -> bool { false }
    fn collect_modified_sectors(&self, _out: &mut Vec<(i32, i32)>) {}
}
