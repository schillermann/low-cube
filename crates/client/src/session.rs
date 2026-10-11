use std::num::NonZeroU32;
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use world::PlanetaryWorld;
use engine_render::{GraphicPipeline, Mesh, ShaderProgram};
use crate::avatar::Avatar;
use crate::controls::KeyboardControls;

/// Fully initialized active game session with code-free primary constructor.
pub struct GameSession {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    gl: glow::Context,
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
        window: Window,
        gl_context: PossiblyCurrentContext,
        gl_surface: Surface<WindowSurface>,
        gl: glow::Context,
        pipeline: Box<dyn GraphicPipeline>,
        shader: Box<dyn ShaderProgram>,
        mesh: Box<dyn Mesh>,
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
    ) -> Self {
        Self {
            window,
            gl_context,
            gl_surface,
            gl,
            pipeline,
            shader,
            mesh,
            world,
            avatar,
            controls,
        }
    }

    pub fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.gl_surface.resize(
                        &self.gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                    unsafe {
                        use glow::HasContext;
                        self.gl.viewport(0, 0, size.width as i32, size.height as i32);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                let _ = self.window
                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                    .or_else(|_| self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
                self.window.set_cursor_visible(false);
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
                    KeyCode::KeyW | KeyCode::KeyS | KeyCode::KeyA | KeyCode::KeyD => {
                        self.controls.set_key(code, pressed);
                    }
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
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = 1.0 / 60.0;
                self.avatar.step_motion(dt, self.controls.as_ref(), self.world.as_ref());

                let win_size = self.window.inner_size();
                let aspect = (win_size.width as f32) / (win_size.height.max(1) as f32);
                let view_proj = self.avatar.view_projection_matrix(aspect);

                self.pipeline.render(
                    &self.gl,
                    self.shader.as_ref(),
                    self.mesh.as_ref(),
                    &view_proj,
                );

                self.gl_surface.swap_buffers(&self.gl_context).expect("Buffer swap failed");
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.avatar.rotate_view(dx as f32, dy as f32);
        }
    }
}
