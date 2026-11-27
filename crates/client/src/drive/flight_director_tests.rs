use glam::{Mat4, Quat, Vec3};
use world::{BlockMedium, BlockMold, CelestialAtlas, CelestialSystem, MeshTarget, PlanetaryWorld, Quad};
use crate::menu::{
    BellyFlightHud, BurnMagnetOverlay, CompassRingOverlay, CourseAlignmentOverlay, HudSightStage,
    LateralSlipOverlay, NoseFlightHud, OpticOverlay, ProgradeDriftOverlay, ScreenPoint,
    SilentOverlay, TouchdownGhostOverlay,
};
use super::{
    ARC_NODES, BellySnapshot, BurnVector, FlightKinematics, HullClearance, NavigationCourse,
    OrrerySnapshot, TOUCHDOWN_NODES, TouchdownFootprint, TouchdownTone, TouchdownTrace,
    TransferArc,
};

struct ColorProbe {
    total: usize,
    emerald: usize,
    amber: usize,
    coral: usize,
}

impl ColorProbe {
    const fn empty() -> Self {
        Self { total: 0, emerald: 0, amber: 0, coral: 0 }
    }
}

impl MeshTarget for ColorProbe {
    fn push_quad(&mut self, quad: Quad) {
        self.total += 1;
        quad.transfer(|_, _, _, _, _, col| {
            if (col - TouchdownTone::SafeEmerald.color()).length() < 1e-3 {
                self.emerald += 1;
            }
            if (col - TouchdownTone::CautionAmber.color()).length() < 1e-3 {
                self.amber += 1;
            }
            if (col - TouchdownTone::CrashCoral.color()).length() < 1e-3 {
                self.coral += 1;
            }
        });
    }
}

fn build_test_ship(world: &mut dyn PlanetaryWorld) -> world::VehicleAssembly {
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0).y;
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
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 2.90, 0.25),
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
    world
        .board_vehicle(
            Vec3::new(1.75, g0 + 1.80, 0.75),
            Vec3::new(-0.985, -0.174, 0.0).normalize(),
            false,
        )
        .expect("Rocket assembly must resolve cleanly")
}

#[test]
fn test_burn_guidance_counters_lateral_newtonian_drift_toward_course() {
    let nodes: [Vec3; ARC_NODES] = std::array::from_fn(|i| Vec3::new(0.0, 0.0, -(i as f32) * 20.0));
    let arc = TransferArc::new(nodes, Vec3::Z, 8.0);

    let zero_drift = FlightKinematics::new(Vec3::ZERO, Vec3::NEG_Z, Vec3::new(0.0, 0.0, -40.0));
    let vec_clean = zero_drift.burn_guidance(&arc);
    assert!(matches!(vec_clean, BurnVector::ForwardLead(_)));
    vec_clean.present(Vec3::NEG_Z, |dir_clean, _, cue| {
        assert_eq!(cue, "LOCKED");
        assert!(dir_clean.x.abs() < 1e-4 && dir_clean.z < -0.99);
    });

    let drifting_right = FlightKinematics::new(Vec3::ZERO, Vec3::NEG_Z, Vec3::new(18.0, 0.0, -40.0));
    drifting_right.burn_guidance(&arc).present(Vec3::NEG_Z, |dir_lead, _, cue| {
        assert_eq!(cue, "BURN");
        assert!(
            dir_lead.x < -0.50 && dir_lead.z < -0.60,
            "Rightward (+X) drift must shift BURN magnet leftward (-X) to cancel drift, got {dir_lead:?}"
        );
    });
}

#[test]
fn test_nose_flight_director_decorators_compose_compass_prograde_magnet_and_course() {
    let vp = Mat4::perspective_rh(65.0_f32.to_radians(), 16.0 / 9.0, 0.03, 65_536.0)
        * Mat4::look_to_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);
    let stage = HudSightStage::new(vp, ScreenPoint::center(), 9.0 / 16.0);
    let atlas = CelestialAtlas::new(16.0, Vec3::new(140.0, 60.0, 120.0), 16.0);
    let nodes: [Vec3; ARC_NODES] = std::array::from_fn(|i| Vec3::new(0.0, 0.0, -(i as f32) * 15.0));
    let arc = TransferArc::new(nodes, Vec3::Z, 6.0);
    let ship_pos = Vec3::new(0.0, 40.0, 0.0);
    let vel = Vec3::new(12.0, -4.0, -36.0);
    let kinematics = FlightKinematics::new(ship_pos, Vec3::NEG_Z, vel);

    let compass = CompassRingOverlay::new(SilentOverlay, stage, atlas, ship_pos);
    let mut compass_probe = ColorProbe::empty();
    compass.emit(&mut compass_probe);
    assert!(compass_probe.total >= 72);

    let prograde = ProgradeDriftOverlay::new(compass, stage, kinematics);
    let mut prograde_probe = ColorProbe::empty();
    prograde.emit(&mut prograde_probe);
    assert!(prograde_probe.total > compass_probe.total + 20);

    let magnet = BurnMagnetOverlay::new(prograde, stage, kinematics, arc);
    let mut magnet_probe = ColorProbe::empty();
    magnet.emit(&mut magnet_probe);
    assert!(magnet_probe.total > prograde_probe.total + 18);

    let aligned = CourseAlignmentOverlay::new(magnet, stage, kinematics, arc);
    let mut aligned_probe = ColorProbe::empty();
    aligned.emit(&mut aligned_probe);
    assert!(aligned_probe.total > magnet_probe.total + 12);

    let snap = OrrerySnapshot::new(atlas, ship_pos, (Vec3::NEG_Z, vel), NavigationCourse::Charted(arc));
    let mut nose_probe = ColorProbe::empty();
    NoseFlightHud::new(snap, vp, ScreenPoint::center(), 9.0 / 16.0).emit(&mut nose_probe);
    assert!(nose_probe.total > aligned_probe.total + 40);
}

