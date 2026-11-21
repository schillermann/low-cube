use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use glam::Vec3;
use world::{CompanionPortal as _, PlanetaryWorld};
use crate::avatar::{AvatarMotor as _, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};

pub struct HeapLedger;

thread_local! {
    static TRACKING_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for HeapLedger {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACKING_ACTIVE.with(|active| {
            if active.get() {
                ALLOCATION_COUNT.with(|count| {
                    count.set(count.get() + 1);
                });
            }
        });
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

pub fn count_allocations<F: FnOnce()>(f: F) -> usize {
    ALLOCATION_COUNT.with(|c| c.set(0));
    TRACKING_ACTIVE.with(|a| a.set(true));
    f();
    TRACKING_ACTIVE.with(|a| a.set(false));
    ALLOCATION_COUNT.with(|c| c.get())
}

struct WalkControls;

impl KeyboardControls for WalkControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(forward);
    }
}

struct JumpAndSprintControls;

impl KeyboardControls for JumpAndSprintControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(forward);
        target.jump();
        target.sprint();
    }
}

#[test]
fn test_tracking_allocator_detects_heap_allocation() {
    let allocs = count_allocations(|| {
        let boxed = Box::new(42);
        assert_eq!(*boxed, 42);
    });
    assert!(allocs > 0, "Tracking allocator must detect heap allocations");
}

#[test]
fn test_hot_simulation_loop_zero_runtime_allocations() {
    let world = world::BiomedCubeWorld::beveled(32, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    let walk_ctrl = WalkControls;
    let sprint_ctrl = JumpAndSprintControls;
    let dt = 1.0 / 60.0;

    // Warm-up step
    avatar.step_motion(dt, &walk_ctrl, &world);

    // Audit 600 frames of walking (10 seconds)
    let walk_allocs = count_allocations(|| {
        for _ in 0..600 {
            avatar.step_motion(dt, &walk_ctrl, &world);
        }
    });
    assert_eq!(walk_allocs, 0, "Walking must have zero runtime allocations in hot simulation loop");

    // Audit 600 frames of sprinting and jumping
    let sprint_allocs = count_allocations(|| {
        for _ in 0..600 {
            avatar.step_motion(dt, &sprint_ctrl, &world);
        }
    });
    assert_eq!(sprint_allocs, 0, "Sprinting/jumping must have zero runtime allocations in hot simulation loop");

    // Audit 600 frames of walking, sprinting, and jumping with active BurdenedGait load
    struct HeavySatchel;
    impl crate::server_satchel::StartingSatchel for HeavySatchel {
        fn provision(&self, satchel: crate::attire::BeltSatchel) -> crate::attire::BeltSatchel {
            satchel.with_stowed_relic(crate::attire::HeldRelic::QuarriedStone, 12)
        }
    }
    let mut burdened = PlanetaryAvatar::at(spawn).with_starting_satchel(&HeavySatchel);
    burdened.step_motion(dt, &walk_ctrl, &world);
    let burdened_allocs = count_allocations(|| {
        for frame in 0..600 {
            let ctrl: &dyn KeyboardControls = if frame % 2 == 0 { &walk_ctrl } else { &sprint_ctrl };
            burdened.step_motion(dt, ctrl, &world);
        }
    });
    assert_eq!(burdened_allocs, 0, "BurdenedGait walking/sprinting/jumping must perform zero runtime heap allocations");

    // Audit 600 frames through BlackBoxAvatar with active LiveTape and SilentTape Null Object
    let mut recorded = crate::black_box::BlackBoxAvatar::wrapping(
        Box::new(PlanetaryAvatar::at(spawn)),
        Box::new(crate::black_box::LiveTape::fresh()),
    );
    recorded.step_motion(dt, &sprint_ctrl, &world);
    let live_tape_allocs = count_allocations(|| {
        for _ in 0..600 {
            recorded.step_motion(dt, &sprint_ctrl, &world);
        }
    });
    assert_eq!(live_tape_allocs, 0, "Active LiveTape flight blackbox must perform zero runtime heap allocations");

    struct NullTarget;
    impl world::MeshTarget for NullTarget {
        fn push_quad(&mut self, _quad: world::Quad) {}
    }
    let mut sink = NullTarget;
    let mold = world::BlockMold::with_medium(world::MoldArchetype::FullCube, world::CardinalTurn::Forward, world::BlockMedium::TurfEarth);
    let mini_allocs = count_allocations(|| {
        for _ in 0..60 {
            mold.emit_miniature(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z, 0.25, Vec3::ONE, Vec3::ONE, &mut sink);
        }
    });
    assert_eq!(mini_allocs, 0, "Miniature block emission in hand/satchel must perform zero heap allocations");

    // Audit multiplayer walking on HostedPlanet & RemotePlanet while a companion drives a vehicle
    let host = sanctuary::SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(world::BiomedCubeWorld::barren_beveled(16, 0.5)),
        16,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("host addr");
    let mut hosted = sanctuary::HostedPlanet::new(host);
    let mut guest = sanctuary::RemotePlanet::dial(sanctuary::JoinTarget::Direct(addr), 16, 0.5).expect("dial guest");
    for _ in 0..6 {
        let _ = guest.step_surface(0.03);
        let _ = hosted.step_surface(0.03);
    }
    let craft_id = world::CraftIdentity::new(1);
    let axes = world::VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let driven = world::DrivenCraft::new(
        0,
        craft_id,
        world::CraftPose::parked(spawn + Vec3::X * 4.0, axes, Vec3::Z),
    );
    guest.sync_pilot_craft(world::PilotCraft::Driving(driven));
    let _ = guest.step_surface(0.03);
    let _ = hosted.step_surface(0.03);
    avatar.step_motion(dt, &walk_ctrl, &hosted);
    let mp_walk_allocs = count_allocations(|| {
        for _ in 0..120 {
            avatar.step_motion(dt, &walk_ctrl, &hosted);
        }
    });
    assert_eq!(
        mp_walk_allocs, 0,
        "Multiplayer walking alongside active companion vehicles must perform zero heap allocations"
    );
}
