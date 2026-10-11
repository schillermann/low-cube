use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::bootstrap::SessionBootstrapper;
use crate::session::GameSession;

/// Lifecycle states for Winit event loop.
enum AppLifecycle {
    Uninitialized,
    Active(GameSession),
}

/// Clean outer application driver.
pub struct LowCubeApp {
    lifecycle: AppLifecycle,
}

impl LowCubeApp {
    pub fn new() -> Self {
        Self {
            lifecycle: AppLifecycle::Uninitialized,
        }
    }
}

impl ApplicationHandler for LowCubeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppLifecycle::Uninitialized = self.lifecycle {
            let bootstrapper = SessionBootstrapper::new();
            self.lifecycle = AppLifecycle::Active(bootstrapper.bootstrap(event_loop));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let AppLifecycle::Active(session) = &mut self.lifecycle {
            session.handle_window_event(event_loop, event);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let AppLifecycle::Active(session) = &mut self.lifecycle {
            session.handle_device_event(event);
        }
    }
}
