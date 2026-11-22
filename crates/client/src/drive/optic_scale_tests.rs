use glam::{Mat4, Quat, Vec2, Vec3};
use world::{BlockMedium, BlockMold, CelestialSystem, MeshTarget, PlanetaryWorld, Quad};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar, AvatarMotor, AvatarOptics, HandGesture, PlanetaryAvatar};
use crate::controls::{ActionBindings, ControlAction};
use crate::flight::SurfaceFlight;
use crate::menu::{
    ApproachZoneScale, BoresightTape, ChevronReadoutScale, GlassPlateScale, GraduatedSpine,
    OpticOverlay, PhosphorTone, RangeScale, RangeTapeOverlay, ReticleStandoffScale, ScaleDomain,
    ScreenPoint, SilentOverlay, TargetLockOverlay,
};
use crate::zero_g_flight::ZeroGFlight;
use super::{HullClearance, PadClearance};

struct ScaleBandProbe {
    spine_quads: usize,
    peak_quads: usize,
    min_x: f32,
    highest_badge_y: f32,
}

impl ScaleBandProbe {
    const fn empty() -> Self {
        Self { spine_quads: 0, peak_quads: 0, min_x: 2.0, highest_badge_y: -2.0 }
    }
}

impl MeshTarget for ScaleBandProbe {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, col| {
            let c = (v0 + v2) * 0.5;
            let q_min_x = v0.x.min(v1.x).min(v2.x).min(v3.x);
            if (0.11..0.38).contains(&c.x) && (-0.26..0.32).contains(&c.y) {
                self.spine_quads += 1;
                self.min_x = self.min_x.min(q_min_x);
                assert!(
                    col.y > col.z && col.z > col.x && ((col.z - col.x) - 0.15 * (col.y - col.x)).abs() < 2e-4,
                    "Every range-scale quad must use the monochromatic 129-deg HUD green hue, got {col:?}"
                );
                if (col - PhosphorTone::PeakPhosphor.color()).length() < 1e-3 {
                    self.peak_quads += 1;
                }
                if (col - PhosphorTone::BadgeWell.color()).length() < 1e-3 {
                    self.highest_badge_y = self.highest_badge_y.max(c.y);
                }
            }
        });
    }
}

struct LaunchpadRocket {
    with_camera: bool,
}

impl LaunchpadRocket {
    const fn new(with_camera: bool) -> Self {
        Self { with_camera }
    }

    fn erect(self, world: &mut dyn PlanetaryWorld) -> f32 {
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
        if self.with_camera {
            assert!(world.place_molded_surface_block(
                Vec3::new(1.25, g0 + 1.75, 1.25),
                Vec3::NEG_Z,
                av,
                BlockMold::cube().in_medium(BlockMedium::RocketCamera),
            ));
        }
        g0
    }

    fn board(self, world: &mut dyn PlanetaryWorld) -> PlanetaryAvatar {
        self.erect(world);
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
}

fn capture_reticle_and_scale(pilot: &PlanetaryAvatar) -> (Vec2, Vec<Vec2>, f32) {
    struct AnchorProbe {
        first: Option<Vec2>,
        scale_centers: Vec<Vec2>,
        scale_min_x: f32,
    }
    impl MeshTarget for AnchorProbe {
        fn push_quad(&mut self, quad: Quad) {
            quad.transfer(|v0, v1, v2, v3, _, col| {
                let c = Vec2::new((v0.x + v2.x) * 0.5, (v0.y + v2.y) * 0.5);
                if self.first.is_none() {
                    self.first = Some(c);
                }
                if (col - PhosphorTone::GlassTint.color()).length() < 1e-3
                    || (col - PhosphorTone::SpineRule.color()).length() < 1e-3
                    || (col - PhosphorTone::BadgeWell.color()).length() < 1e-3
                {
                    self.scale_centers.push(c);
                    self.scale_min_x = self.scale_min_x.min(v0.x.min(v1.x).min(v2.x).min(v3.x));
                }
            });
        }
    }
    let mut probe = AnchorProbe { first: None, scale_centers: Vec::new(), scale_min_x: 2.0 };
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut probe);
    (probe.first.expect("crosshair quad"), probe.scale_centers, probe.scale_min_x)
}

