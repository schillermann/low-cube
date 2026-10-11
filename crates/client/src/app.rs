use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::launch::SessionLaunch;
use crate::session::GameSession;

/// State pattern interface for application lifecycle transitions.
pub trait AppLifecycle {
    fn resume(self: Box<Self>, event_loop: &ActiveEventLoop) -> Box<dyn AppLifecycle>;
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent);
    fn device_event(&mut self, event: DeviceEvent);
}

/// Initial pending state before display connection is available.
pub struct PendingLifecycle;

impl AppLifecycle for PendingLifecycle {
    fn resume(self: Box<Self>, event_loop: &ActiveEventLoop) -> Box<dyn AppLifecycle> {
        let launch = SessionLaunch::new();
        let session = launch.assemble(event_loop);
        Box::new(ActiveLifecycle::new(session))
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}
    fn device_event(&mut self, _event: DeviceEvent) {}
}

/// Transient state used during atomic lifecycle transition.
pub struct TransitioningLifecycle;

impl AppLifecycle for TransitioningLifecycle {
    fn resume(self: Box<Self>, _event_loop: &ActiveEventLoop) -> Box<dyn AppLifecycle> {
        self
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}
    fn device_event(&mut self, _event: DeviceEvent) {}
}

/// Active running state delegating to GameSession.
pub struct ActiveLifecycle {
    session: GameSession,
}

impl ActiveLifecycle {
    pub fn new(session: GameSession) -> Self {
        Self { session }
    }
}

impl AppLifecycle for ActiveLifecycle {
    fn resume(self: Box<Self>, _event_loop: &ActiveEventLoop) -> Box<dyn AppLifecycle> {
        self
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        self.session.handle_window_event(event_loop, event);
    }

    fn device_event(&mut self, event: DeviceEvent) {
        self.session.handle_device_event(event);
    }
}

/// Clean outer application driver: zero enums, pure polymorphic state dispatch.
pub struct LowCubeApp {
    lifecycle: Box<dyn AppLifecycle>,
}

impl LowCubeApp {
    pub fn new() -> Self {
        Self {
            lifecycle: Box::new(PendingLifecycle),
        }
    }
}

impl ApplicationHandler for LowCubeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let current = std::mem::replace(&mut self.lifecycle, Box::new(TransitioningLifecycle));
        self.lifecycle = current.resume(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.lifecycle.window_event(event_loop, id, event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.lifecycle.device_event(event);
    }
}
