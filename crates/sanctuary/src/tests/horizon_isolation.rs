use glam::{Quat, Vec3};
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CardinalTurn, CompanionPortal, CompanionPresence,
    CraftIdentity, CraftPose, DrivenCraft, MoldArchetype, PlanetaryWorld, PresenceAttire,
    PresenceKinematics, SolarProportions, VehicleAxes, WorldEarthworks, WorldFact,
};
use crate::{
    CrackNotice, HorizonStream, HostedPlanet, JoinTarget, MtuBudgetAudience, PulseAudience,
    RemotePlanet, SanctuaryEnvelope, SanctuaryHost, SectorCracks, SectorGridAudience,
    SpatialFactIndex, VelocityScaledSight, INTIMATE_NEIGHBOR_CAP, NEARBY_SIGHT_METERS,
    PACKET_COMPANION_CAP,
};

fn peer_at(slot: u32, pos: Vec3) -> CompanionPresence {
    CompanionPresence::new(
        slot,
        PresenceKinematics::new(pos, Quat::IDENTITY, Quat::IDENTITY, (1.5, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::squire(),
    )
}

#[test]
fn test_sector_grid_audience_and_velocity_scaled_sight_isolate_climate_zones() {
    let half = 512.0_f32;
    let alice_temperate = peer_at(2, Vec3::new(0.0, half, 0.0));
    let bob_nearby = peer_at(3, Vec3::new(40.0, half, 12.0));
    let charlie_far_temperate = peer_at(4, Vec3::new(320.0, half, 0.0));
    let diana_desert = peer_at(5, Vec3::new(-half, 0.0, 0.0));
    let roster = [alice_temperate, bob_nearby, charlie_far_temperate, diana_desert];

    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let descending_rocket = DrivenCraft::new(
        9,
        CraftIdentity::new(77),
        CraftPose::parked(Vec3::new(20.0, half + 950.0, 10.0), axes, Vec3::Z),
    );
    let volcanic_rover = DrivenCraft::new(
        10,
        CraftIdentity::new(88),
        CraftPose::parked(Vec3::new(0.0, -half - 2.0, 0.0), axes, Vec3::Z),
    );
    let crafts = [(descending_rocket, 350.0_f32), (volcanic_rover, 400.0_f32)];

    let audience = MtuBudgetAudience::new(
        VelocityScaledSight::new(
            SectorGridAudience::indexed(&roster, NEARBY_SIGHT_METERS, half),
            &crafts,
            half,
        ),
        INTIMATE_NEIGHBOR_CAP,
        PACKET_COMPANION_CAP,
    );

    let mut cursor = 0usize;
    let (mut peers, mut seen_crafts) = (Vec::new(), Vec::new());
    audience.select_for(2, Vec3::new(0.0, half, 0.0), &mut cursor, &mut peers, &mut seen_crafts);
    assert_eq!(peers.len(), 1, "Only nearby Temperate neighbor must be selected");
    assert!(peers[0].same_slot(3));
    assert_eq!(
        seen_crafts.len(),
        1,
        "High-speed rocket descending in Temperate sky must be visible while opposite Volcanic craft stays culled"
    );
    assert_eq!(seen_crafts[0].identity(), CraftIdentity::new(77));
}

#[test]
fn test_sector_cracks_and_calm_wildlife_prevent_cross_zone_leakage() {
    let mut cracks = SectorCracks::empty();
    cracks.record(2, CrackNotice::new(Vec3::new(2.0, 512.0, 0.0), Vec3::NEG_Y, 2));
    for i in 0..24 {
        cracks.record(3, CrackNotice::new(Vec3::new((i as f32) * 0.5, -512.0, 0.0), Vec3::Y, 3));
    }
    let temperate_seen = cracks.collect_near(Vec3::new(0.0, 512.0, 0.0), 99, NEARBY_SIGHT_METERS);
    assert_eq!(
        temperate_seen.len(),
        1,
        "24 simultaneous dig cracks in opposite ClimateZone must never evict Temperate's local crack"
    );

    let mut calm_bytes = Vec::new();
    let calm_env = SanctuaryEnvelope::HostPulse {
        total_facts: 0,
        cracks: Vec::new(),
        companions: Vec::new(),
        crafts: Vec::new(),
        wildlife: world::WildlifePulse::calm(),
    };
    calm_env.inscribe(&mut calm_bytes);
    assert_eq!(
        calm_bytes.len(),
        8,
        "HostPulse outside wildlife earshot must omit all 93 wildlife bytes (0 extra bytes on wire)"
    );
    assert_eq!(SanctuaryEnvelope::decode(&calm_bytes), Some(calm_env));
}

#[test]
fn test_horizon_chronicle_isolates_climate_zones_and_streams_on_arrival() {
    let prop = SolarProportions::true_scale(1_024);
    let half = prop.earth_half();
    let host_world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::barren_beveled(1_024, 1.0));
    let raw_host = SanctuaryHost::bind_cosmos("127.0.0.1:0", host_world, prop).expect("bind 1024 host");
    let addr = raw_host.local_addr().expect("addr");
    let mut host = HostedPlanet::spawning(raw_host);

    // Host builds a stone block in Desert (-X face) and digs a pit 400m away in Temperate (+Y face)
    let desert_spot = host.constrain_to_surface(Vec3::new(-half - 2.0, 0.25, 0.25), 0.0);
    let stone = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::QuarriedStone);
    assert!(host.place_molded_surface_block(
        desert_spot + Vec3::NEG_X * 1.5,
        Vec3::X,
        desert_spot + Vec3::Z * 1.5,
        stone,
    ));
    let far_temp = host.constrain_to_surface(Vec3::new(400.25, half + 2.0, 0.25), 0.0);
    assert!(host.harvest_surface(far_temp + Vec3::Y * 1.5, Vec3::NEG_Y).is_some());

    // Alice joins at Temperate origin (0, half, 0)
    let mut alice = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("alice connect");
    let temp_origin = alice.constrain_to_surface(Vec3::new(0.25, half + 2.0, 0.25), 0.0);
    alice.sync_presence(peer_at(0, temp_origin));
    for _ in 0..8 {
        let _ = alice.step_surface(0.04);
        let _ = host.step_surface(0.04);
    }

    // While standing at Temperate origin, neither Desert (-X) nor 400m-away Temperate edits were streamed!
    let alice_desert_before = alice.constrain_to_surface(Vec3::new(-half - 2.0, 0.25, 0.25), 0.0);
    assert!(
        (alice_desert_before.x - desert_spot.x).abs() < 0.05,
        "Player in Temperate must receive 0 facts from Desert ClimateZone"
    );
    let alice_far_before = alice.constrain_to_surface(Vec3::new(400.25, half + 2.0, 0.25), 0.0);
    assert!(
        (alice_far_before.y - far_temp.y).abs() < 0.05,
        "Player at Temperate origin must receive 0 facts from 400m away"
    );

    // Now Alice walks over to Desert (-X) -> Desert's block streams automatically upon entering horizon!
    alice.sync_presence(peer_at(0, desert_spot + Vec3::Z * 1.5));
    for _ in 0..8 {
        let _ = alice.step_surface(0.04);
        let _ = host.step_surface(0.04);
    }
    let alice_desert_after = alice.constrain_to_surface(Vec3::new(-half - 2.0, 0.25, 0.25), 0.0);
    assert!(
        alice_desert_after.x < desert_spot.x - 0.35,
        "Upon entering Desert horizon, Desert's placed stone block must stream cleanly (before={}, after={})",
        desert_spot.x,
        alice_desert_after.x
    );

    // Unused direct unit check on SpatialFactIndex + HorizonStream empty state
    let mut idx = SpatialFactIndex::empty();
    let store = crate::ChronicleLog::new(vec![WorldFact::Harvested { eye: Vec3::new(0.0, half, 0.0), direction: Vec3::NEG_Y }]);
    idx.sync_with(&store, half);
    let mut stream = HorizonStream::empty();
    stream.gather_horizon(Vec3::new(0.0, -half, 0.0), &[], &idx, prop, 0.0);
    assert_eq!(stream.total(), 0);
}
