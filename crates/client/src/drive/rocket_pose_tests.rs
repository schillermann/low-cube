use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{BlockMedium, BlockMold, CelestialSystem, MeshTarget, Quad};
use crate::attire::{BodyPose, CrewPhysique, EvaAttire, HeldRelic, MartialStyle, RocketControls, StrideRhythm};
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarPresence as _, HandGesture, PlanetaryAvatar};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, InputTrigger, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::menu::{StudioAnimation, TurntableStage};
use crate::zero_g_flight::ZeroGFlight;

struct MeshRecorder(Vec<Vec3>);
impl MeshTarget for MeshRecorder {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, _| self.0.extend_from_slice(&[v0, v1, v2, v3]));
    }
}

#[test]
fn test_rocket_pilot_arms_drive_four_way_stick_and_fore_aft_throttle_grip() {
    let base = BodyPose::new(
        Vec3::ZERO,
        Quat::IDENTITY,
        Quat::IDENTITY,
        0.46,
        StrideRhythm::idle(),
        CrewPhysique::OrbitalCadet,
    );
    let sample_arm = |ctrl: RocketControls, side: f32| {
        let pose = base.with_rocket_controls(ctrl);
        let (_, _, hand, _) = pose.arm(side, MartialStyle::PalmCradle).external(true);
        let curl = pose.arm(side, MartialStyle::PalmCradle).finger_curl(false);
        (hand, curl)
    };

    let (r_neutral, r_curl) = sample_arm(RocketControls::neutral(), 1.0);
    let (r_fwd, _) = sample_arm(RocketControls::clamped(0.0, 1.0, 0.0), 1.0);
    let (r_back, _) = sample_arm(RocketControls::clamped(0.0, -1.0, 0.0), 1.0);
    let (r_left, _) = sample_arm(RocketControls::clamped(-1.0, 0.0, 0.0), 1.0);
    let (r_right, _) = sample_arm(RocketControls::clamped(1.0, 0.0, 0.0), 1.0);
    let w_rn = r_neutral.point(0.0, 0.0, 0.0);
    assert!(r_curl >= 0.85, "Right hand must firmly grip vertical flight stick");
    assert!(
        (r_neutral.point(0.0, 1.0, 0.0) - w_rn).y > 0.75,
        "Right hand thumb axis (+Y) must point upward on vertical Steuerknueppel"
    );
    assert!(-r_fwd.point(0.0, 0.0, 0.0).z > -w_rn.z + 0.05, "Stick forward must push right wrist forward");
    assert!(-r_back.point(0.0, 0.0, 0.0).z < -w_rn.z - 0.05, "Stick backward must pull right wrist back");
    assert!(r_right.point(0.0, 0.0, 0.0).x > w_rn.x + 0.06, "Stick right must shift right wrist right");
    assert!(r_left.point(0.0, 0.0, 0.0).x < w_rn.x - 0.06, "Stick left must shift right wrist left");

    let (l_neutral, l_curl) = sample_arm(RocketControls::neutral(), -1.0);
    let (l_burn, _) = sample_arm(RocketControls::clamped(0.0, 0.0, 1.0), -1.0);
    let (l_retro, _) = sample_arm(RocketControls::clamped(0.0, 0.0, -1.0), -1.0);
    let (l_during_stick, _) = sample_arm(RocketControls::clamped(1.0, -1.0, 0.0), -1.0);
    let w_ln = l_neutral.point(0.0, 0.0, 0.0);
    assert!(l_curl >= 0.84, "Left hand must grip horizontal throttle bar");
    assert!(
        (l_neutral.point(1.0, 0.0, 0.0) - w_ln).y < -0.75,
        "Left palm must face downward in overhand grip on horizontal Schubknueppel"
    );
    assert!(-l_burn.point(0.0, 0.0, 0.0).z > -w_ln.z + 0.07, "Thrust forward must push left wrist forward");
    assert!(-l_retro.point(0.0, 0.0, 0.0).z < -w_ln.z - 0.07, "Thrust backward must pull left wrist back");
    assert!((l_burn.point(0.0, 0.0, 0.0).x - l_retro.point(0.0, 0.0, 0.0).x).abs() < 1e-4);
    assert_eq!(l_during_stick.point(0.0, 0.0, 0.0), w_ln, "Stick motion must not move throttle hand");
}

