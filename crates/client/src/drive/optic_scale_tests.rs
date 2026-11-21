use glam::{Mat4, Quat, Vec3};
use world::{
    BlockMedium, BlockMold, CelestialSystem, MeshTarget, PlanetaryWorld, Quad,
};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar, AvatarMotor, AvatarOptics, HandGesture, PlanetaryAvatar};
use crate::controls::{ActionBindings, ControlAction};
use crate::flight::SurfaceFlight;
use crate::menu::{
    ApproachZoneScale, ChevronReadoutScale, GraduatedSpine, OpticOverlay, RangeScale,
    RangeTapeOverlay, ScaleDomain, SilentOverlay, TargetLockOverlay,
};
use crate::zero_g_flight::ZeroGFlight;
use super::{HullClearance, PadClearance};

struct ScaleBandProbe {
    spine_quads: usize,
    crimson_quads: usize,
    highest_badge_y: f32,
}

impl ScaleBandProbe {
    const fn new(spine_quads: usize, crimson_quads: usize, highest_badge_y: f32) -> Self {
        Self { spine_quads, crimson_quads, highest_badge_y }
    }

    const fn empty() -> Self {
        Self::new(0, 0, -1.0)
    }
}

impl MeshTarget for ScaleBandProbe {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, _, v2, _, _, col| {
            let c = (v0 + v2) * 0.5;
            if (0.04..0.24).contains(&c.x) && (-0.24..0.32).contains(&c.y) {
                self.spine_quads += 1;
                if col.x > 0.90 && col.y < 0.35 && col.z < 0.30 {
                    self.crimson_quads += 1;
                }
                if (col - Vec3::new(0.04, 0.10, 0.16)).length() < 0.015 {
                    self.highest_badge_y = self.highest_badge_y.max(c.y);
                }
            }
        });
    }
}

fn board_test_rocket(world: &mut dyn PlanetaryWorld) -> PlanetaryAvatar {
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
fn test_range_scale_decorator_chain_slides_chevron_and_flags_steep_sink() {
    let ux = 0.5625;
    let low_chain = ChevronReadoutScale::belly_radar(
        ApproachZoneScale::belly_touchdown(GraduatedSpine::new(ScaleDomain::LunarPad, ux), -2.0),
        4.0,
        -2.0,
    );
    let high_chain = ChevronReadoutScale::belly_radar(
        ApproachZoneScale::belly_touchdown(GraduatedSpine::belly_altitude(ux), 8.0),
        64.0,
        8.0,
    );
    let danger_chain = ChevronReadoutScale::belly_radar(
        ApproachZoneScale::belly_touchdown(GraduatedSpine::belly_altitude(ux), -18.0),
        12.0,
        -18.0,
    );

    let mut low_probe = ScaleBandProbe::empty();
    let mut high_probe = ScaleBandProbe::empty();
    let mut danger_probe = ScaleBandProbe::empty();
    low_chain.emit_scale(&mut low_probe);
    high_chain.emit_scale(&mut high_probe);
    danger_chain.emit_scale(&mut danger_probe);

    assert!(high_probe.highest_badge_y > low_probe.highest_badge_y + 0.15);
    assert_eq!(low_probe.crimson_quads, 0, "Gentle descent must remain emerald/amber");
    assert_eq!(high_probe.crimson_quads, 0, "Upward climb must not trigger sink danger");
    assert!(danger_probe.crimson_quads >= 6, "Steep sink near ground must trigger crimson warning");
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
        ChevronReadoutScale::nose_lock(
            ApproachZoneScale::nose_braking(GraduatedSpine::nose_target(0.5625), -24.0),
            384.0,
            -24.0,
        ),
    );
    let mut full_probe = ScaleBandProbe::empty();
    full.emit(&mut full_probe);
    assert!(full_probe.spine_quads > 40, "RangeTapeOverlay must emit decorated TGT spine");
}

#[test]
fn test_hull_clearance_resolves_lunar_approach_above_gravity_halo_and_deep_space() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
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

    let mid_void = moon_c * 0.5;
    let deep = HullClearance::new(&assembly, mid_void, Vec3::ZERO, Quat::IDENTITY)
        .sample_pad(atlas, world.as_ref());
    assert_eq!(deep, PadClearance::DeepSpace);
}

#[test]
fn test_nose_and_belly_cameras_switch_between_target_range_and_live_pad_altitude() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut pilot = board_test_rocket(world.as_mut());

    let mut uncharted_nose = ScaleBandProbe::empty();
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut uncharted_nose);
    assert_eq!(
        uncharted_nose.spine_quads, 0,
        "Uncharted Nose Camera must omit target range tape until a target is marked"
    );

    let cycle_next = ActionBindings::seeded().trigger_for(ControlAction::MoldRight);
    assert!(pilot.dispatch_trigger(cycle_next), "Wheel cycle must switch to CAM 2 BELLY");
    let mut belly_probe = ScaleBandProbe::empty();
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut belly_probe);
    assert!(
        belly_probe.spine_quads > 40,
        "Belly Camera must render live ALT pad-clearance scale even without a marked target"
    );

    let cycle_prev = ActionBindings::seeded().trigger_for(ControlAction::MoldLeft);
    assert!(pilot.dispatch_trigger(cycle_prev), "Wheel cycle back must return to CAM 1 NOSE");

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

    let mut charted_nose = ScaleBandProbe::empty();
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut charted_nose);
    assert!(
        charted_nose.spine_quads > 40,
        "Charted Nose Camera must render TGT vertical range scale alongside Orrery"
    );
}
