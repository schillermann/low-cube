use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BlockMedium, BlockMold, CelestialAtlas, CelestialSystem, GravityField, MeshTarget,
    PlanetaryWorld, Quad, TerrainMesh, VesselDeck, WoodlandLife, WorldEarthworks, CompanionPortal,
};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar, AvatarMotor, AvatarOptics, HandGesture, PlanetaryAvatar};
use crate::controls::{ActionBindings, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;
use super::{ARC_NODES, TransferArc};

struct ZeroGravityWorld;

impl GravityField for ZeroGravityWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 { Vec3::NEG_Y }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 { Vec3::Y }
    fn apply_pull(&self, _position: Vec3, _dt: f32) -> Vec3 { Vec3::ZERO }
}
impl WorldEarthworks for ZeroGravityWorld {}
impl TerrainMesh for ZeroGravityWorld {
    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
}
impl WoodlandLife for ZeroGravityWorld {}
impl VesselDeck for ZeroGravityWorld {}
impl CompanionPortal for ZeroGravityWorld {}
impl PlanetaryWorld for ZeroGravityWorld {
    fn constrain_to_surface(&self, position: Vec3, _player_height: f32) -> Vec3 { position }
    fn celestial_atlas(&self) -> CelestialAtlas {
        CelestialAtlas::new(16.0, CelestialSystem::moon_center(32), 16.0)
    }
}

fn assemble_nose_rocket(world: &mut dyn PlanetaryWorld) -> PlanetaryAvatar {
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
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.25),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().in_medium(BlockMedium::RocketCamera),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 0.40, 1.25),
        Vec3::NEG_Y,
        Vec3::new(1.25, g0, 2.25),
        BlockMold::cube().in_medium(BlockMedium::QuarriedStone),
    ));
    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world));
    assert!(pilot.use_right_hand(world));
    pilot
}

#[test]
fn test_cosmic_sight_ray_locks_lunar_surface_from_launchpad_distance() {
    let world = CelestialSystem::barren_authoritative(32, 1.0);
    let moon_center = CelestialSystem::moon_center(32);
    let eye = Vec3::new(1.25, 20.0, 0.75);
    let dir = (moon_center - eye).normalize();
    let hit = world
        .celestial_atlas()
        .lock_sight_surface(eye, dir, world.as_ref())
        .expect("Cosmic sight ray aimed at Moon must lock onto LunarWorld surface");
    let (surface_pt, surface_norm) = hit;
    let dist_from_moon_center = (surface_pt - moon_center).abs().max_element();
    assert!(
        (14.0..=18.5).contains(&dist_from_moon_center) && surface_norm.length() > 0.95,
        "Locked target must lie on the 16-half-size Moon-Cube crust, got pt={surface_pt:?}, norm={surface_norm:?}"
    );
}

#[test]
fn test_transfer_arc_incorporates_binary_gravity_curvature() {
    let world = CelestialSystem::barren_authoritative(32, 1.0);
    let vacuum = ZeroGravityWorld;
    let origin = Vec3::new(1.25, 19.0, 0.75);
    let target = CelestialSystem::moon_center(32) + Vec3::new(0.0, -16.0, 0.0);
    let grav_arc = TransferArc::charted(origin, Vec3::ZERO, target, Vec3::NEG_Y, world.as_ref());
    let vac_arc = TransferArc::charted(origin, Vec3::ZERO, target, Vec3::NEG_Y, &vacuum);

    let mut max_divergence = 0.0_f32;
    grav_arc.feed_nodes(|g_nodes, _| {
        vac_arc.feed_nodes(|v_nodes, _| {
            assert!((g_nodes[0] - origin).length() < 1e-4);
            assert!((g_nodes[ARC_NODES - 1] - target).length() < 1e-4);
            for i in 1..(ARC_NODES - 1) {
                max_divergence = max_divergence.max((g_nodes[i] - v_nodes[i]).length());
            }
        });
    });
    assert!(
        max_divergence > 1.5,
        "Real BinaryGravity wells must curve the transfer trajectory relative to zero-g vacuum (delta={max_divergence:.2}m)"
    );
}

