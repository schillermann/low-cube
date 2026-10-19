mod app;
mod argument_aurora;
mod argument_celestial;
mod argument_physics;
mod attire;
mod aurora_cadence;
mod aurora_canopy;
mod avatar;
mod cadence;
mod celestial;
mod clock;
mod controls;
mod cycle_duration;
mod diurnal_pipeline;
mod envelope;
mod environment_aurora;
mod environment_celestial;
mod environment_physics;
mod file_aurora;
mod file_celestial;
mod file_physics;
mod firmament;
mod firmament_glsl;
mod flight;
mod gait;
mod gl_firmament;
mod heading;
mod horizon;
mod host_env;
mod launch;
mod lens;
mod order;
mod orientation;
mod phase;
mod physics;
mod posture;
mod server_aurora;
mod server_celestial;
mod server_physics;
mod session;
mod stride;
mod surface;
mod threshold;

#[cfg(test)]
mod allocation_audit;
#[cfg(test)]
#[global_allocator]
static ALLOCATOR: allocation_audit::TrackingAllocator = allocation_audit::TrackingAllocator;

#[cfg(test)]
mod aurora_tests;
#[cfg(test)]
mod ballistic_tests;
#[cfg(test)]
mod cadence_tests;
#[cfg(test)]
mod celestial_tests;
#[cfg(test)]
mod chunk_traversal_tests;
#[cfg(test)]
mod circuit_tests;
#[cfg(test)]
mod controls_tests;
#[cfg(test)]
mod flight_tests;
#[cfg(test)]
mod mountain_tests;
#[cfg(test)]
mod physics_tests;
#[cfg(test)]
mod posture_tests;
#[cfg(test)]
mod seam_clearance_tests;
#[cfg(test)]
mod threshold_tests;
#[cfg(test)]
mod tilt_tests;
#[cfg(test)]
mod traversal_tests;
#[cfg(test)]
mod visual_tests;

use winit::event_loop::{ControlFlow, EventLoop};
use app::LowCubeApp;
use launch::SessionLaunch;

fn world_scale() -> u32 {
    std::env::args()
        .find_map(|arg| {
            arg.strip_prefix("--blocks=")
                .and_then(|val| val.parse::<u32>().ok())
        })
        .unwrap_or(128)
}

fn tron_active() -> bool {
    std::env::args().any(|arg| arg == "--tron" || arg == "tron")
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let tron = tron_active();
    let title = if tron {
        "Low-Cube [TRON] [WASD: Move, Space: Jump, Shift: Sneak, M: Mirror, 1-7: Attire, H: Relic, T: Tron, Esc: Exit]"
    } else {
        "Low-Cube [WASD: Move, Space: Jump, Shift: Sneak, M: Mirror, 1-7: Attire, H: Relic, T: Tron, Esc: Exit]"
    };

    let launch = SessionLaunch::with_tron(
        title,
        1280.0,
        720.0,
        world_scale(),
        tron,
    );
    let mut app = LowCubeApp::launching(Box::new(launch));
    event_loop.run_app(&mut app).expect("Application run failed");
}
