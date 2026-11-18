use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, ClimateZone, GravityField, PlanetaryWorld,
};
use crate::avatar::{Avatar, HandGesture, PlanetaryAvatar};
use crate::controls::{KeyMotion, KeyboardControls};

fn build_car_at_origin(world: &mut dyn PlanetaryWorld) -> f32 {
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av_pos = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::PedalCluster),
    ));
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    let wheel = BlockMold::cube().next_form().in_medium(BlockMedium::CarWheel);
    for (eye, dir, mold) in [
        (Vec3::new(1.25, g0 + 2.40, 0.25), Vec3::NEG_Y, BlockMold::cube().turn_clockwise().in_medium(BlockMedium::SteeringWheel)),
        (Vec3::new(1.25, g0 + 1.25, 0.75), Vec3::NEG_Z, wheel),
        (Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, BlockMold::cube().in_medium(BlockMedium::CabinSeat)),
        (Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, wheel),
        (Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, wheel.next_form()),
        (Vec3::new(1.25, g0 + 2.35, 0.75), Vec3::NEG_Y, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)),
    ] {
        assert!(world.place_molded_surface_block(eye, dir, av_pos, mold));
    }
    g0
}

#[test]
fn test_car_circumnavigates_all_four_90_degree_cube_seams_and_reparks_on_side_face() {
    let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
    let _ = build_car_at_origin(&mut world);
    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let cam_fwd = |a: &PlanetaryAvatar| {
        (a.view_projection_matrix(1.0).inverse().project_point3(Vec3::new(0.0, 0.0, 0.5)) - cam_pos(a)).normalize()
    };

    let mut driver = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0));
    driver.rotate_degrees(-90.0, 12.0);
    assert!(HandGesture::RightTap.perform(&mut driver, &mut world));
    assert!(driver.use_right_hand(&mut world));

    let mut gas = KeyMotion::idle();
    gas.register_key(KeyCode::KeyW, true);
    let dt = 1.0 / 120.0;
    let mut prev_eye = cam_pos(&driver);
    let mut visited_zones = [false; 6];
    let mut max_step_jump = 0.0_f32;
    let mut parked_and_reboarded_on_side = false;

    for tick in 0..1600 {
        driver.step_motion(dt, &gas, &world);
        let eye = cam_pos(&driver);
        let fwd = cam_fwd(&driver);
        assert!(eye.is_finite() && fwd.is_finite(), "Camera state must stay finite across seams at tick {tick}");
        if tick > 2 {
            let jump = (eye - prev_eye).length();
            max_step_jump = max_step_jump.max(jump);
            assert!(
                jump < 0.40,
                "Car must cross 90-degree seams smoothly without teleporting at tick {tick}: prev={prev_eye:?}, cur={eye:?}, jump={jump}"
            );
        }
        prev_eye = eye;
        let zone = ClimateZone::dominant(eye);
        visited_zones[zone.index()] = true;

        // Once well onto the +X face (Desert / Arctic / side face where x > 16.5 and |y| < 10.0), test parking & re-boarding on a vertical face!
        if !parked_and_reboarded_on_side && eye.x > 16.5 && eye.y.abs() < 8.0 {
            let up_side = world.surface_up_at(eye);
            assert!(
                up_side.dot(Vec3::X) > 0.95,
                "Surface normal on +X face must point along +X, got {up_side:?}"
            );
            assert!(
                fwd.dot(Vec3::NEG_Y) > 0.65,
                "Driver camera forward must reorient downward (-Y) along the +X face road, got {fwd:?}"
            );
            assert!(driver.use_left_hand(&mut world), "Driver must release wheel to park on +X face");
            for _ in 0..5 {
                driver.step_motion(dt, &KeyMotion::idle(), &world);
            }
            // Look slightly left/down toward the SteeringWheel cell on the parked car, then re-board and look forward again!
            driver.rotate_degrees(-75.0, 12.0);
            assert!(driver.use_right_hand(&mut world), "Driver must re-board parked car on +X face");
            prev_eye = cam_pos(&driver);
            parked_and_reboarded_on_side = true;
        }
    }

    let distinct_faces = visited_zones.into_iter().filter(|&v| v).count();
    assert!(
        distinct_faces >= 4 && parked_and_reboarded_on_side,
        "Car must traverse all 4 loop faces (+Y -> +X -> -Y -> -X) and re-board on +X: visited={visited_zones:?}, max_jump={max_step_jump}"
    );
}

