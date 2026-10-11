mod app;
mod bootstrap;
mod controls;
mod player;
mod session;

use winit::event_loop::{ControlFlow, EventLoop};
use app::LowCubeApp;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = LowCubeApp::new();
    event_loop.run_app(&mut app).expect("Application run failed");
}
