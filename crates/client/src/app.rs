use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::launch::SessionAssembly;
use crate::session::Session;

/// State pattern interface for application lifecycle transitions.
pub trait AppLifecycle {
    fn resume(self: Box<Self>, event_loop: &ActiveEventLoop) -> Box<dyn AppLifecycle>;
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent);
    fn device_event(&mut self, event: DeviceEvent);
}

/// Initial pending state before display connection is available.
pub struct PendingLifecycle {
    assembly: Box<dyn SessionAssembly>,
}

impl PendingLifecycle {
    pub fn new(assembly: Box<dyn SessionAssembly>) -> Self {
        Self { assembly }
    }
}

impl AppLifecycle for PendingLifecycle {
    fn resume(self: Box<Self>, event_loop: &ActiveEventLoop) -> Box<dyn AppLifecycle> {
        let session = self.assembly.assemble(event_loop);
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

/// Active running state delegating to polymorphically wrapped Session.
pub struct ActiveLifecycle {
    session: Box<dyn Session>,
}

impl ActiveLifecycle {
    pub fn new(session: Box<dyn Session>) -> Self {
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
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(lifecycle: Box<dyn AppLifecycle>) -> Self {
        Self { lifecycle }
    }

    /// Secondary constructor initializing with a session assembly mechanism.
    pub fn launching(assembly: Box<dyn SessionAssembly>) -> Self {
        Self::new(Box::new(PendingLifecycle::new(assembly)))
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