#[test]
fn test_car_climbs_half_meter_slab_and_arrests_at_vertical_one_meter_wall() {
    let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
    let g0 = build_car_at_origin(&mut world);
    let av_pos = Vec3::new(-2.0, g0, 0.75);
    let stone_cube = BlockMold::cube().in_medium(BlockMedium::QuarriedStone);

    // Place a single 0.5m step block at x = 3.75 and a 1.0m two-layer vertical wall at x = 6.75 across z = 0.25..=1.25
    for z in [0.25_f32, 0.75, 1.25] {
        assert!(world.place_molded_surface_block(Vec3::new(3.75, g0 + 1.5, z), Vec3::NEG_Y, av_pos, stone_cube));
        assert!(world.place_molded_surface_block(Vec3::new(6.75, g0 + 1.5, z), Vec3::NEG_Y, av_pos, stone_cube));
        assert!(world.place_molded_surface_block(Vec3::new(6.75, g0 + 2.0, z), Vec3::NEG_Y, av_pos, stone_cube));
    }

    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let mut driver = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0));
    driver.rotate_degrees(-90.0, 12.0);
    assert!(HandGesture::RightTap.perform(&mut driver, &mut world));
    assert!(driver.use_right_hand(&mut world));

    let mut gas = KeyMotion::idle();
    gas.register_key(KeyCode::KeyW, true);
    let mut climbed_step = false;
    for _ in 0..180 {
        driver.step_motion(1.0 / 120.0, &gas, &world);
        let eye = cam_pos(&driver);
        if eye.x > 4.2 {
            climbed_step = true;
        }
        assert!(
            eye.x < 6.50 && eye.y < g0 + 2.75,
            "Car must arrest at 1.0m vertical wall without teleporting onto its roof, got {eye:?}"
        );
    }
    assert!(climbed_step, "Car must climb the 0.5m step block smoothly before arresting at the 1.0m wall");
}

#[test]
fn test_multiplayer_driver_and_passenger_traverse_90_degree_seam_together() {
    use sanctuary::{HostedPlanet, JoinTarget, RemoteLink, RemotePlanet, SanctuaryHost};

    let host = SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(BiomedCubeWorld::barren_beveled(32, 1.0)),
        32,
        false,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("addr");
    let mut host_world = HostedPlanet::new(host);
    let mut guest_world = RemotePlanet::new(
        Box::new(BiomedCubeWorld::barren_beveled(32, 1.0)),
        RemoteLink::bind(JoinTarget::Direct(addr)).expect("guest link"),
        std::collections::VecDeque::new(),
    );
    for _ in 0..8 {
        let _ = guest_world.step_surface(0.03);
        let _ = host_world.step_surface(0.03);
    }

    let g0 = build_car_at_origin(&mut host_world);
    // Add a secondary passenger CabinSeat at (0.75, g0 + 1.5, 1.25) behind the driver seat
    assert!(host_world.place_molded_surface_block(
        Vec3::new(0.75, g0 + 2.35, 1.25),
        Vec3::NEG_Y,
        Vec3::new(-1.0, g0, 1.25),
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));

    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let mut host_driver = PlanetaryAvatar::at(host_world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0));
    host_driver.rotate_degrees(-90.0, 12.0);
    assert!(HandGesture::RightTap.perform(&mut host_driver, &mut host_world));
    assert!(host_driver.use_right_hand(&mut host_world));
    host_driver.sync_world_presence(&mut host_world);
    for _ in 0..8 {
        let _ = host_world.step_surface(0.03);
        let _ = guest_world.step_surface(0.03);
    }

    // Guest boards the passenger CabinSeat at (0.75, g0, 1.25)
    let mut guest_rider = PlanetaryAvatar::at(guest_world.constrain_to_surface(Vec3::new(0.75, 22.0, 2.25), 1.0));
    guest_rider.rotate_degrees(0.0, 50.0);
    assert!(HandGesture::RightTap.perform(&mut guest_rider, &mut guest_world), "Guest must board passenger seat");
    let init_sector = guest_rider.sector_stamp();
    assert_eq!(init_sector.0, ClimateZone::Temperate.index() as u8);

    let mut gas = KeyMotion::idle();
    gas.register_key(KeyCode::KeyW, true);
    let dt = 1.0 / 120.0;
    for _ in 0..240 {
        host_driver.step_motion(dt, &gas, &host_world);
        host_driver.sync_world_presence(&mut host_world);
        let _ = host_world.step_surface(dt);
        let _ = guest_world.step_surface(dt);
        guest_rider.step_motion(dt, &KeyMotion::idle(), &guest_world);
    }

    let host_eye = cam_pos(&host_driver);
    let guest_eye = cam_pos(&guest_rider);
    assert!(
        host_eye.x > 16.5 && host_eye.y < 13.0 && guest_eye.x > 16.5 && guest_eye.y < 14.0,
        "Both host driver and guest passenger must ride over the 90-degree seam onto +X face: host={host_eye:?}, guest={guest_eye:?}"
    );
    assert_eq!(
        guest_rider.sector_stamp().0,
        ClimateZone::Desert.index() as u8,
        "Sector stamp must immediately track the +X face crossing for MeadowRing and slice visibility"
    );
}
