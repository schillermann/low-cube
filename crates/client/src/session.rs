use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

use world::PlanetaryWorld;
use engine::{CompositeMesh, CpuMeshBatch, CpuTronBatch, GraphicPipeline, Mesh, ShaderProgram};
use crate::avatar::Avatar;
use crate::controls::KeyboardControls;
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
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                self.surface.capture_cursor();
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
                match code {
                    KeyCode::KeyI => {
                        if pressed {
                            self.avatar.toggle_invert_y();
                        }
                    }
                    KeyCode::KeyO => {
                        if pressed {
                            self.avatar.toggle_invert_x();
                        }
                    }
                    KeyCode::KeyT => {
                        if pressed {
                            self.tron_active = !self.tron_active;
                        }
                    }
                    KeyCode::Escape => event_loop.exit(),
                    _ => {
                        if !pressed || !self.avatar.dispatch_wardrobe_key(code) {
                            self.controls.register_key(code, pressed);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = 1.0 / 60.0;
                self.avatar.step_motion(dt, self.controls.as_ref(), self.world.as_ref());

                if self.tron_active {
                    let mut avatar_tron = CpuTronBatch::empty();
                    self.avatar.emit_appearance(&mut avatar_tron);
                    let composed = CompositeMesh::new(self.tron_mesh.as_ref(), &avatar_tron);
                    self.surface.render_frame(
                        self.tron_pipeline.as_ref(),
                        self.tron_shader.as_ref(),
                        &composed,
                        self.avatar.as_ref(),
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
                    );
                }
            }

            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.avatar.rotate_view(dx as f32, dy as f32);
        }
    }
}
