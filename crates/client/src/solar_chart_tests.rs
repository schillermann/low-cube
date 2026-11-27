use std::cell::RefCell;
use std::rc::Rc;
use glam::{DVec3, Vec3};
use winit::keyboard::KeyCode;
use acoustics::{AcousticWave as _, FusionHum};
use world::{
    BlockMedium, BlockMold, BodyId, CelestialAtlas, CelestialSystem, MeshTarget, PlanetaryWorld,
    Quad, SolarEphemeris, SolarProportions,
};
use crate::attire::EvaAttire;
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarPresence as _, HandGesture, PlanetaryAvatar,
};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, ControlAction, InputTrigger, KeyMotion, KeyboardControls};
use crate::drive::{
    ARC_NODES, BrachistochroneArc, CourseProfile, FlipBurnReadout, NavigationCourse,
    SymplecticOrbit, TransferArc,
};
use crate::flight::SurfaceFlight;
use crate::menu::{
    ChartSelection, ChartViewport, EscapeMenu, InspectionModal, PortalInvite, SessionModal,
    SolarChartModal, SystemClipboard,
};
use crate::posture::AvatarPosture;
use crate::session::{HeadlessSession, HeadlessSurface};
use crate::zero_g_flight::ZeroGFlight;

fn build_fusion_torchship(world: &mut dyn PlanetaryWorld) -> Vec3 {
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for mold in [
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().in_medium(BlockMedium::FusionTorch),
    ] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av, mold));
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
        Vec3::new(1.25, g0 + 1.75, 1.75),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().in_medium(BlockMedium::RocketCamera),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.75, 0.75),
        Vec3::X,
        av,
        BlockMold::cube().in_medium(BlockMedium::PlanedPlanks),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.75, g0 + 0.90, 0.75),
        Vec3::Y,
        av,
        BlockMold::cube().in_medium(BlockMedium::FusionTorch),
    ));
    world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0)
}

#[test]
fn test_system_map_key_m_opens_clicks_mars_and_jupiter_and_plots_curved_slingshot_courses() {
    assert_eq!(ActionBindings::seeded().trigger_for(ControlAction::SystemMap), InputTrigger::Key(KeyCode::KeyM));

    let prop = SolarProportions::compressed(32, 0.01);
    let mut world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let spawn = build_fusion_torchship(world.as_mut());
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));

    let ui_quads = Rc::new(RefCell::new(Vec::new()));
    let surface = HeadlessSurface::hd(Rc::clone(&ui_quads));
    let mut modal = SolarChartModal::wrapping(InspectionModal::new(EscapeMenu::with_clipboard(
        PortalInvite::empty(),
        SystemClipboard::new().detached(),
    )));

    assert!(!modal.open());
    assert!(modal.toggle_gameplay_trigger(InputTrigger::Key(KeyCode::KeyM), &surface, &mut pilot, world.as_ref()));
    assert!(modal.open() && !surface.cursor_captured(), "Pressing KeyM must open SolarChartModal and release cursor");

    assert!(modal.handle_trigger(InputTrigger::WheelDown, &mut pilot));
    let mars_pt = modal.body_screen_point(BodyId::MARS, 16.0 / 9.0).expect("Mars projected on chart");
    modal.move_cursor(mars_pt);
    let _ = modal.press_mouse(&surface, &mut pilot);

    let (snap_mars, _, _) = pilot.solar_chart_snapshot(world.as_ref());
    snap_mars.transfer(|atlas, _, _, course| {
        let vp = ChartViewport::calibrated(atlas, Vec3::ZERO, mars_pt);
        assert_eq!(ChartSelection::Locked(BodyId::MARS), ChartSelection::Locked(vp.pick_body(atlas, 9.0 / 16.0).unwrap_or(BodyId::MARS)));
        match course {
            NavigationCourse::Uncharted => panic!("Clicking Mars on SolarChartModal must plot a course in HelmChart"),
            NavigationCourse::Charted(arc) => assert!(arc.flip_marker().is_some(), "FusionTorch ship must default to Brachistochrone 50% flip course"),
        }
    });

    let _ = modal.handle_key(KeyCode::Tab, false, &mut pilot);
    for _ in 0..4 {
        let _ = modal.handle_trigger(InputTrigger::WheelDown, &mut pilot);
    }
    let jup_pt = modal.body_screen_point(BodyId::JUPITER, 16.0 / 9.0).expect("Jupiter projected on chart");
    modal.move_cursor(jup_pt);
    let _ = modal.press_mouse(&surface, &mut pilot);
    let (snap_jup, _, _) = pilot.solar_chart_snapshot(world.as_ref());
    snap_jup.transfer(|_, _, _, course| match course {
        NavigationCourse::Uncharted => panic!("Clicking Jupiter must plot ballistic intercept"),
        NavigationCourse::Charted(arc) => {
            arc.feed_nodes(|nodes, _| {
                let chord_mid = (nodes[0] + nodes[ARC_NODES - 1]) * 0.5;
                let curve_sag = (nodes[ARC_NODES / 2] - chord_mid).length();
                assert!(curve_sag > 0.25, "Multi-body Keplerian Jupiter transfer must curve off a straight chord, sag={curve_sag}");
            });
        }
    });

    let eph = SolarEphemeris::new(SolarProportions::true_scale(64), 0.0);
    let snaps = eph.snapshot_all(0.0);
    let jup = snaps.into_iter().find(|s| s.id() == BodyId::JUPITER).expect("Jupiter");
    let orbit = SymplecticOrbit::new(eph, 8);
    let flyby = orbit.coasting_arc(jup.center() - DVec3::new(jup.half_size() * 1.4, 0.0, 4_000.0), jup.velocity() + DVec3::Z * 260.0, 28.0);
    let mid_sag = (flyby[ARC_NODES / 2] - (flyby[0] + flyby[ARC_NODES - 1]) * 0.5).length();
    assert!(mid_sag > 15.0, "SymplecticOrbit coasting preview must bend into a Jupiter slingshot hyperbola");

    let mut session = HeadlessSession::solitary(16, HeadlessSession::closed_modal());
    session.input_key(KeyCode::KeyM, true);
    assert!(session.modal_open());
    session.advance_ticks(1);
    assert!(session.ui_quad_count() > 40, "Open SolarChartModal must emit hologram quads");
    session.input_key(KeyCode::KeyM, true);
    assert!(!session.modal_open());
}

