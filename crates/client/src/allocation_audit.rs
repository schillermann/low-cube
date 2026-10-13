use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use glam::Vec3;
use world::PlanetaryWorld;
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};

pub struct TrackingAllocator;

thread_local! {
    static TRACKING_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for TrackingAllocator {
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
    let world = world::BiomedCubeWorld::centripetal(32, 2.5, 4.0);
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
}
