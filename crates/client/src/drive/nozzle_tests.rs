use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BlockMedium, BlockMold, CelestialSystem, MeshTarget, Quad};
use crate::attire::{BeltSatchel, HeldRelic, LimbFrame, YouthAttire};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarPresence as _, HandGesture, PlanetaryAvatar,
};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, InputTrigger, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;

struct NozzleProbe {
    lip_normals: Vec<Vec3>,
    plume_count: usize,
    glow_count: usize,
    count: usize,
}

impl NozzleProbe {
    fn empty() -> Self {
        Self { lip_normals: Vec::new(), plume_count: 0, glow_count: 0, count: 0 }
    }
}

impl MeshTarget for NozzleProbe {
    fn push_quad(&mut self, quad: Quad) {
        self.count += 1;
        let g = quad.radiate(|g| g);
        if g < -1.5 {
            self.plume_count += 1;
        } else if g > 0.2 {
            self.glow_count += 1;
        }
        quad.transfer(|v0, v1, v2, _, n, _| {
            if (v1 - v0).length() < 0.20 && (v2 - v1).length() < 0.06 {
                self.lip_normals.push(n);
            }
        });
    }
}

fn right_mold_code(avatar: &PlanetaryAvatar) -> u8 {
    let mut probe = PresenceProbe::at_origin();
    avatar.report_presence(&mut probe);
    probe
        .presence()
        .transfer(|_, _, _, att| att.transfer(|_, _, molds| molds.on_side(1.0).chronicle_bytes()[0]))
}

#[test]
fn test_nozzle_in_palm_starts_downward_and_mouse_wheel_toggles_steuerduese() {
    let squire = YouthAttire::squire();
    let planks = squire.with_relics(HeldRelic::PlankBlock, HeldRelic::PlankBlock);
    let mut satchel = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::RocketNozzle, 2)
        .reconcile_attire(planks);
    satchel = satchel.map_mold(planks, BlockMold::next_form).map_mold(planks, BlockMold::next_form);
    assert_eq!(satchel.palm_molds().on_side(1.0).chronicle_bytes()[0], 2);

    let mut avatar = PlanetaryAvatar::with_attire_and_bindings(
        Vec3::new(0.0, 17.0, 0.0),
        Box::new(SurfaceFlight::calibrated(0.522, 9.81, 1.0)),
        planks,
        ActionBindings::seeded(),
    );
    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    assert_eq!(right_mold_code(&avatar), 1);

    avatar.apply_attire(squire.with_relics(HeldRelic::RocketNozzle, HeldRelic::PlankBlock));
    assert_eq!(right_mold_code(&avatar), 0, "Equipping RocketNozzle must always reset to downward Main Nozzle");
    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    assert_eq!(right_mold_code(&avatar), 1, "Wheel scroll must switch to Steuerdüse");
    assert!(avatar.dispatch_trigger(InputTrigger::WheelUp));
    assert_eq!(right_mold_code(&avatar), 0, "Second wheel scroll must return to Main Nozzle");

    let palm_frame = LimbFrame::new(Vec3::ZERO, Vec3::NEG_Y, Vec3::X, Vec3::NEG_Z);
    let mut main_probe = NozzleProbe::empty();
    HeldRelic::RocketNozzle.figure(BlockMold::cube()).emit_in_palm(1.0, &palm_frame, &mut main_probe);
    assert_eq!(main_probe.plume_count, 0, "Idle Main Nozzle in palm must not emit exhaust fire");
    assert_eq!(main_probe.glow_count, 0, "Idle Main Nozzle must be unlit hollow metal");
    assert!(
        main_probe.lip_normals.iter().any(|n| n.y < -0.95),
        "Main Nozzle hollow mouth rim in palm must point downward (-Y), got {:?}",
        main_probe.lip_normals
    );
    let mut rcs_probe = NozzleProbe::empty();
    HeldRelic::RocketNozzle.figure(BlockMold::cube().next_form()).emit_in_palm(1.0, &palm_frame, &mut rcs_probe);
    assert_ne!(main_probe.count, rcs_probe.count, "Steuerdüse must emit distinct 5-horn RCS geometry in palm");
    assert_eq!(rcs_probe.plume_count, 0, "Idle Steuerdüse in palm must not emit exhaust fire");
    assert_eq!(rcs_probe.glow_count, 0, "Idle Steuerdüse must be unlit hollow metal");
    assert!(
        rcs_probe.lip_normals.iter().any(|n| n.z > 0.90),
        "Steuerdüse outward horn mouth rim in palm must point toward the player (+Z), got {:?}",
        rcs_probe.lip_normals
    );
}

