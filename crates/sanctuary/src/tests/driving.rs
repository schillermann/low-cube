use glam::{Quat, Vec3};
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CompanionPresence, CraftBlueprint, CraftPose,
    DrivenCraft, MeshTarget, PilotCraft, PlanetaryWorld, PresenceAttire, PresenceKinematics, Quad,
    SurfaceFrame, VehicleAxes, VehiclePart, WorldEarthworks,
};
use crate::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};

struct QuadCounter(usize);
impl MeshTarget for QuadCounter {
    fn push_quad(&mut self, _quad: Quad) {
        self.0 += 1;
    }
}

#[test]
fn test_large_blueprint_roundtrip_and_80_byte_pose_wire_budget() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let mut parts = Vec::with_capacity(96);
    parts.push(VehiclePart::new(
        (0, 0),
        0.0,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(Vec3::NEG_Z, &frame),
    ));
    for idx in 1..96i32 {
        parts.push(VehiclePart::new(
            (idx % 12, idx / 12),
            0.0,
            BlockMold::cube().in_medium(BlockMedium::PlanedPlanks).shape_for(Vec3::NEG_Z, &frame),
        ));
    }
    let bp = CraftBlueprint::commission(Vec3::new(1.25, 16.0, 1.25), parts);
    let mut buf = Vec::new();
    bp.inscribe(&mut buf);
    let (decoded, used) = CraftBlueprint::decode(&buf).expect("decode 96-part blueprint");
    assert_eq!(used, buf.len());
    assert_eq!(decoded, bp, "Blueprints exceeding 64 blocks must serialize without truncation");

    let pose = CraftPose::new(
        Vec3::new(4.0, 16.0, 2.0),
        VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
        Vec3::X,
        (3.2, -0.5),
    );
    let driven = DrivenCraft::new(2, bp.identity(), pose);
    let mut pose_buf = Vec::new();
    driven.inscribe(&mut pose_buf);
    assert_eq!(pose_buf.len(), 80, "Live DrivenCraft pulse wire size must be O(1) 80 bytes");
}

