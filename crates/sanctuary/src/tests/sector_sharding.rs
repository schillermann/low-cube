use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use glam::{Quat, Vec3};
use world::{
    BlockMedium, BlockMold, CardinalTurn, CompanionPortal, CompanionPresence, CosmicFrame, Face,
    MeshTarget, ModifiedSlices, MoldArchetype, PilotCraft, PlanetaryWorld, PresenceAttire,
    PresenceKinematics, SectorColumnState, SectorSnapshot, SolarProportions, VesselBerth,
    WorldEarthworks, WorldFact, ZoneShards,
};
use crate::{
    ChronicleLog, ChronicleStore, CompactedChronicle, DiskVaultChronicle, GuestWard,
    HandshakeIdentity, InboundBatch, JoinTarget, ParallelInbox, PulseInbox, RemoteLink,
    RemotePlanet, SanctuaryEnvelope, SanctuaryHost, SequentialInbox, SnapshottedChronicle,
    WardVitals,
};
use super::build_ore_test_planet;

struct CountingFace {
    inner: Box<dyn Face>,
    steps: Arc<AtomicUsize>,
}

impl Face for CountingFace {
    fn matches(&self, point: Vec3) -> bool { self.inner.matches(point) }
    fn surface_up(&self) -> Vec3 { self.inner.surface_up() }
    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 { self.inner.constrain(point, distance) }
    fn emit_surface(&self, target: &mut dyn MeshTarget) { self.inner.emit_surface(target); }
    fn harvest(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> { self.inner.harvest(eye, direction) }
    fn step_surface(&mut self, dt: f32) -> bool {
        self.steps.fetch_add(1, Ordering::Relaxed);
        self.inner.step_surface(dt)
    }
    fn step_flora(&mut self, dt: f32) -> bool { self.inner.step_flora(dt) }
    fn active_simulation(&self) -> bool { self.inner.active_simulation() }
}

#[test]
fn test_sector_snapshot_compacts_raycasts_into_raycast_free_wal_and_preserves_climate_isolation() {
    let wal_path = std::env::temp_dir().join(format!(
        "low_cube_sector_snap_{}_{}.wal",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let prop = SolarProportions::true_scale(32);

    {
        let mut world: Box<dyn PlanetaryWorld> = Box::new(build_ore_test_planet(true));
        let disk = DiskVaultChronicle::replay_and_open(
            &wal_path,
            CompactedChronicle::wrapping(ChronicleLog::empty()),
            world.as_mut(),
        )
        .expect("open wal");
        let store = Box::new(SnapshottedChronicle::with_threshold(disk, 12));
        let host = SanctuaryHost::bind_cosmos_with_store("127.0.0.1:0", world, prop, store)
            .expect("bind snapshot host");
        let mut hosted = crate::HostedPlanet::new(host);

        for idx in 0..4 {
            let x = 0.25 + (idx as f32) * 0.5;
            for _ in 0..5 {
                let g = hosted.constrain_to_surface(Vec3::new(x, 28.0, 0.25), 0.0);
                let _ = hosted.harvest_surface(g + Vec3::Y * 1.3, Vec3::NEG_Y);
            }
        }
        let build_spot = Vec3::new(0.25, 28.0, 1.25);
        for _ in 0..8 {
            let g = hosted.constrain_to_surface(build_spot, 0.0);
            assert!(hosted.place_molded_surface_block(
                g + Vec3::Y * 1.4,
                Vec3::NEG_Y,
                Vec3::new(3.25, g.y, 1.25),
                BlockMold::cube().in_medium(BlockMedium::PlanedPlanks),
            ));
            let top = hosted.constrain_to_surface(build_spot, 0.0);
            assert!(hosted.harvest_surface(top + Vec3::Y * 1.2, Vec3::NEG_Y).is_some());
        }
        let stair_base = hosted.constrain_to_surface(Vec3::new(1.25, 28.0, 1.25), 0.0);
        let aim_south = Vec3::new(0.0, -1.0, -0.15).normalize();
        assert!(hosted.place_molded_surface_block(
            stair_base + Vec3::new(0.0, 1.4, 0.21),
            aim_south,
            Vec3::new(1.25, stair_base.y, 3.25),
            BlockMold::with_medium(MoldArchetype::SteppedStair, CardinalTurn::Forward, BlockMedium::QuarriedStone),
        ));
        let cam_base = hosted.constrain_to_surface(Vec3::new(2.25, 28.0, 1.25), 0.0);
        assert!(hosted.place_molded_surface_block(
            cam_base + Vec3::new(0.0, 1.4, 0.21),
            aim_south,
            Vec3::new(2.25, cam_base.y, 3.25),
            BlockMold::cube().in_medium(BlockMedium::FieldCamera).next_form().next_form(),
        ));
        let rig_base = hosted.constrain_to_surface(Vec3::new(2.75, 28.0, 2.25), 0.0);
        let aim_east = Vec3::new(0.15, -1.0, 0.0).normalize();
        assert!(hosted.place_molded_surface_block(
            rig_base + Vec3::new(-0.21, 1.4, 0.0),
            aim_east,
            Vec3::new(1.25, rig_base.y, 2.25),
            BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        ));
        let _ = hosted.step_surface(0.05);
    }

    let raw_wal_bytes = std::fs::metadata(&wal_path).expect("raw wal").len();
    let mut restarted_world: Box<dyn PlanetaryWorld> = Box::new(build_ore_test_planet(true));
    let restarted_disk = DiskVaultChronicle::replay_and_open(
        &wal_path,
        CompactedChronicle::wrapping(ChronicleLog::empty()),
        restarted_world.as_mut(),
    )
    .expect("replay and compact wal");
    let compacted_total = restarted_disk.total();
    let compacted_wal_bytes = std::fs::metadata(&wal_path).expect("compacted wal").len();
    assert!(
        compacted_total <= 4 && compacted_wal_bytes * 2 < raw_wal_bytes,
        "Restart WAL compaction must fold 40+ raycasts into <= 4 SectorSnapshots ({compacted_total} facts, {compacted_wal_bytes}B vs {raw_wal_bytes}B)"
    );

    let expected_pit_y = restarted_world.constrain_to_surface(Vec3::new(0.25, 28.0, 0.25), 0.0).y;
    let expected_stair_y = restarted_world.constrain_to_surface(Vec3::new(1.25, 28.0, 1.25), 0.0).y;
    let expected_tripod = restarted_world.placed_camera_tripod().expect("restarted camera tripod");

    let mut host = SanctuaryHost::bind_cosmos_with_store(
        "127.0.0.1:0",
        restarted_world,
        prop,
        Box::new(SnapshottedChronicle::wrapping(restarted_disk)),
    )
    .expect("bind restarted host");
    let addr = host.local_addr().expect("addr");

    let mut temp_guest = RemotePlanet::new(
        Box::new(build_ore_test_planet(false)),
        RemoteLink::bind(JoinTarget::Direct(addr)).expect("temp link"),
        std::collections::VecDeque::new(),
    );
    let mut volc_guest = RemotePlanet::new(
        Box::new(build_ore_test_planet(false)),
        RemoteLink::bind(JoinTarget::Direct(addr)).expect("volc link"),
        std::collections::VecDeque::new(),
    );
    temp_guest.sync_presence(CompanionPresence::new(
        0,
        PresenceKinematics::new(Vec3::new(0.5, 16.0, 0.5), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::squire(),
    ));
    volc_guest.sync_presence(CompanionPresence::new(
        0,
        PresenceKinematics::new(Vec3::new(0.5, -16.0, 0.5), Quat::from_rotation_x(std::f32::consts::PI), Quat::IDENTITY, (0.0, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::squire(),
    ));
    for _ in 0..10 {
        let _ = temp_guest.step_surface(0.05);
        let _ = volc_guest.step_surface(0.05);
        let _ = host.tick(0.05);
    }

    let synced_pit_y = temp_guest.constrain_to_surface(Vec3::new(0.25, 28.0, 0.25), 0.0).y;
    let synced_stair_y = temp_guest.constrain_to_surface(Vec3::new(1.25, 28.0, 1.25), 0.0).y;
    let synced_tripod = temp_guest.placed_camera_tripod().expect("synced camera tripod");
    assert!((synced_pit_y - expected_pit_y).abs() < 0.02, "Late joiner must reconstruct exact dug pit depth from SectorSnapshot");
    assert!((synced_stair_y - expected_stair_y).abs() < 0.02, "Late joiner must reconstruct sculpted stairs from SectorSnapshot");
    assert!((synced_tripod.1 - expected_tripod.1).length() < 0.02, "Late joiner must reconstruct exact 8-way FieldCamera yaw");

    let volc_untouched_y = volc_guest.constrain_to_surface(Vec3::new(0.25, 28.0, 0.25), 0.0).y;
    assert!(
        volc_untouched_y > synced_pit_y + 1.5 && volc_guest.placed_camera_tripod().is_none(),
        "Guest on Volcanic face must receive zero Temperate SectorSnapshots"
    );

    let forged = SectorSnapshot::new(0, (0, 0), u64::MAX, SectorColumnState::untouched());
    let sock = UdpSocket::bind("127.0.0.1:0").expect("rogue");
    sock.set_nonblocking(true).expect("nb");
    let rogue_k = HandshakeIdentity::generate();
    let mut knock = Vec::from(&b"LKNK"[..]);
    knock.extend_from_slice(&rogue_k.public_bytes());
    sock.send_to(&knock, addr).expect("knock");
    let _ = host.tick(0.05);
    let mut buf = [0u8; 1500];
    let _ = sock.recv_from(&mut buf).expect("lwel");
    let cid = u32::from_le_bytes(buf[36..40].try_into().expect("cid"));
    let mut seal = rogue_k.establish(host.ticket().host_key(), true);
    let (mut p, mut f) = (Vec::new(), Vec::new());
    SanctuaryEnvelope::GuestPetition { fact: WorldFact::SectorSnapshot { snapshot: forged } }.inscribe(&mut p);
    seal.seal(cid, &p, &mut f);
    sock.send_to(&f, addr).expect("forged snap");
    let _ = host.tick(0.05);
    assert!(host.world().placed_camera_tripod().is_some(), "Untrusted GuestPetition SectorSnapshot must be rejected");

    let _ = std::fs::remove_file(&wal_path);
}

#[test]
fn test_zone_shards_keep_uninhabited_climate_zones_dormant_and_parallelize_active_zones() {
    let counters: Vec<Arc<AtomicUsize>> = (0..6).map(|_| Arc::new(AtomicUsize::new(0))).collect();
    let mut faces: Vec<Box<dyn Face>> = world::ClimateZone::all()
        .into_iter()
        .enumerate()
        .map(|(i, z)| -> Box<dyn Face> {
            Box::new(CountingFace {
                inner: Box::new(world::FacetedReliefFace::hollow_in_zone(z, 16.0, 64)),
                steps: Arc::clone(&counters[i]),
            })
        })
        .collect();
    let mut slices = ModifiedSlices::clean();

    let idle_shards = ZoneShards::autonomous(0.05);
    assert_eq!(idle_shards.awake_count(&faces), 0, "All 6 uninhabited undisturbed ClimateZones must be Dormant");
    assert!(!idle_shards.step_faces(&mut faces, &mut slices));
    assert_eq!(counters.iter().map(|c| c.load(Ordering::Relaxed)).sum::<usize>(), 0);

    let active_mask = (1u8 << 0) | (1u8 << 2) | (1u8 << 3) | (1u8 << 5);
    let multi_shards = ZoneShards::new(0.05, active_mask);
    assert_eq!(multi_shards.awake_count(&faces), 4, "Exactly the 4 occupied ClimateZones must wake");
    let _ = multi_shards.step_faces(&mut faces, &mut slices);
    assert_eq!(counters[0].load(Ordering::Relaxed), 1);
    assert_eq!(counters[1].load(Ordering::Relaxed), 0, "Unoccupied Volcanic zone must remain at 0 steps");
    assert_eq!(counters[2].load(Ordering::Relaxed), 1);
    assert_eq!(counters[3].load(Ordering::Relaxed), 1);
    assert_eq!(counters[4].load(Ordering::Relaxed), 0, "Unoccupied Jungle zone must remain at 0 steps");
    assert_eq!(counters[5].load(Ordering::Relaxed), 1);

    let g = faces[1].constrain(Vec3::new(0.25, -20.0, 0.25), 0.0);
    assert!(faces[1].harvest(g + Vec3::NEG_Y * 1.3, Vec3::Y).is_some());
    assert!(faces[1].active_simulation(), "Freshly excavated Volcanic pit must wake autonomous grass regrowth");
    for _ in 0..70 {
        let _ = ZoneShards::autonomous(4.0).step_faces(&mut faces, &mut slices);
    }
    assert!(!faces[1].active_simulation(), "Once fertile pit grass regrows, uninhabited zone must return to Dormant");
}

#[test]
fn test_parallel_inbox_authenticates_24_companion_burst_in_deterministic_order() {
    let host_keys = HandshakeIdentity::generate();
    let peer: std::net::SocketAddr = "127.0.0.1:49100".parse().expect("addr");
    let mut wards: HashMap<u32, GuestWard> = HashMap::new();
    let mut grouped: HashMap<u32, Vec<(u64, std::net::SocketAddr, Vec<u8>)>> = HashMap::new();

    for i in 0..24u32 {
        let cid = i + 2;
        let gk = HandshakeIdentity::generate();
        let host_seal = host_keys.establish(gk.public_bytes(), false);
        let mut guest_seal = gk.establish(host_keys.public_bytes(), true);
        let pres = CompanionPresence::new(
            cid,
            PresenceKinematics::new(Vec3::new(i as f32, 16.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            (0.0, 0.0, 0.0, 0.0),
            PresenceAttire::squire(),
        );
        wards.insert(cid, GuestWard::new(host_seal, peer, pres, WardVitals::fresh()));

        let (mut plain, mut wire) = (Vec::new(), Vec::new());
        SanctuaryEnvelope::GuestPulse {
            ack_seq: i,
            presence: pres,
            craft: PilotCraft::Pedestrian,
            frame: CosmicFrame::earth(),
            berth: VesselBerth::planetary(),
            sky_watch: false,
        }
        .inscribe(&mut plain);
        guest_seal.seal(cid, &plain, &mut wire);
        let mut corrupted = wire.clone();
        let last = corrupted.len() - 1;
        corrupted[last] ^= 0xFF;
        grouped.insert(cid, vec![(u64::from(24 - i), peer, wire), (u64::from(100 + i), peer, corrupted)]);
    }

    let mut batches: Vec<InboundBatch<'_>> = wards
        .iter_mut()
        .filter_map(|(&cid, w)| grouped.remove(&cid).map(|raw| InboundBatch::prepare(cid, w, raw)))
        .collect();
    ParallelInbox::hardware(SequentialInbox::new()).open_batches(&mut batches);
    let ordered = InboundBatch::collect_ordered(batches);
    assert_eq!(ordered.len(), 24, "All 24 authentic AEAD frames must pass while all 24 tampered frames drop");
    assert_eq!(ordered[0].0, 25, "Reverse sequence numbers must sort back into exact arrival order (cid 25 = seq 1)");
    assert_eq!(ordered[23].0, 2, "Last arrival sequence must appear last (cid 2 = seq 24)");
}
