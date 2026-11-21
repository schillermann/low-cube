use std::time::{Duration, Instant};
use glam::Vec3;
use fauna::SparrowFlock;
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarOptics as _, PlanetaryAvatar};
use crate::clock::{ControllableClock, MonotonicTime as _};
use crate::controls::{ActionBindings, InputTrigger, KeyMotion, KeyboardControls};
use super::pulse::EpochPulse;

#[test]
fn test_epoch_pulse_runs_identical_bird_and_world_ticks_at_5000_fps_and_60_fps() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let listener = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);

    let rig_clock = ControllableClock::starting_at(Instant::now());
    let mut rig_pulse = EpochPulse::anchored_to(Box::new(rig_clock.replica()));
    let mut rig_flock = SparrowFlock::seeded();
    let mut rig_ticks = 0_u32;
    for _ in 0..5_000 {
        rig_clock.advance_by(Duration::from_micros(200));
        let (_, beat) = rig_pulse.begin_frame();
        rig_ticks += beat.due_ticks();
        for _ in 0..beat.due_ticks() {
            let _ = rig_flock.step_spatial(beat.tick_seconds(), listener, Vec3::NEG_Z, &world);
        }
    }

    let laptop_clock = ControllableClock::starting_at(Instant::now());
    let mut laptop_pulse = EpochPulse::anchored_to(Box::new(laptop_clock.replica()));
    let mut laptop_flock = SparrowFlock::seeded();
    let mut laptop_ticks = 0_u32;
    for idx in 0..60 {
        let nanos = if idx % 3 == 2 { 16_666_668 } else { 16_666_666 };
        laptop_clock.advance_by(Duration::from_nanos(nanos));
        let (_, beat) = laptop_pulse.begin_frame();
        laptop_ticks += beat.due_ticks();
        for _ in 0..beat.due_ticks() {
            let _ = laptop_flock.step_spatial(beat.tick_seconds(), listener, Vec3::NEG_Z, &world);
        }
    }

    assert_eq!(rig_ticks, 120, "5000 FPS hardware must execute exact 120 world ticks per second");
    assert_eq!(laptop_ticks, 120, "60 FPS office laptop must execute exact 120 world ticks per second");

    let mut rig_quads = engine::CpuMeshBatch::empty();
    let mut laptop_quads = engine::CpuMeshBatch::empty();
    let cone = fauna::SightCone::omnidirectional(listener);
    rig_flock.emit_visible(cone, &mut rig_quads);
    laptop_flock.emit_visible(cone, &mut laptop_quads);

    let mut probe_rig = Vec::new();
    let mut probe_laptop = Vec::new();
    struct PosCapture<'a>(&'a mut Vec<Vec3>);
    impl world::MeshTarget for PosCapture<'_> {
        fn push_quad(&mut self, quad: world::Quad) {
            self.0.push(quad.transfer(|v0, _, _, _, _, _| v0));
        }
    }
    rig_flock.emit_visible(cone, &mut PosCapture(&mut probe_rig));
    laptop_flock.emit_visible(cone, &mut PosCapture(&mut probe_laptop));
    assert_eq!(
        probe_rig, probe_laptop,
        "Bird flight trajectories on 5000 FPS rig and 60 FPS laptop must be bit-identical"
    );
}

#[test]
fn test_elastic_horizon_dilates_on_fast_hardware_even_under_60hz_vsync_and_contracts_under_load() {
    let clock = ControllableClock::starting_at(Instant::now());
    let mut pulse = EpochPulse::anchored_to(Box::new(clock.replica()));

    for _ in 0..32 {
        let (work_start, _) = pulse.begin_frame();
        clock.advance_by(Duration::from_micros(400));
        pulse.record_work(work_start);
        clock.advance_by(Duration::from_micros(16_267));
    }
    let (_, fast_beat) = pulse.begin_frame();
    assert!(
        fast_beat.sight_scale() > 1.35,
        "Sub-millisecond pre-swap work must dilate SightCone horizon even with 60Hz VSync ({})",
        fast_beat.sight_scale()
    );

    for _ in 0..32 {
        let (work_start, _) = pulse.begin_frame();
        clock.advance_by(Duration::from_millis(14));
        pulse.record_work(work_start);
    }
    let (_, strained_beat) = pulse.begin_frame();
    assert!(
        strained_beat.sight_scale() < 0.72,
        "Heavy 14ms laptop frame work must contract SightCone horizon smoothly ({})",
        strained_beat.sight_scale()
    );
}

#[test]
fn test_stall_resync_and_subtick_avatar_interpolation_preserve_continuity() {
    let clock = ControllableClock::starting_at(Instant::now());
    let mut pulse = EpochPulse::anchored_to(Box::new(clock.replica()));

    clock.advance_by(Duration::from_secs(3));
    let (_, stalled) = pulse.begin_frame();
    assert_eq!(stalled.due_ticks(), 12, "Multi-second OS stall must cap catch-up at 12 ticks (0.10s)");

    clock.advance_by(Duration::from_nanos(4_166_667));
    let (_, half_step) = pulse.begin_frame();
    assert_eq!(half_step.due_ticks(), 0, "Frame immediately after stall resync must not spiral-of-death");
    assert!((half_step.subtick_blend() - 0.5).abs() < 0.02);

    let world = BiomedCubeWorld::of_size(16);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 9.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    let mut controls = KeyMotion::with_bindings(ActionBindings::seeded());
    controls.register_trigger(InputTrigger::Key(winit::keyboard::KeyCode::KeyW), true);

    let view_start = avatar.view_projection_matrix(1.0);
    avatar.step_motion(1.0 / 120.0, &controls, &world);
    let view_end = avatar.view_projection_matrix(1.0);
    avatar.blend_subtick(0.5);
    let view_mid = avatar.view_projection_matrix(1.0);

    assert_ne!(view_start, view_mid, "Half-tick blend must advance camera past prior tick");
    assert_ne!(view_mid, view_end, "Half-tick blend must interpolate smoothly before current tick");
}

#[test]
fn test_wake_gate_anchors_to_frame_origin_so_60hz_vsync_never_halves_to_30fps() {
    let clock = ControllableClock::starting_at(Instant::now());
    let mut pulse = EpochPulse::anchored_to(Box::new(clock.replica()));

    for _ in 0..24 {
        let (started, _) = pulse.begin_frame();
        // Simulate heavy 9.5ms multiplayer CPU/GPU work followed by blocking 60Hz VSync swap to 16.667ms:
        clock.advance_by(Duration::from_micros(9_500));
        pulse.record_work(started);
        clock.advance_by(Duration::from_micros(7_167));
        let post_vsync_now = clock.current_instant();
        let deadline = pulse.next_deadline(false);
        assert!(
            post_vsync_now >= deadline,
            "After 16.67ms VSync swap, WakeGate must schedule the next frame immediately instead of sleeping past the next VBlank"
        );
    }
}
