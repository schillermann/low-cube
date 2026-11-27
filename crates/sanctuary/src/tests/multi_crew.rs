use glam::{Quat, Vec3};
use world::{
    BlockMedium, BlockMold, CelestialSystem, CompanionPortal, CompanionPresence, CosmicFrame,
    CraftBlueprint, CraftPose, DrivenCraft, PilotCraft, PlanetaryWorld, PresenceAttire,
    PresenceKinematics, SolarProportions, SurfaceFrame, VehicleAxes, VehiclePart, VesselBerth,
    WorldEarthworks,
};
use crate::{
    CircularStrideCurve, GlideClock, HostedPlanet, JoinTarget, PresenceCurve, RemotePlanet,
    SanctuaryEnvelope, SanctuaryHost, VesselDeckCurve, WaypointCurve,
};

fn make_presence(slot: u32, pos: Vec3, s_rot: Quat, v_rot: Quat) -> CompanionPresence {
    CompanionPresence::new(
        slot,
        PresenceKinematics::new(pos, s_rot, v_rot, (1.45, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::astronaut(),
    )
}

#[test]
fn test_vessel_deck_curve_bijection_and_zero_overhead_planetary_wire_footprint() {
    let hull_rot = Quat::from_euler(glam::EulerRot::YXZ, 1.85, 0.42, -0.35).normalize();
    let pose = CraftPose::new(
        Vec3::new(-1_240.0, 880.0, 2_110.0),
        VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
        Vec3::Z,
        (6.0, 0.0),
    )
    .rotated(hull_rot);
    let world_crew = make_presence(
        4,
        pose.world_point_at(Vec3::new(0.5, 1.35, -1.25)),
        (hull_rot * Quat::from_rotation_y(0.6)).normalize(),
        (hull_rot * Quat::from_rotation_x(-0.2)).normalize(),
    );
    let local_crew = world_crew.localized_to_hull(pose);
    let local_pt = local_crew.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
    assert!(
        (local_pt - Vec3::new(0.5, 1.35, -1.25)).length() < 1e-4,
        "localized_to_hull must extract exact cabin coordinates, got {local_pt:?}"
    );

    let curve = VesselDeckCurve::new(
        CircularStrideCurve::new(
            WaypointCurve::new(local_crew, local_crew, GlideClock::settled(1.0)),
            (local_crew, local_crew),
            GlideClock::settled(1.0),
        ),
        pose,
    );
    let folded = curve.sample_at(1.0);
    let folded_pt = folded.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
    let expected_pt = world_crew.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
    assert!(
        (folded_pt - expected_pt).length() < 1e-3,
        "VesselDeckCurve must fold hull-local crew back onto rotated 6-DOF hull with < 1mm error"
    );

    // Verify 0 extra wire bytes for planetary GuestPulse on Earth
    let mut earth_pulse = Vec::new();
    let env_earth = SanctuaryEnvelope::GuestPulse {
        ack_seq: 7,
        presence: world_crew,
        craft: PilotCraft::Pedestrian,
        frame: CosmicFrame::earth(),
        berth: VesselBerth::planetary(),
        sky_watch: false,
    };
    env_earth.inscribe(&mut earth_pulse);
    assert_eq!(SanctuaryEnvelope::decode(&earth_pulse), Some(env_earth));
}

#[test]
fn test_multi_crew_epstein_ship_at_200_km_per_sec_zero_cabin_slip_and_intercom_chat() {
    let prop = SolarProportions::true_scale(128);
    let earth_half = prop.earth_half();
    let host_world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let raw_host = SanctuaryHost::bind_cosmos("127.0.0.1:0", host_world, prop).expect("bind host");
    let addr = raw_host.local_addr().expect("addr");
    let mut host = HostedPlanet::spawning(raw_host);

    let mut alice = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("alice");
    let mut bob = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("bob");
    let mut earth_peer = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("earth_peer");

    // Commission an Epstein ship blueprint and claim its helm for Pilot Alice via HostChronicle
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let stand = host.constrain_to_surface(Vec3::new(0.25, earth_half + 4.0, 0.25), 0.0);
    let crane_eye = Vec3::new(0.25, stand.y + 1.5, 1.25);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    let av_pos = Vec3::new(0.25, stand.y, 1.75);
    assert!(host.place_molded_surface_block(
        crane_eye,
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));
    let parts = vec![
        VehiclePart::new((0, 0), 0.0, BlockMold::cube().in_medium(BlockMedium::CabinSeat).atom_for(Vec3::NEG_Z, &frame)),
        VehiclePart::new((0, 1), 0.0, BlockMold::cube().in_medium(BlockMedium::FlightConsole).atom_for(Vec3::NEG_Z, &frame)),
        VehiclePart::new((1, 0), 0.0, BlockMold::cube().in_medium(BlockMedium::CabinSeat).atom_for(Vec3::NEG_Z, &frame)),
        VehiclePart::new((0, -1), 0.0, BlockMold::cube().in_medium(BlockMedium::FusionTorch).atom_for(Vec3::NEG_Z, &frame)),
    ];
    let bp = CraftBlueprint::commission(Vec3::new(1.25, stand.y + 2.0, 1.25), parts);
    assert!(host.build_rig_blueprint(crane_eye, crane_dir, &bp));

    let seat_eye = Vec3::new(1.25, stand.y + 2.2, 0.25);
    alice.sync_presence(make_presence(0, seat_eye, Quat::IDENTITY, Quat::IDENTITY));
    earth_peer.sync_presence(make_presence(0, stand, Quat::IDENTITY, Quat::IDENTITY));
    for _ in 0..10 {
        let _ = alice.step_surface(0.03);
        let _ = bob.step_surface(0.03);
        let _ = earth_peer.step_surface(0.03);
        let _ = host.step_surface(0.03);
    }
    let console_dir = (Vec3::new(1.25, stand.y + 1.4, 0.75) - seat_eye).normalize();
    let boarded = alice
        .board_vehicle(seat_eye, console_dir, true)
        .or_else(|| alice.board_vehicle(Vec3::new(1.25, stand.y + 2.6, 0.75), Vec3::NEG_Y, true))
        .expect("Alice must board commissioned rig spacecraft");
    let actual_id = boarded.identity();
    for _ in 0..8 {
        let _ = alice.step_surface(0.03);
        let _ = bob.step_surface(0.03);
        let _ = host.step_surface(0.03);
    }

    // Jump Alice & Bob into DeepSpace (1.8 AU) flying at 200,000 m/s!
    let deep_probe = Vec3::new(1.8 * (prop.astronomical_unit() as f32), 0.0, 0.0);
    let _ = alice.reconcile_cosmic_frame(deep_probe, Vec3::new(200_000.0, 0.0, 0.0));
    let _ = bob.reconcile_cosmic_frame(deep_probe, Vec3::new(200_000.0, 0.0, 0.0));
    assert!(matches!(alice.cosmic_frame(), CosmicFrame::DeepSpace(_)));

    let cabin_offset_bob = Vec3::new(0.5, 1.25, -0.5);
    let cabin_offset_alice = Vec3::new(0.0, 1.12, 0.0);
    let base_pose = CraftPose::new(
        Vec3::new(-800.0, 40.0, 20.0),
        VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
        Vec3::Z,
        (8.0, 0.0),
    );
    alice.sync_pilot_craft(PilotCraft::Driving(DrivenCraft::new(0, actual_id, base_pose)));
    alice.sync_berth_presence(
        make_presence(0, base_pose.world_point_at(cabin_offset_alice), Quat::IDENTITY, Quat::IDENTITY)
            .localized_to_hull(base_pose),
        VesselBerth::aboard(actual_id),
    );
    bob.sync_berth_presence(
        make_presence(0, base_pose.world_point_at(cabin_offset_bob), Quat::IDENTITY, Quat::IDENTITY)
            .localized_to_hull(base_pose),
        VesselBerth::aboard(actual_id),
    );
    for _ in 0..8 {
        let _ = alice.step_surface(0.02);
        let _ = bob.step_surface(0.02);
        let _ = earth_peer.step_surface(0.02);
        let _ = host.step_surface(0.02);
    }

    // CRITICAL TEST: Pilot Alice now leaps +1,667 m (200 km/s * 8.33 ms) AND flips 180° locally BEFORE any network packet from Bob!
    let flip = Quat::from_rotation_y(std::f32::consts::PI);
    let flipped_pose = base_pose.rotated(flip).with_origin(Vec3::new(867.0, 40.0, 20.0));
    alice.sync_pilot_craft(PilotCraft::Driving(DrivenCraft::new(0, actual_id, flipped_pose)));

    let mut alice_seen = Vec::new();
    alice.active_companions(&mut alice_seen);
    assert_eq!(alice_seen.len(), 1, "Pilot Alice must see Crewmate Bob aboard the ship");
    let rendered_bob = alice_seen[0].transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
    let expected_bob = flipped_pose.world_point_at(cabin_offset_bob);
    assert!(
        (rendered_bob - expected_bob).length() < 1e-3,
        "At 200 km/s + 180° flip, Crewmate Bob must stay pinned to Alice's instantaneous hull with 0.000 m lag! Got {rendered_bob:?} vs {expected_bob:?}"
    );

    // Cabin intercom / proximity chat at 200 km/s in DeepSpace:
    bob.utter_nearby("Flip and burn complete!");
    for _ in 0..6 {
        let _ = bob.step_surface(0.02);
        let _ = host.step_surface(0.02);
        let _ = alice.step_surface(0.02);
        let _ = earth_peer.step_surface(0.02);
    }
    let mut alice_chat = Vec::new();
    alice.drain_utterances(&mut alice_chat);
    assert_eq!(alice_chat.len(), 1, "Pilot Alice must hear Crewmate Bob's cabin proximity chat at 200 km/s");

    let mut earth_chat = Vec::new();
    let mut earth_comps = Vec::new();
    earth_peer.drain_utterances(&mut earth_chat);
    earth_peer.active_companions(&mut earth_comps);
    assert!(earth_chat.is_empty() && earth_comps.is_empty(), "Earth observer must receive 0 bytes from 1.8 AU crew");
}
