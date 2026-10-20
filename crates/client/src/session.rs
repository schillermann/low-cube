use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

use world::PlanetaryWorld;
use engine::{CompositeMesh, CpuMeshBatch, CpuTronBatch, GraphicPipeline, Mesh, ShaderProgram};
use crate::avatar::Avatar;
use crate::controls::KeyboardControls;
use crate::menu::{ModalDirective, SessionModal};
use crate::surface::PlatformSurface;

/// Interface representing an active, interactive display session.
pub trait Session {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
    fn handle_device_event(&mut self, event: DeviceEvent);
}

/// Fully initialized active game session with code-free primary constructor.
pub struct GameSession {
    surface: Box<dyn PlatformSurface>,
    primary_pipeline: Box<dyn GraphicPipeline>,
    primary_shader: Box<dyn ShaderProgram>,
    primary_mesh: Box<dyn Mesh>,
    tron_pipeline: Box<dyn GraphicPipeline>,
    tron_shader: Box<dyn ShaderProgram>,
    tron_mesh: Box<dyn Mesh>,
    tron_active: bool,
    world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>,
    controls: Box<dyn KeyboardControls>,
    modal: Box<dyn SessionModal>,
}

impl GameSession {
    /// Primary constructor doing zero computation or I/O.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface: Box<dyn PlatformSurface>,
        primary_pipeline: Box<dyn GraphicPipeline>,
        primary_shader: Box<dyn ShaderProgram>,
        primary_mesh: Box<dyn Mesh>,
        tron_pipeline: Box<dyn GraphicPipeline>,
        tron_shader: Box<dyn ShaderProgram>,
        tron_mesh: Box<dyn Mesh>,
        tron_active: bool,
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
        modal: Box<dyn SessionModal>,
    ) -> Self {
        Self {
            surface,
            primary_pipeline,
            primary_shader,
            primary_mesh,
            tron_pipeline,
            tron_shader,
            tron_mesh,
            tron_active,
            world,
            avatar,
            controls,
            modal,
        }
    }
}

impl Session for GameSession {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.surface.resize(size.width, size.height);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = self.surface.normalize_cursor(position.x, position.y);
                self.modal.move_cursor(point);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if self.modal.open() {
                    if state == ElementState::Pressed {
                        let directive = self.modal.press_mouse(self.surface.as_ref(), self.avatar.as_mut());
                        if matches!(directive, ModalDirective::ExitGame) {
                            event_loop.exit();
                        }
                    } else {
                        self.modal.release_mouse();
                    }
                } else if state == ElementState::Pressed {
                    self.surface.capture_cursor();
                    match button {
                        MouseButton::Left => self.avatar.trigger_left_strike(),
                        MouseButton::Right => self.avatar.trigger_right_strike(),
                        _ => {}
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state,
                    ..
                },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                if code == KeyCode::Escape {
                    if pressed {
                        self.modal.toggle_escape(self.surface.as_ref());
                    }
                    return;
                }
                if self.modal.open() {
                    if pressed {
                        self.modal.handle_key(code, self.avatar.as_mut());
                    }
                    return;
                }
                match code {
                    KeyCode::KeyI => if pressed { self.avatar.toggle_invert_y(); },
                    KeyCode::KeyO => if pressed { self.avatar.toggle_invert_x(); },
                    KeyCode::KeyT => if pressed { self.tron_active = !self.tron_active; },
                    _ => {
                        if !pressed || !self.avatar.dispatch_wardrobe_key(code) {
                            self.controls.register_key(code, pressed);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = 1.0 / 60.0;
                if self.modal.open() {
                    self.modal.step(dt);
                } else {
                    self.avatar.step_motion(dt, self.controls.as_ref(), self.world.as_ref());
                }

                if self.tron_active {
                    let mut avatar_tron = CpuTronBatch::empty();
                    self.avatar.emit_appearance(&mut avatar_tron);
                    let composed = CompositeMesh::new(self.tron_mesh.as_ref(), &avatar_tron);
                    self.surface.render_frame(
                        self.tron_pipeline.as_ref(),
                        self.tron_shader.as_ref(),
                        &composed,
                        self.avatar.as_ref(),
                        self.primary_shader.as_ref(),
                        self.modal.as_ref(),
                    );
                } else {
                    let mut avatar_batch = CpuMeshBatch::empty();
                    self.avatar.emit_appearance(&mut avatar_batch);
                    let composed = CompositeMesh::new(self.primary_mesh.as_ref(), &avatar_batch);
                    self.surface.render_frame(
                        self.primary_pipeline.as_ref(),
                        self.primary_shader.as_ref(),
                        &composed,
                        self.avatar.as_ref(),
                        self.primary_shader.as_ref(),
                        self.modal.as_ref(),
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
