use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BlockMedium, BlockMold, CelestialSystem, CraftBlueprint, CraftPose, MeshTarget, PlanetaryWorld,
    Quad, VesselSink,
};
use crate::attire::EvaAttire;
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarPresence as _, HandGesture, PlanetaryAvatar,
};
use crate::controls::{ActionBindings, ControlAction, InputTrigger, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;

fn build_deep_space_rocket(world: &mut dyn PlanetaryWorld) -> PlanetaryAvatar {
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for med in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
        assert!(world.place_molded_surface_block(
            Vec3::new(0.25, g0 + 1.5, 1.25),
            crane_dir,
            av,
            BlockMold::cube().in_medium(med),
        ));
    }
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 2.40, 0.25),
        Vec3::NEG_Y,
        av,
        BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 0.75),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.25),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, -0.25),
        Vec3::Z,
        Vec3::new(1.25, g0, -1.25),
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 2.90, 0.25),
        Vec3::NEG_Y,
        av,
        BlockMold::cube().in_medium(BlockMedium::RocketCamera),
    ));

    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world));
    assert!(pilot.use_right_hand(world));
    pilot
}

fn ship_pose(pilot: &PlanetaryAvatar) -> CraftPose {
    struct PoseGrab(Option<CraftPose>);
    impl VesselSink for PoseGrab {
        fn receive_vessel(&mut self, _bp: &CraftBlueprint, pose: CraftPose) {
            self.0 = Some(pose);
        }
    }
    let mut grab = PoseGrab(None);
    pilot.present_vessel(&mut grab);
    grab.0.expect("Piloted rocket must present live CraftPose")
}

#[test]
fn test_flight_computer_toggle_shows_on_nose_camera_and_switches_vacuum_spin_damping() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut pilot = build_deep_space_rocket(world.as_mut());

    struct TopBannerProbe {
        emerald_lamp: bool,
        amber_lamp: bool,
        top_quads: usize,
    }
    impl MeshTarget for TopBannerProbe {
        fn push_quad(&mut self, quad: Quad) {
            quad.transfer(|v0, _, v2, _, _, col| {
                let c = (v0 + v2) * 0.5;
                if c.y > 0.84 && c.x.abs() < 0.35 {
                    self.top_quads += 1;
                    if col.y > 0.90 && col.x < 0.30 {
                        self.emerald_lamp = true;
                    }
                    if col.x > 0.90 && col.y < 0.35 {
                        self.amber_lamp = true;
                    }
                }
            });
        }
    }

    let mut hud_on = TopBannerProbe { emerald_lamp: false, amber_lamp: false, top_quads: 0 };
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut hud_on);
    assert!(
        hud_on.emerald_lamp && !hud_on.amber_lamp && hud_on.top_quads > 30,
        "Nose camera HUD must display FLUGCOMPUTER: EIN with emerald indicator"
    );

    let mut ascent = KeyMotion::idle();
    ascent.register_key(KeyCode::Space, true);
    for _ in 0..420 {
        pilot.step_motion(1.0 / 120.0, &ascent, world.as_ref());
    }

    let toggle_fc = ActionBindings::seeded().trigger_for(ControlAction::FlightGuidance);
    assert!(pilot.dispatch_trigger(toggle_fc));

    let mut hud_off = TopBannerProbe { emerald_lamp: false, amber_lamp: false, top_quads: 0 };
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut hud_off);
    assert!(
        hud_off.amber_lamp && !hud_off.emerald_lamp && hud_off.top_quads != hud_on.top_quads,
        "Nose camera HUD must switch to FLUGCOMPUTER: AUS with warning indicator"
    );

    let mut yaw_burst = KeyMotion::idle();
    yaw_burst.register_key(KeyCode::KeyA, true);
    for _ in 0..24 {
        pilot.step_motion(1.0 / 120.0, &yaw_burst, world.as_ref());
    }
    let h_after_burst = ship_pose(&pilot).heading();
    let idle = KeyMotion::idle();
    for _ in 0..60 {
        pilot.step_motion(1.0 / 120.0, &idle, world.as_ref());
    }
    let h_coasted = ship_pose(&pilot).heading();
    assert!(
        h_after_burst.dot(h_coasted) < 0.97,
        "With Flight Computer OFF, spacecraft must keep spinning in vacuum after releasing keys"
    );

    assert!(pilot.dispatch_trigger(toggle_fc));
    for _ in 0..75 {
        pilot.step_motion(1.0 / 120.0, &idle, world.as_ref());
    }
    let h_damped_start = ship_pose(&pilot).heading();
    for _ in 0..30 {
        pilot.step_motion(1.0 / 120.0, &idle, world.as_ref());
    }
    let h_damped_end = ship_pose(&pilot).heading();
    assert!(
        h_damped_start.dot(h_damped_end) > 0.9999,
        "Re-engaging Flight Computer must arrest vacuum rotation to rest"
    );
}