#[test]
fn test_belly_touchdown_ghost_and_lateral_slip_classify_safe_caution_and_crash_descents() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let assembly = build_test_ship(world.as_mut());
    let atlas = world.celestial_atlas();
    let moon_c = CelestialSystem::moon_center(32);
    let hover_pos = moon_c + Vec3::new(0.25, 16.0 + 28.0, 0.25);
    let vp = Mat4::perspective_rh(65.0_f32.to_radians(), 16.0 / 9.0, 0.03, 65_536.0)
        * Mat4::look_to_rh(hover_pos, Vec3::NEG_Y, Vec3::Z);
    let stage = HudSightStage::new(vp, ScreenPoint::center(), 9.0 / 16.0);

    let pad_safe = HullClearance::new(&assembly, hover_pos, Vec3::new(0.0, -3.0, 0.0), Quat::IDENTITY)
        .sample_pad(atlas, world.as_ref());
    let trail: [Vec3; TOUCHDOWN_NODES] = std::array::from_fn(|i| {
        let t = (i as f32) / ((TOUCHDOWN_NODES - 1) as f32);
        hover_pos.lerp(moon_c + Vec3::new(0.25, 16.0, 0.25), t)
    });
    let safe_trace = TouchdownTrace::Predicted(TouchdownFootprint::new(
        trail,
        Vec3::Y,
        Vec3::new(0.1, -3.2, 0.0),
        1.1,
    ));
    let caution_trace = TouchdownTrace::Predicted(TouchdownFootprint::new(
        trail,
        Vec3::Y,
        Vec3::new(1.5, -8.5, 0.0),
        1.1,
    ));
    let crash_trace = TouchdownTrace::Predicted(TouchdownFootprint::new(
        trail,
        Vec3::Y,
        Vec3::new(6.4, -18.0, 0.0),
        1.1,
    ));

    let mut safe_probe = ColorProbe::empty();
    LateralSlipOverlay::new(TouchdownGhostOverlay::new(SilentOverlay, stage, safe_trace), stage, safe_trace)
        .emit(&mut safe_probe);
    assert!(safe_probe.emerald >= 28 && safe_probe.coral == 0, "Gentle vertical sink must glow SafeEmerald");

    let mut caution_probe = ColorProbe::empty();
    TouchdownGhostOverlay::new(SilentOverlay, stage, caution_trace).emit(&mut caution_probe);
    assert!(caution_probe.amber >= 28, "Medium sink must glow CautionAmber");

    let mut crash_probe = ColorProbe::empty();
    LateralSlipOverlay::new(TouchdownGhostOverlay::new(SilentOverlay, stage, crash_trace), stage, crash_trace)
        .emit(&mut crash_probe);
    assert!(
        crash_probe.coral >= 28 && crash_probe.total > safe_probe.total + 20,
        "High sink + horizontal slip must glow CrashCoral and add SLIP reticle"
    );

    let belly_snap = BellySnapshot::new(pad_safe, safe_trace, NavigationCourse::Uncharted, hover_pos);
    let mut belly_probe = ColorProbe::empty();
    BellyFlightHud::new(belly_snap, vp, ScreenPoint::center(), 9.0 / 16.0).emit(&mut belly_probe);
    assert!(belly_probe.total > safe_probe.total + 40);

    let shifted = crash_trace.shifted_by(Vec3::new(100.0, -50.0, 25.0));
    shifted.visit_predicted(|fp| {
        fp.transfer(|pts, _, _, _| {
            assert!((pts[0] - (trail[0] + Vec3::new(100.0, -50.0, 25.0))).length() < 1e-4);
        });
    });
}