#[test]
fn test_live_rocket_cockpit_and_character_studio_animate_stick_and_throttle() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0).y;
    let av_pos = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for mold in [
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().in_medium(BlockMedium::RocketNozzle),
    ] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av_pos, mold));
    }
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 2.40, 0.25),
        Vec3::NEG_Y,
        av_pos,
        BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 0.75),
        Vec3::NEG_Z,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));
    let spawn = world.constrain_to_surface(Vec3::new(1.75, 16.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));

    let mut idle_mesh = MeshRecorder(Vec::new());
    pilot.emit_appearance(&mut idle_mesh);
    let mut left_stick = KeyMotion::idle();
    left_stick.register_key(KeyCode::KeyA, true);
    for _ in 0..8 {
        pilot.step_motion(1.0 / 120.0, &left_stick, world.as_ref());
    }
    let mut stick_mesh = MeshRecorder(Vec::new());
    pilot.emit_appearance(&mut stick_mesh);
    assert_ne!(idle_mesh.0, stick_mesh.0, "Deflecting 4-way stick without thrust must animate pilot arm");

    let attire = EvaAttire::astronaut();
    let vp = TurntableStage::front_showcase().view_projection(16.0 / 9.0);
    let mut phases = Vec::new();
    for step in 0..4 {
        let t = (step as f32) * 0.55 + 0.275;
        let mut fig = MeshRecorder(Vec::new());
        StudioAnimation::RocketPilot.emit_figure(Vec3::new(0.0, 1.0, 0.0), 0.0, t, &attire, &mut fig);
        for v in &fig.0 {
            let clip = vp * v.extend(1.0);
            let ndc_y = clip.y / clip.w;
            assert!((-0.70..=0.68).contains(&ndc_y), "RocketPilot must fit studio viewport, got {ndc_y}");
        }
        phases.push(fig.0);
    }
    assert!(phases[0] != phases[1] && phases[1] != phases[2] && phases[2] != phases[3]);
}

#[test]
fn test_hoisted_crane_dual_steuerduesen_preserve_thruster_and_never_perch_off_seat() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for mold in [
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().in_medium(BlockMedium::FlightConsole),
    ] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g + 1.5, 1.25), crane_dir, av, mold));
    }
    assert!(world.adjust_rig(Vec3::new(1.25, g + 1.75, 1.25), Vec3::NEG_Z, 5, false));
    for med in [BlockMedium::TurfEarth, BlockMedium::TurfEarth, BlockMedium::RocketNozzle] {
        assert!(world.place_molded_surface_block(
            Vec3::new(1.25, g + 1.0, 0.25),
            Vec3::Y,
            Vec3::new(1.25, g + 1.0, 0.25),
            BlockMold::cube().in_medium(med),
        ));
    }
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g + 4.25, 1.25),
        Vec3::NEG_Z,
        Vec3::new(1.25, g + 1.0, 2.25),
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));

    let attire = EvaAttire::astronaut().with_relics(HeldRelic::RocketNozzle, HeldRelic::EmptyPalm);
    // 1. Place first Steuerduese on North side of dirt block (2, -1, g + 3.0)
    let mut north_builder = PlanetaryAvatar::with_attire_and_bindings(
        Vec3::new(1.25, g + 1.0, -0.75),
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        attire,
        ActionBindings::seeded(),
    );
    assert!(north_builder.dispatch_trigger(InputTrigger::WheelDown));
    north_builder.rotate_degrees(180.0, -68.0);
    assert!(HandGesture::RightTap.perform(&mut north_builder, world.as_mut()));

    // 2. Walk directly underneath the hoisted CabinSeat (1.25, g + 1.0, 0.75) with 2nd Steuerduese:
    //    Clipping the underside of the main bell must NOT consume the Steuerduese or perch on the seat!
    let mut under_seat = PlanetaryAvatar::with_attire_and_bindings(
        Vec3::new(1.25, g + 1.0, 0.75),
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        attire,
        ActionBindings::seeded(),
    );
    assert!(under_seat.dispatch_trigger(InputTrigger::WheelDown));
    under_seat.rotate_degrees(0.0, -68.0);
    let _ = HandGesture::RightTap.perform(&mut under_seat, world.as_mut());
    let mut probe = PresenceProbe::at_origin();
    under_seat.report_presence(&mut probe);
    let held_after = probe.presence().transfer(|_, _, _, pa| {
        pa.transfer(|b, _, _| EvaAttire::from_multiplayer_bytes(b).held_pair().0)
    });
    assert_eq!(held_after, HeldRelic::RocketNozzle, "Underside bell graze must never swallow Steuerduese");
    assert!((probe.position().y - (g + 1.0)).abs() < 0.05, "Builder must remain standing on ground");

    // Clicking dirt/thruster with empty left hand must also never teleport onto CabinSeat
    let _ = HandGesture::LeftTap.perform(&mut under_seat, world.as_mut());
    under_seat.report_presence(&mut probe);
    assert!((probe.position().y - (g + 1.0)).abs() < 0.05, "Clicking non-seat crane block must not perch");

    // 3. Step back to aim cleanly at opposite side wall of dirt block -> attaches 2nd Steuerduese!
    let mut south_builder = PlanetaryAvatar::with_attire_and_bindings(
        Vec3::new(1.25, g + 1.0, 1.25),
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        attire,
        ActionBindings::seeded(),
    );
    assert!(south_builder.dispatch_trigger(InputTrigger::WheelDown));
    south_builder.rotate_degrees(0.0, -62.0);
    assert!(HandGesture::RightTap.perform(&mut south_builder, world.as_mut()));
    south_builder.report_presence(&mut probe);
    assert!((probe.position().y - (g + 1.0)).abs() < 0.05);
}