#[test]
fn test_range_scale_decorators_compose_monochromatic_hud_green_gap_and_sink_intensity() {
    let ux = 0.5625;
    let aim = ScreenPoint::center();
    let spine = GlassPlateScale::new(GraduatedSpine::new(ScaleDomain::LunarPad, ux), ux);
    let low_chain = ReticleStandoffScale::beside(
        ChevronReadoutScale::belly_radar(ApproachZoneScale::belly_touchdown(spine, -2.0), 4.0, -2.0),
        aim,
        ux,
    );
    let commuted_low = ChevronReadoutScale::belly_radar(
        ApproachZoneScale::belly_touchdown(ReticleStandoffScale::beside(spine, aim, ux), -2.0),
        4.0,
        -2.0,
    );
    let high_chain = BoresightTape::belly(aim, 64.0, 8.0, ux);
    let danger_chain = BoresightTape::belly(aim, 12.0, -18.0, ux);

    let mut low_probe = ScaleBandProbe::empty();
    let mut commuted_probe = ScaleBandProbe::empty();
    let mut high_probe = ScaleBandProbe::empty();
    let mut danger_probe = ScaleBandProbe::empty();
    low_chain.emit_scale(&mut low_probe);
    commuted_low.emit_scale(&mut commuted_probe);
    high_chain.emit_scale(&mut high_probe);
    danger_chain.emit_scale(&mut danger_probe);

    assert_eq!(low_probe.spine_quads, commuted_probe.spine_quads);
    assert!((low_probe.highest_badge_y - commuted_probe.highest_badge_y).abs() < 1e-6);
    assert!(high_probe.highest_badge_y > low_probe.highest_badge_y + 0.15);
    assert!(
        low_probe.min_x >= 0.12,
        "ReticleStandoffScale must keep a generous horizontal gap from crosshair tip (0.0234), got min_x={:.4}",
        low_probe.min_x
    );
    assert_eq!(low_probe.peak_quads, 0, "Gentle descent must use nominal HUD green luminance");
    assert_eq!(high_probe.peak_quads, 0, "Upward climb must not trigger peak sink luminance");
    assert!(danger_probe.peak_quads >= 6, "Steep sink near ground must elevate chevron/band to peak HUD green");
}

#[test]
fn test_optic_overlay_decorators_compose_silent_lock_and_tape_layers() {
    let mut silent_probe = ScaleBandProbe::empty();
    SilentOverlay.emit(&mut silent_probe);
    assert_eq!(silent_probe.spine_quads, 0);

    let vp = Mat4::perspective_rh_gl(1.1, 16.0 / 9.0, 0.1, 500.0)
        * Mat4::look_to_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);
    let locked = TargetLockOverlay::new(SilentOverlay, Vec3::new(0.0, 0.0, -40.0), vp, 0.5625);
    let full = RangeTapeOverlay::new(
        locked,
        ReticleStandoffScale::beside(
            ChevronReadoutScale::nose_lock(
                ApproachZoneScale::nose_braking(
                    GlassPlateScale::new(GraduatedSpine::new(ScaleDomain::CosmicTarget, 0.5625), 0.5625),
                    -24.0,
                ),
                384.0,
                -24.0,
            ),
            ScreenPoint::center(),
            0.5625,
        ),
    );
    let mut full_probe = ScaleBandProbe::empty();
    full.emit(&mut full_probe);
    assert!(full_probe.spine_quads > 40, "RangeTapeOverlay must emit decorated TGT spine");
}

