use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

use world::PlanetaryWorld;
use engine::{GraphicPipeline, Mesh, ShaderProgram};
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
    pipeline: Box<dyn GraphicPipeline>,
    shader: Box<dyn ShaderProgram>,
    mesh: Box<dyn Mesh>,
    world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>,
    controls: Box<dyn KeyboardControls>,
}

impl GameSession {
    /// Primary constructor doing zero computation or I/O.
    pub fn new(
        surface: Box<dyn PlatformSurface>,
        pipeline: Box<dyn GraphicPipeline>,
        shader: Box<dyn ShaderProgram>,
        mesh: Box<dyn Mesh>,
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
    ) -> Self {
        Self {
            surface,
            pipeline,
            shader,
            mesh,
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
                    KeyCode::Escape => event_loop.exit(),
                    _ => self.controls.register_key(code, pressed),
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = 1.0 / 60.0;
                self.avatar.step_motion(dt, self.controls.as_ref(), self.world.as_ref());

                self.surface.render_frame(
                    self.pipeline.as_ref(),
                    self.shader.as_ref(),
                    self.mesh.as_ref(),
                    self.avatar.as_ref(),
                );
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