#[test]
fn test_high_g_tunnel_vision_fusion_hum_and_end_to_end_earth_to_mars_brachistochrone() {
    let hum = FusionHum::torch();
    assert!(hum.amplitude_at(0.12).abs() > 0.02 && hum.amplitude_at(0.90) == 0.0);

    struct VignetteCounter(usize);
    impl MeshTarget for VignetteCounter {
        fn push_quad(&mut self, q: Quad) {
            if q.radiate(|g| g < -0.01) {
                self.0 += 1;
            }
        }
    }
    let calm = AvatarPosture::crouching();
    let mut strained = calm;
    for _ in 0..90 {
        strained = strained.sustain_high_g(7.2, 1.0 / 60.0);
    }
    assert!(strained.cardiac_fov() < calm.cardiac_fov() - 0.08, "Sustained >4G burn must narrow FOV");
    let mut vig = VignetteCounter(0);
    strained.emit_reticle(&mut vig);
    assert!(vig.0 >= 64, "High-G strain must emit peripheral tunnel-vision vignette bands");

    // Commission FusionTorch vessel in Low Earth Orbit aimed along BrachistochroneArc's gravity-solved vector
    let prop = SolarProportions::compressed(32, 0.01);
    let mut world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let mars_center = world.body_center(BodyId::MARS).expect("Mars center");
    let leo_origin = Vec3::new(0.0, 48.0, 0.0);
    let atlas: CelestialAtlas = world.celestial_atlas();
    let arc = BrachistochroneArc::new(78.5)
        .intercept_body(leo_origin.as_dvec3(), DVec3::ZERO, BodyId::MARS, atlas)
        .expect("Mars Brachistochrone course");
    let mut launch_dir = (mars_center - leo_origin).normalize();
    arc.feed_nodes(|nodes, _| {
        launch_dir = (nodes[4] - nodes[0]).normalize_or_zero();
    });
    let ref_up = if launch_dir.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let right = ref_up.cross(launch_dir).normalize();
    let fwd = launch_dir.cross(right).normalize();
    let parts = vec![
        world::VehiclePart::new((0, 0), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::CabinSeat.code()])),
        world::VehiclePart::new((0, 1), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::FlightConsole.code()])),
        world::VehiclePart::new((0, -1), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::FlightConsole.code()])),
        world::VehiclePart::new((-1, 0), 0.0, BlockMold::shape_from_stamp([1, 0, BlockMedium::RocketNozzle.code()])),
        world::VehiclePart::new((1, 0), 0.0, BlockMold::shape_from_stamp([1, 0, BlockMedium::RocketNozzle.code()])),
        world::VehiclePart::new((0, 0), -0.5, BlockMold::shape_from_stamp([0, 0, BlockMedium::FusionTorch.code()])),
    ];
    let assembly = world::VehicleAssembly::new(leo_origin, world::VehicleAxes::new(right, fwd, launch_dir), fwd, parts);
    assert!(world.park_vehicle(assembly));

    let eva_spawn = leo_origin - Vec3::Z * 1.8 - Vec3::Y * 0.8;
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        eva_spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(180.0, 0.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    let idle_prep = KeyMotion::idle();
    for _ in 0..20 {
        pilot.step_motion(1.0 / 60.0, &idle_prep, world.as_ref());
    }
    assert!(pilot.use_right_hand(world.as_mut()));

    let flip_pt = arc.flip_marker().expect("50% flip marker");
    assert!(matches!(
        TransferArc::with_profile([leo_origin; ARC_NODES], Vec3::Y, 12.0, CourseProfile::Brachistochrone { flip_point: flip_pt, total_secs: 10.0, burn_accel: 78.5 })
            .sample_flip_burn(leo_origin, Vec3::ZERO, Vec3::Y),
        FlipBurnReadout::Inactive | FlipBurnReadout::TimeToFlip(_)
    ));
    assert!(matches!(arc.sample_flip_burn(leo_origin, launch_dir * 12.0, launch_dir), FlipBurnReadout::TimeToFlip(_)));
    assert!(matches!(arc.sample_flip_burn(flip_pt, launch_dir * 40.0, launch_dir), FlipBurnReadout::FlipNow));
    assert!(matches!(arc.sample_flip_burn(flip_pt, launch_dir * 40.0, -launch_dir), FlipBurnReadout::BrakeBurn(_)));

    pilot.install_navigation_course(NavigationCourse::Charted(arc));
    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    let idle = KeyMotion::idle();
    let dt = 1.0 / 120.0;
    let pos_of = |a: &PlanetaryAvatar| {
        let mut p = PresenceProbe::at_origin();
        a.report_presence(&mut p);
        p.position()
    };

    let total_span = (mars_center - leo_origin).length();
    let mut burn_ticks = 0usize;
    let mut prev_pos = pos_of(&pilot);
    while (pos_of(&pilot) - leo_origin).length() < total_span * 0.42 && burn_ticks < 3_600 {
        prev_pos = pos_of(&pilot);
        pilot.step_motion(dt, &burn, world.as_ref());
        burn_ticks += 1;
    }
    let mid_speed = (pos_of(&pilot) - prev_pos).length() / dt;
    assert!(mid_speed > 120.0, "FusionTorch burn must break far past 45 m/s chemical cap, got {mid_speed:.1} m/s");

    // 50% Flip-and-Burn: execute 180-degree RetroFlip (`KeyR`) across midpoint, then brake burn into Mars orbit!
    assert!(pilot.dispatch_trigger(InputTrigger::Key(KeyCode::KeyR)));
    for _ in 0..150 {
        pilot.step_motion(dt, &idle, world.as_ref());
    }
    let mut min_mars_dist = (pos_of(&pilot) - mars_center).length();
    let mut min_speed = mid_speed;
    for _ in 0..burn_ticks + 240 {
        let before = pos_of(&pilot);
        pilot.step_motion(dt, &burn, world.as_ref());
        let now = pos_of(&pilot);
        let vel = (now - before) / dt;
        let speed = vel.length();
        min_mars_dist = min_mars_dist.min((now - mars_center).length());
        min_speed = min_speed.min(speed);
        if vel.dot(launch_dir) <= 2.0 {
            break;
        }
    }
    assert!(
        min_mars_dist < total_span * 0.14 && min_speed < mid_speed * 0.15,
        "End-to-end Brachistochrone must arrive in Mars orbit and shed cruise velocity: min_dist={min_mars_dist:.1}m (span={total_span:.1}m), min_speed={min_speed:.1} m/s (from {mid_speed:.1})"
    );
}