#[test]
fn test_multiplayer_vehicle_ownership_lock_passenger_seat_and_smooth_glide() {
    let host = SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(BiomedCubeWorld::barren_beveled(32, 1.0)),
        32,
        false,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("host addr");
    let mut host_world = HostedPlanet::new(host);
    let mut guest_a = RemotePlanet::new(
        Box::new(BiomedCubeWorld::barren_beveled(32, 1.0)),
        crate::RemoteLink::bind(JoinTarget::Direct(addr)).expect("bind guest a"),
        std::collections::VecDeque::new(),
    );
    let mut guest_b = RemotePlanet::new(
        Box::new(BiomedCubeWorld::barren_beveled(32, 1.0)),
        crate::RemoteLink::bind(JoinTarget::Direct(addr)).expect("bind guest b"),
        std::collections::VecDeque::new(),
    );
    for _ in 0..8 {
        let _ = guest_a.step_surface(0.03);
        let _ = guest_b.step_surface(0.03);
        let _ = host_world.step_surface(0.03);
    }
    assert!(guest_a.connected() && guest_b.connected());

    let g0 = host_world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av_pos = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    let seat_mold = BlockMold::cube().in_medium(BlockMedium::CabinSeat);
    let wheel_mold = BlockMold::cube().next_form().in_medium(BlockMedium::CarWheel);

    assert!(host_world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));
    assert!(host_world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::PedalCluster),
    ));
    assert!(host_world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    for (eye, dir, mold) in [
        (Vec3::new(1.25, g0 + 2.40, 0.25), Vec3::NEG_Y, BlockMold::cube().turn_clockwise().in_medium(BlockMedium::SteeringWheel)),
        (Vec3::new(1.25, g0 + 1.25, 0.75), Vec3::NEG_Z, wheel_mold),
        (Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, seat_mold),
        (Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, wheel_mold),
        (Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, wheel_mold.next_form()),
        (Vec3::new(1.25, g0 + 2.35, 0.75), Vec3::NEG_Y, seat_mold),
    ] {
        assert!(host_world.place_molded_surface_block(eye, dir, av_pos, mold));
    }
    for _ in 0..8 {
        let _ = host_world.step_surface(0.03);
        let _ = guest_a.step_surface(0.03);
        let _ = guest_b.step_surface(0.03);
    }

    let seat_pos = Vec3::new(1.25, g0 + 2.2, 1.25);
    let steer_dir = (Vec3::new(1.25, g0 + 1.75, 0.25) - seat_pos).normalize();
    let pres = CompanionPresence::new(
        0,
        PresenceKinematics::new(seat_pos, Quat::IDENTITY, Quat::IDENTITY, (0.6, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::squire(),
    );
    guest_a.sync_presence(pres);
    guest_b.sync_presence(pres);

    // Simultaneous helm click by Guest A and Guest B within the same network window:
    let mut assembly_a = guest_a.board_vehicle(seat_pos, steer_dir, true).expect("guest A boards");
    let assembly_b = guest_b.board_vehicle(seat_pos, steer_dir, true).expect("guest B optimistic board");
    assert_eq!(assembly_a.identity(), assembly_b.identity(), "Deterministic CraftIdentity must match");
    let craft_id = assembly_a.identity();

    for _ in 0..8 {
        let _ = guest_a.step_surface(0.03);
        let _ = guest_b.step_surface(0.03);
        let _ = host_world.step_surface(0.03);
    }
    let a_revoked = guest_a.helm_revoked(craft_id);
    let b_revoked = guest_b.helm_revoked(craft_id);
    assert_ne!(a_revoked, b_revoked, "Exactly one simultaneous driver must keep the helm while the loser is revoked");

    if a_revoked {
        std::mem::swap(&mut guest_a, &mut guest_b);
        assembly_a = assembly_b;
    }

    let mut samples_x = Vec::new();
    for tick in 0..36 {
        let dt = 1.0 / 120.0;
        let next = assembly_a.origin() + Vec3::X * (14.0 * dt);
        let ground = guest_a.constrain_to_surface(next + Vec3::Y * 4.0, 0.0);
        assembly_a.relocate(ground, Vec3::Y);
        guest_a.sync_presence(CompanionPresence::new(
            0,
            PresenceKinematics::new(ground + Vec3::Y * 1.12, Quat::IDENTITY, Quat::IDENTITY, (0.6, 0.0)),
            (0.0, 0.0, 0.0, 0.0),
            PresenceAttire::squire(),
        ));
        guest_a.sync_pilot_craft(PilotCraft::Driving(DrivenCraft::new(
            0,
            craft_id,
            assembly_a.pose().with_motion((tick as f32) * 0.4, 0.25),
        )));
        let _ = guest_a.step_surface(dt);
        let _ = host_world.step_surface(dt);
        let _ = guest_b.step_surface(dt);
        host_world.blend_companions(0.5 * dt);

        if tick == 18 {
            let mut quads = QuadCounter(0);
            host_world.emit_active_crafts(&mut quads);
            assert!(quads.0 > 10, "BlueprintCatalog must render 80-byte DrivenCraft poses");
            let mut active = Vec::new();
            host_world.active_crafts(&mut active);
            let live_origin = active[0].pose().origin();
            let roof_probe = Vec3::new(live_origin.x, g0 + 5.0, live_origin.z);
            assert!(
                host_world.constrain_to_surface(roof_probe, 0.0).y > g0 + 0.4,
                "Moving vehicle must physically support colliding/standing avatars"
            );
            let pass_aim_eye = Vec3::new(live_origin.x, g0 + 2.4, 0.75);
            assert!(
                host_world.board_passenger_seat(pass_aim_eye, Vec3::NEG_Y).is_some(),
                "Second CabinSeat on moving vehicle must be boardable by passengers"
            );
        }
        let mut active = Vec::new();
        host_world.active_crafts(&mut active);
        if tick >= 6 && !active.is_empty() {
            samples_x.push(active[0].pose().origin().x);
        }
    }
    let smooth_steps = samples_x.windows(2).filter(|w| (w[1] - w[0]).abs() > 1e-4 && (w[1] - w[0]).abs() < 0.25).count();
    assert!(smooth_steps >= 15);

    let final_x = assembly_a.origin().x;
    assert!(guest_a.park_vehicle(assembly_a));
    guest_a.sync_pilot_craft(PilotCraft::Pedestrian);
    for _ in 0..8 {
        let _ = guest_a.step_surface(0.03);
        let _ = host_world.step_surface(0.03);
        let _ = guest_b.step_surface(0.03);
    }
    assert!(host_world.constrain_to_surface(Vec3::new(final_x, 22.0, 0.75), 0.0).y > g0 + 0.5);
    assert!(guest_b.constrain_to_surface(Vec3::new(final_x, 22.0, 0.75), 0.0).y > g0 + 0.5);
}
