use std::num::NonZeroU32;
use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use world::{CubePlanet, GravityField, PlanetaryWorld};
use engine_render::{CpuMeshBuffer, GlMesh, VoxelShader};

struct LowCubeApp {
    gl_context: Option<PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    window: Option<Window>,
    gl: Option<glow::Context>,
    shader: Option<VoxelShader>,
    mesh: Option<GlMesh>,
    planet: CubePlanet,
    // Player state
    player_pos: Vec3,
    player_up: Vec3,
    yaw: f32,
    pitch: f32,
    key_w: bool,
    key_s: bool,
    key_a: bool,
    key_d: bool,
    invert_y: bool,
    invert_x: bool,
}

impl LowCubeApp {
    fn new() -> Self {
        Self {
            gl_context: None,
            gl_surface: None,
            window: None,
            gl: None,
            shader: None,
            mesh: None,
            planet: CubePlanet::new(16),
            player_pos: Vec3::new(0.0, 9.0, 0.0),
            player_up: Vec3::Y,
            yaw: 0.0,
            pitch: 0.0,
            key_w: false,
            key_s: false,
            key_a: false,
            key_d: false,
            invert_y: false,
            invert_x: false,
        }
    }
}

impl ApplicationHandler for LowCubeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = WindowAttributes::default()
            .with_title("Low-Cube [WASD: Move, Mouse: Look, I: Invert Y, O: Invert X, Esc: Exit]")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .expect("Failed to create window and GL config");

        let window = window.expect("Failed to get window handle");

        let raw_window_handle = window.window_handle().ok().map(|h| h.as_raw());

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3))))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create OpenGL context")
        };

        let attrs = window.build_surface_attributes(Default::default()).expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create window surface")
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .expect("Failed to make context current");

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|s| {
                gl_config.display().get_proc_address(s)
            })
        };

        let shader = unsafe { VoxelShader::compile(&gl) };

        // Generate geometry for the 16x16x16 cube planet
        let mut cpu_buf = CpuMeshBuffer::new();
        self.planet.emit_surface(&mut cpu_buf);
        let mesh = unsafe { GlMesh::from_cpu_buffer(&gl, &cpu_buf) };

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            gl.clear_color(0.04, 0.05, 0.08, 1.0); // Deep space background
        }

        self.window = Some(window);
        self.gl_context = Some(gl_context);
        self.gl_surface = Some(gl_surface);
        self.gl = Some(gl);
        self.shader = Some(shader);
        self.mesh = Some(mesh);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let (Some(surface), Some(context), Some(gl)) =
                    (&self.gl_surface, &self.gl_context, &self.gl)
                {
                    if size.width > 0 && size.height > 0 {
                        surface.resize(
                            context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                        unsafe {
                            gl.viewport(0, 0, size.width as i32, size.height as i32);
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                if let Some(window) = &self.window {
                    let _ = window
                        .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                        .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
                    window.set_cursor_visible(false);
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
                match code {
                    KeyCode::KeyW => self.key_w = pressed,
                    KeyCode::KeyS => self.key_s = pressed,
                    KeyCode::KeyA => self.key_a = pressed,
                    KeyCode::KeyD => self.key_d = pressed,
                    KeyCode::KeyI => {
                        if pressed {
                            self.invert_y = !self.invert_y;
                            println!("[Controls] Invert Y (Pitch) is now: {}", self.invert_y);
                        }
                    }
                    KeyCode::KeyO => {
                        if pressed {
                            self.invert_x = !self.invert_x;
                            println!("[Controls] Invert X (Yaw) is now: {}", self.invert_x);
                        }
                    }
                    KeyCode::Escape => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                // Update player physics and gravity
                let dt = 1.0 / 60.0;
                let target_up = self.planet.surface_up_at(self.player_pos);

                // Smoothly slerp current up towards target up
                let rot_to_target = Quat::from_rotation_arc(self.player_up, target_up);
                self.player_up = rot_to_target.slerp(Quat::IDENTITY, 0.85) * self.player_up;
                if self.player_up.dot(target_up) > 0.999 {
                    self.player_up = target_up;
                }

                // Compute orientation basis on tangent plane
                let up = self.player_up.normalize();
                let arbitrary_ref = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
                let tangent = up.cross(arbitrary_ref).normalize();
                let bitangent = tangent.cross(up).normalize();

                // Build surface rotation
                let surface_rot = Quat::from_mat3(&glam::Mat3::from_cols(tangent, up, bitangent));
                let view_rot = surface_rot * Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);

                let forward = view_rot * Vec3::NEG_Z;
                let right = view_rot * Vec3::X;

                // Tangent movement
                let mut move_vec = Vec3::ZERO;
                if self.key_w { move_vec += forward; }
                if self.key_s { move_vec -= forward; }
                if self.key_d { move_vec += right; }
                if self.key_a { move_vec -= right; }

                // Project move_vec onto tangent plane perpendicular to up
                let tangent_move = move_vec - up * move_vec.dot(up);
                if tangent_move.length_squared() > 1e-4 {
                    self.player_pos += tangent_move.normalize() * (6.0 * dt);
                }

                // Constrain player position to world surface (player height = 1.0)
                self.player_pos = self.planet.constrain_to_surface(self.player_pos, 1.0);

                // Render frame
                if let (Some(window), Some(context), Some(surface), Some(gl), Some(shader), Some(mesh)) = (
                    &self.window,
                    &self.gl_context,
                    &self.gl_surface,
                    &self.gl,
                    &self.shader,
                    &self.mesh,
                ) {
                    let win_size = window.inner_size();
                    let aspect = (win_size.width as f32) / (win_size.height.max(1) as f32);

                    let proj = Mat4::perspective_rh(65.0f32.to_radians(), aspect, 0.1, 200.0);
                    let eye = self.player_pos + up * 0.8;
                    let target = eye + forward;
                    let view = Mat4::look_at_rh(eye, target, up);
                    let view_proj = proj * view;

                    unsafe {
                        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                        let light_dir = Vec3::new(0.6, 0.8, 0.4).normalize();
                        shader.bind(gl, &view_proj, light_dir);
                        mesh.draw(gl);
                    }

                    surface.swap_buffers(context).expect("Swap buffers failed");
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            let sensitivity = 0.003;
            let x_mult = if self.invert_x { 1.0 } else { -1.0 };
            let y_mult = if self.invert_y { 1.0 } else { -1.0 };

            self.yaw += (dx as f32) * sensitivity * x_mult;
            self.pitch = (self.pitch + (dy as f32) * sensitivity * y_mult).clamp(-1.4, 1.4);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = LowCubeApp::new();
    event_loop.run_app(&mut app).expect("Application run failed");
}
