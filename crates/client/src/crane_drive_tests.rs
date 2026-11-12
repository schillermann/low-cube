use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, BlockMedium, BlockMold, MeshTarget, PlanetaryWorld, Quad};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar, HandGesture, PlanetaryAvatar};
use crate::controls::{InputTrigger, KeyMotion, KeyboardControls};

struct VertexLedger {
    points: Vec<Vec3>,
}

impl VertexLedger {
    const fn empty() -> Self {
        Self { points: Vec::new() }
    }
}

impl MeshTarget for VertexLedger {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, _, _, _, _, _| self.points.push(v0));
    }
}

#[test]
fn test_crane_lowers_built_car_and_clicking_seat_drives_steers_and_brakes() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let rig_mold = BlockMold::cube().in_medium(BlockMedium::CantileverRig);
    let eng_mold = BlockMold::cube().turn_clockwise().in_medium(BlockMedium::SteeringWheel);
    let seat_mold = BlockMold::cube().in_medium(BlockMedium::CabinSeat);
    let wheel_mold = BlockMold::cube().in_medium(BlockMedium::CarWheel);
    let av_pos = Vec3::new(0.25, g0, 1.75);
    let aim = Vec3::new(0.0, -0.8, -0.6).normalize();
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), aim, av_pos, rig_mold));
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), aim, av_pos, eng_mold));
    let hoist_eye = Vec3::new(1.25, g0 + 1.75, 1.25);
    assert!(world.place_molded_surface_block(hoist_eye, Vec3::NEG_Z, av_pos, seat_mold));
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av_pos, wheel_mold));
    assert!((world.constrain_to_surface(Vec3::new(1.25, 22.0, 0.75), 0.0).y - (g0 + 2.0)).abs() < 1e-4);

    let mut crane_hand = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 22.0, 1.25), 1.0)).with_test_engines();
    crane_hand.apply_attire(YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm));
    crane_hand.rotate_degrees(0.0, 35.0);
    assert!(HandGesture::RightTap.perform(&mut crane_hand, &mut world));
    let g_seat = world.constrain_to_surface(Vec3::new(1.25, 22.0, 0.75), 0.0).y;
    assert!(g_seat < g0 + 1.0, "Lowered car seat must rest on ground (got {g_seat})");

    let mut pilot = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.25, 22.0, 1.75), 1.0));
    pilot.rotate_degrees(0.0, 58.0);
    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let stand_eye = cam_pos(&pilot);
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));
    let seated_eye = cam_pos(&pilot);
    assert!((seated_eye.x - 1.25).abs() < 0.25 && (seated_eye.z - 0.75).abs() < 0.25 && seated_eye.y < stand_eye.y);
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));

    let mut gas = KeyMotion::idle();
    gas.register_key(KeyCode::KeyW, true);
    for _ in 0..15 {
        pilot.step_motion(1.0 / 60.0, &gas, &world);
    }
    assert!(cam_pos(&pilot).x > seated_eye.x + 0.4);
    gas.register_key(KeyCode::KeyD, true);
    for _ in 0..20 {
        pilot.step_motion(1.0 / 60.0, &gas, &world);
    }
    let driven_eye = cam_pos(&pilot);
    assert!(driven_eye.distance(seated_eye) > 1.2 && (driven_eye.z - seated_eye.z).abs() > 0.15);

    let mut brake = KeyMotion::idle();
    brake.register_key(KeyCode::KeyS, true);
    for _ in 0..30 {
        pilot.step_motion(1.0 / 60.0, &brake, &world);
    }
    let stopped_eye = cam_pos(&pilot);
    pilot.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    assert!(cam_pos(&pilot).distance(stopped_eye) < 1e-4);
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));

    pilot.apply_attire(YouthAttire::squire().with_relics(HeldRelic::CabinSeat, HeldRelic::EmptyPalm));
    let mut turns = Vec::with_capacity(5);
    for step in 0..5 {
        let mut ledger = VertexLedger::empty();
        pilot.emit_appearance(&mut ledger);
        turns.push(ledger.points);
        if step < 4 {
            assert!(pilot.dispatch_trigger(InputTrigger::WheelClick));
        }
    }
    assert!(turns[0] != turns[1] && turns[1] != turns[2] && turns[2] != turns[3] && turns[0] != turns[2] && turns[1] != turns[3] && turns[0] == turns[4]);
}
