mod app;
mod avatar;
mod controls;
mod launch;
mod orientation;
mod session;
mod surface;

use winit::event_loop::{ControlFlow, EventLoop};
use app::LowCubeApp;
use launch::SessionLaunch;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let launch = SessionLaunch::configured(
        "Low-Cube [WASD: Move, Mouse: Look, I: Invert Y, O: Invert X, Esc: Exit]",
        1280.0,
        720.0,
        16,
    );
    let mut app = LowCubeApp::launching(Box::new(launch));
    event_loop.run_app(&mut app).expect("Application run failed");
}