#[test]
fn test_hull_clearance_resolves_lunar_approach_above_gravity_halo_and_deep_space() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = LaunchpadRocket::new(false).erect(world.as_mut());
    let assembly = world
        .board_vehicle(
            Vec3::new(1.75, g0 + 1.80, 0.75),
            Vec3::new(-0.985, -0.174, 0.0).normalize(),
            false,
        )
        .expect("Rocket perch shell must resolve cleanly from CabinSeat");
    let atlas = world.celestial_atlas();
    let moon_c = CelestialSystem::moon_center(32);

    let lunar_80m = HullClearance::new(
        &assembly,
        moon_c + Vec3::new(0.25, 16.0 + 80.0, 0.25),
        Vec3::new(0.0, -14.0, 0.0),
        Quat::IDENTITY,
    )
    .sample_pad(atlas, world.as_ref());
    lunar_80m.transfer(|in_well, alt, rate| {
        assert!(in_well && (74.0..=84.0).contains(&alt) && (rate + 14.0).abs() < 0.1);
    });

    let deep = HullClearance::new(&assembly, moon_c * 0.5, Vec3::ZERO, Quat::IDENTITY)
        .sample_pad(atlas, world.as_ref());
    assert_eq!(deep, PadClearance::DeepSpace);
}

#[test]
fn test_nose_and_belly_scales_stay_locked_to_crosshair_without_moving_with_mouse() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut pilot = LaunchpadRocket::new(true).board(world.as_mut());

    let (_, uncharted_scale, _) = capture_reticle_and_scale(&pilot);
    assert!(uncharted_scale.is_empty(), "Uncharted Nose Camera must omit target range tape");

    let cycle_next = ActionBindings::seeded().trigger_for(ControlAction::MoldRight);
    assert!(pilot.dispatch_trigger(cycle_next));
    let (belly_aim_0, belly_scale_0, belly_min_x_0) = capture_reticle_and_scale(&pilot);
    assert!(belly_scale_0.len() >= 7 && belly_min_x_0 - (belly_aim_0.x + 0.0234) >= 0.10);

    pilot.rotate_degrees(24.0, -14.0);
    let (belly_aim_1, belly_scale_1, belly_min_x_1) = capture_reticle_and_scale(&pilot);
    assert!((belly_aim_1 - belly_aim_0).length() > 0.15 && belly_scale_1.len() == belly_scale_0.len());
    for (c0, c1) in belly_scale_0.iter().zip(belly_scale_1.iter()) {
        assert!(
            ((*c1 - belly_aim_1) - (*c0 - belly_aim_0)).length() < 1e-5,
            "Belly ALT scale must stay locked at fixed offset from boresight crosshair under mouse look"
        );
    }
    assert!((belly_min_x_1 - belly_aim_1.x - (belly_min_x_0 - belly_aim_0.x)).abs() < 1e-5);

    let cycle_prev = ActionBindings::seeded().trigger_for(ControlAction::MoldLeft);
    assert!(pilot.dispatch_trigger(cycle_prev));
    let inv = pilot.view_projection_matrix(1.0).inverse();
    let eye = inv.project_point3(Vec3::ZERO);
    let fwd = (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - eye).normalize();
    let right = (inv.project_point3(Vec3::new(0.5, 0.0, 0.0)) - eye).normalize();
    let up = (inv.project_point3(Vec3::new(0.0, 0.5, 0.0)) - eye).normalize();
    let to_moon = (CelestialSystem::moon_center(32) - eye).normalize();
    pilot.rotate_degrees(
        to_moon.dot(right).atan2(to_moon.dot(fwd)).to_degrees(),
        (-to_moon.dot(up)).atan2(to_moon.dot(fwd)).to_degrees(),
    );
    assert!(pilot.use_left_hand(world.as_mut()));

    let (nose_aim_a, nose_scale_a, _) = capture_reticle_and_scale(&pilot);
    pilot.rotate_degrees(-26.0, 12.0);
    let (nose_aim_b, nose_scale_b, _) = capture_reticle_and_scale(&pilot);
    assert!(!nose_scale_a.is_empty() && (nose_aim_b - nose_aim_a).length() > 0.12);
    for (ca, cb) in nose_scale_a.iter().zip(nose_scale_b.iter()) {
        assert!(
            ((*cb - nose_aim_b) - (*ca - nose_aim_a)).length() < 1e-5,
            "Nose TGT scale must stay locked to boresight crosshair instead of moving with mouse gaze"
        );
    }
}
