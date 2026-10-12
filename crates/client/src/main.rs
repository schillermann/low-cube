mod app;
mod argument_physics;
mod avatar;
mod cadence;
mod clock;
mod controls;
mod environment_physics;
mod file_physics;
mod flight;
mod gait;
mod heading;
mod horizon;
mod launch;
mod lens;
mod order;
mod orientation;
mod phase;
mod physics;
mod posture;
mod server_physics;
mod session;
mod stride;
mod surface;
mod threshold;

#[cfg(test)]
mod cadence_tests;
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

use winit::event_loop::{ControlFlow, EventLoop};
use app::LowCubeApp;
use launch::SessionLaunch;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let launch = SessionLaunch::configured(
        "Low-Cube [WASD: Move, Double-W: Sprint, Space: Jump, Shift: Sneak, Mouse: Look, I: Invert Y, O: Invert X, Esc: Exit]",
        1280.0,
        720.0,
        32,
    );
    let mut app = LowCubeApp::launching(Box::new(launch));
    event_loop.run_app(&mut app).expect("Application run failed");
}
