use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

use world::PlanetaryWorld;
use engine::{CompositeMesh, CpuMeshBatch, CpuTronBatch, GraphicPipeline, PlanetaryMesh, ShaderProgram};
use crate::avatar::Avatar;
use crate::controls::{ControlAction, InputTrigger, KeyboardControls};
use crate::menu::{ModalDirective, SessionModal};
use crate::surface::PlatformSurface;

/// Interface representing an active, interactive display session.
pub trait Session {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
    fn handle_device_event(&mut self, event: DeviceEvent);
}

/// Fully initialized active game session with code-free primary constructor.
pub struct GameSession {
    surface: Box<dyn PlatformSurface>, primary_pipeline: Box<dyn GraphicPipeline>, primary_shader: Box<dyn ShaderProgram>,
    primary_mesh: PlanetaryMesh, tron_pipeline: Box<dyn GraphicPipeline>, tron_shader: Box<dyn ShaderProgram>,
    tron_mesh: PlanetaryMesh, tron_active: bool, world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>, controls: Box<dyn KeyboardControls>, modal: Box<dyn SessionModal>,
}

impl GameSession {
    /// Primary constructor doing zero computation or I/O.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface: Box<dyn PlatformSurface>, primary_pipeline: Box<dyn GraphicPipeline>, primary_shader: Box<dyn ShaderProgram>,
        primary_mesh: PlanetaryMesh, tron_pipeline: Box<dyn GraphicPipeline>, tron_shader: Box<dyn ShaderProgram>,
        tron_mesh: PlanetaryMesh, tron_active: bool, world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>, controls: Box<dyn KeyboardControls>, modal: Box<dyn SessionModal>,
    ) -> Self {
        Self {
            surface, primary_pipeline, primary_shader, primary_mesh,
            tron_pipeline, tron_shader, tron_mesh, tron_active,
            world, avatar, controls, modal,
        }
    }

    fn refresh_face_slice(&mut self, idx: usize) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            self.world.emit_face(idx, &mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron_mesh.slice_at(idx), &tron_buf);
            self.tron_mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::empty();
            self.world.emit_face(idx, &mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary_mesh.slice_at(idx), &cpu_buf);
            self.primary_mesh.store_slice(idx, next);
        }
    }

    fn refresh_modified_face(&mut self) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            let idx = self.world.emit_modified_face(&mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron_mesh.slice_at(idx), &tron_buf);
            self.tron_mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::empty();
            let idx = self.world.emit_modified_face(&mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary_mesh.slice_at(idx), &cpu_buf);
            self.primary_mesh.store_slice(idx, next);
        }
    }

    fn refresh_world_mesh(&mut self) {
        for idx in 0..self.world.face_count() { self.refresh_face_slice(idx); }
    }

    fn dispatch_gameplay_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        if self.controls.matches_action(ControlAction::TronMode, trigger) {
            if pressed {
                self.tron_active = !self.tron_active;
                self.refresh_world_mesh();
            }
        } else if !pressed || !self.avatar.dispatch_trigger(trigger) {
            self.controls.register_trigger(trigger, pressed);
        }
    }
}

impl Session for GameSession {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.surface.resize(size.width, size.height),
            WindowEvent::CursorMoved { position, .. } => {
                let point = self.surface.normalize_cursor(position.x, position.y);
                self.modal.move_cursor(point);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                if self.modal.open() {
                    if pressed {
                        if button == MouseButton::Middle && self.modal.handle_trigger(InputTrigger::WheelClick, self.avatar.as_mut()) {
                            self.modal.sync_controls(self.controls.as_mut());
                        } else {
                            let directive = self.modal.press_mouse(self.surface.as_ref(), self.avatar.as_mut());
                            self.modal.sync_controls(self.controls.as_mut());
                            if matches!(directive, ModalDirective::ExitGame) { event_loop.exit(); }
                        }
                    } else {
                        self.modal.release_mouse();
                    }
                } else if button == MouseButton::Middle {
                    if pressed { self.surface.capture_cursor(); }
                    self.dispatch_gameplay_trigger(InputTrigger::WheelClick, pressed);
                } else if pressed {
                    self.surface.capture_cursor();
                    match button {
                        MouseButton::Left => { let _ = self.avatar.use_left_hand(self.world.as_mut()); self.refresh_modified_face(); }
                        MouseButton::Right => { let _ = self.avatar.use_right_hand(self.world.as_mut()); self.refresh_modified_face(); }
                        _ => {}
                    }
                } else if matches!(button, MouseButton::Left | MouseButton::Right) && self.avatar.release_hand(self.world.as_mut()) {
                    self.refresh_modified_face();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                let scroll = if dy.abs() >= dx.abs() { dy } else { dx };
                if scroll.abs() <= 1e-5 { return; }
                let trigger = if scroll > 0.0 { InputTrigger::WheelUp } else { InputTrigger::WheelDown };
                if self.modal.open() {
                    if self.modal.handle_trigger(trigger, self.avatar.as_mut()) {
                        self.modal.sync_controls(self.controls.as_mut());
                    }
                } else {
                    self.dispatch_gameplay_trigger(trigger, true);
                    self.dispatch_gameplay_trigger(trigger, false);
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(code), state, .. },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                if code == KeyCode::Escape {
                    if pressed && (self.modal.open() || !self.avatar.dispatch_trigger(InputTrigger::Key(code))) {
                        self.modal.toggle_escape(self.surface.as_ref());
                    }
                    return;
                }
                if self.modal.open() {
                    if pressed && self.modal.handle_key(code, self.avatar.as_mut()) {
                        self.modal.sync_controls(self.controls.as_mut());
                    }
                    return;
                }
                self.dispatch_gameplay_trigger(InputTrigger::Key(code), pressed);
            }
            WindowEvent::RedrawRequested => {
                let dt = 1.0 / 60.0;
                if self.modal.open() {
                    self.modal.step(dt);
                } else {
                    self.avatar.step_motion(dt, self.controls.as_ref(), self.world.as_ref());
                    let dug = self.avatar.step_digging(self.world.as_mut());
                    let regrown = self.world.step_surface(dt);
                    if dug || regrown { self.refresh_modified_face(); }
                }

                if self.tron_active {
                    let mut avatar_tron = CpuTronBatch::empty();
                    self.avatar.emit_appearance(&mut avatar_tron);
                    let composed = CompositeMesh::new(&self.tron_mesh, &avatar_tron);
                    self.surface.render_frame(
                        self.tron_pipeline.as_ref(), self.tron_shader.as_ref(), &composed,
                        self.avatar.as_ref(), self.primary_shader.as_ref(), self.modal.as_ref(),
                    );
                } else {
                    let mut avatar_batch = CpuMeshBatch::empty();
                    self.avatar.emit_appearance(&mut avatar_batch);
                    let composed = CompositeMesh::new(&self.primary_mesh, &avatar_batch);
                    self.surface.render_frame(
                        self.primary_pipeline.as_ref(), self.primary_shader.as_ref(), &composed,
                        self.avatar.as_ref(), self.primary_shader.as_ref(), self.modal.as_ref(),
                    );
                }
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        if !self.modal.open() {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                self.avatar.rotate_view(dx as f32, dy as f32);
            }
        }
    }
}
