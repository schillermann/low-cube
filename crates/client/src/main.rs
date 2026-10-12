mod app;
mod avatar;
mod basis;
mod controls;
mod flight;
mod launch;
mod lens;
mod order;
mod orientation;
mod phase;
mod session;
mod stride;
mod surface;
mod threshold;

#[cfg(test)]
mod controls_tests;
#[cfg(test)]
mod flight_tests;
#[cfg(test)]
mod mountain_tests;
#[cfg(test)]
mod threshold_tests;
#[cfg(test)]
mod traversal_tests;

use winit::event_loop::{ControlFlow, EventLoop};
use app::LowCubeApp;
use launch::SessionLaunch;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let launch = SessionLaunch::configured(
        "Low-Cube [WASD: Move, Space: Jump, Mouse: Look, I: Invert Y, O: Invert X, Esc: Exit]",
        1280.0,
        720.0,
        32,
    );
    let mut app = LowCubeApp::launching(Box::new(launch));
    event_loop.run_app(&mut app).expect("Application run failed");
}