#[test]
fn test_route_telemetry_tracks_on_course_and_off_course_deviation() {
    let world = CelestialSystem::barren_authoritative(32, 1.0);
    let origin = Vec3::new(1.25, 19.0, 0.75);
    let target = CelestialSystem::moon_center(32) + Vec3::new(0.0, -16.0, 0.0);
    let arc = TransferArc::charted(origin, Vec3::ZERO, target, Vec3::NEG_Y, world.as_ref());

    let mut mid_pt = Vec3::ZERO;
    let mut mid_tan = Vec3::Y;
    arc.feed_nodes(|nodes, _| {
        mid_pt = nodes[12];
        mid_tan = (nodes[13] - nodes[12]).normalize();
    });

    let on_course = arc.sample_telemetry(mid_pt + Vec3::new(1.2, 0.0, 0.0), mid_tan);
    on_course.transfer(|_, cross_m, angle_deg, aligned| {
        assert!(aligned && cross_m < 2.5 && angle_deg < 5.0, "Ship riding along curve must report AUF KURS");
    });

    let off_course = arc.sample_telemetry(mid_pt + Vec3::new(38.0, 0.0, 0.0), Vec3::X);
    off_course.transfer(|_, cross_m, angle_deg, aligned| {
        assert!(!aligned && cross_m > 30.0 && angle_deg > 45.0, "Ship far off trajectory must report ABWEICHUNG");
    });
}

#[test]
fn test_nose_camera_marks_lunar_target_without_releasing_console_and_renders_flight_director() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut pilot = assemble_nose_rocket(world.as_mut());

    struct DirectorHudProbe {
        compass_quads: usize,
        burn_gold_quads: usize,
        top_right_strip_quads: usize,
        total_quads: usize,
    }
    impl MeshTarget for DirectorHudProbe {
        fn push_quad(&mut self, quad: Quad) {
            self.total_quads += 1;
            quad.transfer(|v0, _, v2, _, _, col| {
                let c = (v0 + v2) * 0.5;
                if c.x.abs() < 0.45 && c.y.abs() < 0.55 {
                    if col.z > 0.78 && col.y > 0.60 && col.x < 0.45 {
                        self.compass_quads += 1;
                    }
                    if col.x > 0.94 && col.y > 0.70 && col.z < 0.35 {
                        self.burn_gold_quads += 1;
                    }
                }
                if c.x > 0.55 && c.y > 0.78 {
                    self.top_right_strip_quads += 1;
                }
            });
        }
    }

    let mut uncharted_hud = DirectorHudProbe {
        compass_quads: 0,
        burn_gold_quads: 0,
        top_right_strip_quads: 0,
        total_quads: 0,
    };
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut uncharted_hud);
    assert!(
        uncharted_hud.compass_quads >= 48
            && uncharted_hud.burn_gold_quads == 0
            && uncharted_hud.top_right_strip_quads > 30,
        "Uncharted Nose camera must render 360-degree compass ring and top-right camera strip"
    );

    let inv = pilot.view_projection_matrix(1.0).inverse();
    let cam_eye = inv.project_point3(Vec3::ZERO);
    let cam_fwd = (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - cam_eye).normalize();
    let cam_right = (inv.project_point3(Vec3::new(0.5, 0.0, 0.0)) - cam_eye).normalize();
    let cam_up = (inv.project_point3(Vec3::new(0.0, 0.5, 0.0)) - cam_eye).normalize();
    let to_moon = (CelestialSystem::moon_center(32) - cam_eye).normalize();
    let yaw_deg = to_moon.dot(cam_right).atan2(to_moon.dot(cam_fwd)).to_degrees();
    let pitch_deg = (-to_moon.dot(cam_up)).atan2(to_moon.dot(cam_fwd)).to_degrees();
    pilot.rotate_degrees(yaw_deg, pitch_deg);

    assert!(pilot.use_left_hand(world.as_mut()), "Left-click in Nose Camera must mark lunar target");
    assert_eq!(pilot.collect_camera_previews(1.62, &mut Vec::new()), 1, "Left-click in Nose Camera must keep pilot at FlightConsole");

    let mut charted_hud = DirectorHudProbe {
        compass_quads: 0,
        burn_gold_quads: 0,
        top_right_strip_quads: 0,
        total_quads: 0,
    };
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut charted_hud);
    assert!(
        charted_hud.total_quads > uncharted_hud.total_quads + 65
            && charted_hud.burn_gold_quads >= 8,
        "Marking lunar target must add BURN diamond magnet, target brackets, and course readout ({} -> {})",
        uncharted_hud.total_quads,
        charted_hud.total_quads
    );

    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    for _ in 0..30 {
        pilot.step_motion(1.0 / 120.0, &burn, world.as_ref());
    }
    assert!(pilot.use_right_hand(world.as_mut()), "Right-click must still release FlightConsole cleanly");
}