#[test]
fn test_spacecraft_with_steuerduese_pitches_into_lateral_trajectory() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
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
        BlockMold::cube().in_medium(BlockMedium::RocketNozzle),
    ));
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
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.25),
        Vec3::NEG_Z,
        av_pos,
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));

    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));

    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::Space, true);
    controls.register_key(KeyCode::KeyW, true);
    for _ in 0..180 {
        pilot.step_motion(1.0 / 120.0, &controls, world.as_ref());
    }
    let mut probe = PresenceProbe::at_origin();
    pilot.report_presence(&mut probe);
    let pos = probe.position();
    assert!(
        pos.y > g0 + 4.0 && (pos.x - 1.25).abs() > 6.0,
        "Steuerdüse pitch + thrust must steer craft laterally, got {pos:?}"
    );
}

#[test]
fn test_flight_console_mounts_from_one_block_in_front_of_crane_arm_without_stepping_back() {
    use world::{BiomedCubeWorld, PlanetaryWorld, WorldEarthworks as _};

    for &(u, v, yaw_deg) in &[
        (1.75f32, 0.25f32, -90.0f32),
        (2.05, 0.25, -90.0),
        (0.75, 1.05, 0.0),
        (1.25, 1.05, 0.0),
        (-0.55, 0.25, 90.0),
    ] {
        let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
        let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
        let far = Vec3::new(5.0, g0 + 1.0, 5.0);
        assert!(world.place_molded_surface_block(
            Vec3::new(0.25, g0 + 1.5, 0.25),
            Vec3::NEG_Y,
            far,
            BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        ));
        let spawn = world.constrain_to_surface(Vec3::new(u, 22.0, v), 1.0);
        let squire = YouthAttire::squire().with_relics(HeldRelic::FlightConsole, HeldRelic::EmptyPalm);
        let mut avatar = PlanetaryAvatar::with_attire_and_bindings(
            spawn,
            Box::new(SurfaceFlight::calibrated(0.522, 9.81, 1.0)),
            squire,
            ActionBindings::seeded(),
        );
        avatar.rotate_degrees(yaw_deg, 0.0);
        assert!(
            HandGesture::RightTap.perform(&mut avatar, &mut world),
            "FlightConsole must mount onto empty crane arm from ({u}, {v}) without stepping back",
        );
        assert!(
            world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 0.25), Vec3::NEG_Y, 1, false),
            "Placed FlightConsole from ({u}, {v}) must be mounted on the crane cluster",
        );
    }

    for &(u, v, yaw_deg, expected_cell) in &[
        (0.75f32, 1.25f32, 0.0f32, Vec3::new(0.75, 0.0, 0.25)),
        (1.25, 1.05, 0.0, Vec3::new(1.25, 0.0, 0.75)),
        (2.05, 0.25, -90.0, Vec3::new(1.75, 0.0, 0.25)),
    ] {
        let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
        let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
        let far = Vec3::new(5.0, g0 + 1.0, 5.0);
        for medium in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
            assert!(world.place_molded_surface_block(
                Vec3::new(0.25, g0 + 1.5, 0.25),
                Vec3::NEG_Y,
                far,
                BlockMold::cube().in_medium(medium),
            ));
        }
        let spawn = world.constrain_to_surface(Vec3::new(u, 22.0, v), 1.0);
        let squire = YouthAttire::squire().with_relics(HeldRelic::FlightConsole, HeldRelic::EmptyPalm);
        let mut avatar = PlanetaryAvatar::with_attire_and_bindings(
            spawn,
            Box::new(SurfaceFlight::calibrated(0.522, 9.81, 1.0)),
            squire,
            ActionBindings::seeded(),
        );
        avatar.rotate_degrees(yaw_deg, 0.0);
        assert!(
            HandGesture::RightTap.perform(&mut avatar, &mut world),
            "FlightConsole must attach to crane rocket from ({u}, {v}) without stepping back",
        );
        assert_eq!(
            world.harvest_surface(Vec3::new(expected_cell.x, g0 + 2.4, expected_cell.z), Vec3::NEG_Y),
            Some(BlockMedium::FlightConsole),
            "FlightConsole placed from ({u}, {v}) must occupy expected crane cell {expected_cell:?}",
        );
    }
}