#[test]
fn test_qe_roll_strafe_and_h_controls_slate_toggle() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut pilot = build_deep_space_rocket(world.as_mut());

    struct CountQuads(usize);
    impl MeshTarget for CountQuads {
        fn push_quad(&mut self, _quad: Quad) { self.0 += 1; }
    }
    let mut base_hud = CountQuads(0);
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut base_hud);
    assert!(pilot.dispatch_trigger(InputTrigger::Key(KeyCode::KeyH)));
    let mut shown_hud = CountQuads(0);
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut shown_hud);
    assert!(
        shown_hud.0 > base_hud.0 + 120,
        "Pressing H must show the on-screen controls slate ({} -> {})",
        base_hud.0,
        shown_hud.0
    );
    assert!(pilot.dispatch_trigger(InputTrigger::Key(KeyCode::KeyH)));
    let mut hidden_hud = CountQuads(0);
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut hidden_hud);
    assert_eq!(hidden_hud.0, base_hud.0, "Pressing H again must hide the controls slate");

    assert!(
        !pilot.dispatch_trigger(InputTrigger::Key(KeyCode::KeyQ))
            && !pilot.dispatch_trigger(InputTrigger::Key(KeyCode::KeyE)),
        "Q and E while piloting must pass straight to continuous Roll without emptying hands"
    );

    let mut ascent = KeyMotion::idle();
    ascent.register_key(KeyCode::Space, true);
    for _ in 0..420 {
        pilot.step_motion(1.0 / 120.0, &ascent, world.as_ref());
    }

    let before_roll = ship_pose(&pilot);
    let mut roll_right = KeyMotion::idle();
    roll_right.register_trigger(InputTrigger::Key(KeyCode::KeyE), true);
    for _ in 0..45 {
        pilot.step_motion(1.0 / 120.0, &roll_right, world.as_ref());
    }
    let after_roll = ship_pose(&pilot);
    assert!(
        before_roll.normal().dot(after_roll.normal()) < 0.97
            && before_roll.heading().dot(after_roll.heading()) > 0.98,
        "RollRight binding (E) must roll normal around longitudinal heading"
    );

    let mut roll_left = KeyMotion::idle();
    roll_left.register_trigger(InputTrigger::Key(KeyCode::KeyQ), true);
    for _ in 0..45 {
        pilot.step_motion(1.0 / 120.0, &roll_left, world.as_ref());
    }
    let after_roll_back = ship_pose(&pilot);
    assert!(
        after_roll_back.normal().dot(before_roll.normal()) > 0.97,
        "RollLeft binding (Q) must roll back in the opposite direction"
    );

    let idle = KeyMotion::idle();
    for _ in 0..60 {
        pilot.step_motion(1.0 / 120.0, &idle, world.as_ref());
    }
    let before_strafe = ship_pose(&pilot);
    let mut strafe = KeyMotion::idle();
    strafe.register_key(KeyCode::AltLeft, true);
    strafe.register_key(KeyCode::KeyD, true);
    for _ in 0..60 {
        pilot.step_motion(1.0 / 120.0, &strafe, world.as_ref());
    }
    let after_strafe = ship_pose(&pilot);
    assert!(
        (after_strafe.origin() - before_strafe.origin()).length() > 1.2
            && before_strafe.heading().dot(after_strafe.heading()) > 0.995,
        "Holding RcsTranslate (LALT) + D must translate laterally without yawing heading"
    );
}
