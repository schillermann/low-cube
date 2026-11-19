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

struct GlowProbe {
    normals: Vec<Vec3>,
    count: usize,
}

impl MeshTarget for GlowProbe {
    fn push_quad(&mut self, quad: Quad) {
        self.count += 1;
        if quad.radiate(|g| g > 0.4) {
            quad.transfer(|_, _, _, _, n, _| self.normals.push(n));
        }
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
    let mut main_probe = GlowProbe { normals: Vec::new(), count: 0 };
    HeldRelic::RocketNozzle.figure(BlockMold::cube()).emit_in_palm(1.0, &palm_frame, &mut main_probe);
    assert!(!main_probe.normals.is_empty());
    assert!(
        main_probe.normals.iter().any(|n| n.y < -0.95) && main_probe.normals.iter().all(|n| n.y <= 0.01),
        "Main Nozzle mouth in palm must point downward (-Y) and never upward, got {:?}",
        main_probe.normals
    );
    let mut rcs_probe = GlowProbe { normals: Vec::new(), count: 0 };
    HeldRelic::RocketNozzle.figure(BlockMold::cube().next_form()).emit_in_palm(1.0, &palm_frame, &mut rcs_probe);
    assert_ne!(main_probe.count, rcs_probe.count, "Steuerdüse must emit distinct RCS geometry in palm");
    assert!(
        rcs_probe.normals.iter().any(|n| n.z > 0.90) && rcs_probe.normals.iter().all(|n| n.z >= -0.30),
        "Steuerdüse nozzle cone in palm must point toward the player (+Z) with flange at the back (-Z), got {:?}",
        rcs_probe.normals
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
