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

use world::{CubePlanet, PlanetaryWorld};
use engine_render::{CpuMeshBuffer, ForwardRenderer, GlMesh, Mesh, SceneRenderer, ShaderProgram, VoxelShader};

/// Interface representing player keyboard navigation state.
pub trait KeyboardControls {
    fn set_key(&mut self, code: KeyCode, pressed: bool);
    fn resolve_motion(&self, forward: Vec3, right: Vec3) -> Vec3;
}

/// Encapsulated keyboard state implementing KeyboardControls.
pub struct KeyState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }
}

impl KeyboardControls for KeyState {
    fn set_key(&mut self, code: KeyCode, pressed: bool) {
        match code {
            KeyCode::KeyW => self.forward = pressed,
            KeyCode::KeyS => self.backward = pressed,
            KeyCode::KeyA => self.left = pressed,
            KeyCode::KeyD => self.right = pressed,
            _ => {}
        }
    }

    fn resolve_motion(&self, forward: Vec3, right: Vec3) -> Vec3 {
        let mut move_vec = Vec3::ZERO;
        if self.forward { move_vec += forward; }
        if self.backward { move_vec -= forward; }
        if self.right { move_vec += right; }
        if self.left { move_vec -= right; }
        move_vec
    }
}

/// Interface for player avatars acting within planetary gravity.
pub trait Player {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn toggle_invert_y(&mut self);
    fn toggle_invert_x(&mut self);
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
}

/// Fully encapsulated player entity implementing Player.
pub struct PlayerAvatar {
    position: Vec3,
    surface_up: Vec3,
    yaw: f32,
    pitch: f32,
    invert_y: bool,
    invert_x: bool,
}

impl PlayerAvatar {
    pub fn new(initial_position: Vec3) -> Self {
        Self {
            position: initial_position,
            surface_up: Vec3::Y,
            yaw: 0.0,
            pitch: 0.0,
            invert_y: false,
            invert_x: false,
        }
    }

    fn orientation_basis(&self) -> (Vec3, Vec3, Vec3, Quat) {
        let up = self.surface_up.normalize();
        let arbitrary_ref = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let tangent = up.cross(arbitrary_ref).normalize();
        let bitangent = tangent.cross(up).normalize();

        let surface_rot = Quat::from_mat3(&glam::Mat3::from_cols(tangent, up, bitangent));
        let view_rot = surface_rot * Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);

        let forward = view_rot * Vec3::NEG_Z;
        let right = view_rot * Vec3::X;

        (forward, right, up, view_rot)
    }
}

impl Player for PlayerAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.003;
        let x_sign = if self.invert_x { 1.0 } else { -1.0 };
        let y_sign = if self.invert_y { 1.0 } else { -1.0 };

        self.yaw += dx * sensitivity * x_sign;
        self.pitch = (self.pitch + dy * sensitivity * y_sign).clamp(-1.4, 1.4);
    }

    fn toggle_invert_y(&mut self) {
        self.invert_y = !self.invert_y;
        println!("[Player] Invert Y (Pitch) is now: {}", self.invert_y);
    }

    fn toggle_invert_x(&mut self) {
        self.invert_x = !self.invert_x;
        println!("[Player] Invert X (Yaw) is now: {}", self.invert_x);
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        let target_up = world.surface_up_at(self.position);

        let rot_to_target = Quat::from_rotation_arc(self.surface_up, target_up);
        self.surface_up = rot_to_target.slerp(Quat::IDENTITY, 0.85) * self.surface_up;
        if self.surface_up.dot(target_up) > 0.999 {
            self.surface_up = target_up;
        }

        let (forward, right, up, _) = self.orientation_basis();
        let raw_move = controls.resolve_motion(forward, right);

        let tangent_move = raw_move - up * raw_move.dot(up);
        if tangent_move.length_squared() > 1e-4 {
            self.position += tangent_move.normalize() * (6.0 * dt);
        }

        self.position = world.constrain_to_surface(self.position, 1.0);
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let (forward, _, up, _) = self.orientation_basis();
        let proj = Mat4::perspective_rh(65.0f32.to_radians(), aspect, 0.1, 200.0);
        let eye = self.position + up * 0.8;
        let target = eye + forward;
        let view = Mat4::look_at_rh(eye, target, up);
        proj * view
    }
}

/// Fully initialized, non-null active game session.
pub struct GameSession {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    gl: glow::Context,
    renderer: Box<dyn SceneRenderer>,
    shader: Box<dyn ShaderProgram>,
    mesh: Box<dyn Mesh>,
    world: Box<dyn PlanetaryWorld>,
    player: Box<dyn Player>,
    controls: Box<dyn KeyboardControls>,
}

impl GameSession {
    pub fn build(event_loop: &ActiveEventLoop) -> Self {
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
            .expect("Failed to build GL config");

        let window = window.expect("Failed to get window");
        let raw_window_handle = window.window_handle().ok().map(|h| h.as_raw());

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3))))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create context")
        };

        let attrs = window.build_surface_attributes(Default::default()).expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create surface")
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .expect("Failed to make context current");

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|s| {
                gl_config.display().get_proc_address(s)
            })
        };

        let planet = Box::new(CubePlanet::new(16));

        let mut cpu_buf = CpuMeshBuffer::new();
        planet.emit_surface(&mut cpu_buf);
        let mesh = unsafe { Box::new(GlMesh::upload(&gl, &cpu_buf)) };
        let shader = unsafe { Box::new(VoxelShader::compile(&gl)) };
        let renderer = Box::new(ForwardRenderer::new());

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            gl.clear_color(0.04, 0.05, 0.08, 1.0);
        }

        let _ = window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
        window.set_cursor_visible(false);

        let player = Box::new(PlayerAvatar::new(Vec3::new(0.0, 9.0, 0.0)));
        let controls = Box::new(KeyState::new());

        Self {
            window,
            gl_context,
            gl_surface,
            gl,
            renderer,
            shader,
            mesh,
            world: planet,
            player,
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
                            self.player.toggle_invert_y();
                        }
                    }
                    KeyCode::KeyO => {
                        if pressed {
                            self.player.toggle_invert_x();
                        }
                    }
                    KeyCode::Escape => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = 1.0 / 60.0;
                self.player.step_motion(dt, self.controls.as_ref(), self.world.as_ref());

                let win_size = self.window.inner_size();
                let aspect = (win_size.width as f32) / (win_size.height.max(1) as f32);
                let view_proj = self.player.view_projection_matrix(aspect);
                let light_dir = Vec3::new(0.6, 0.8, 0.4).normalize();

                self.renderer.render(
                    &self.gl,
                    self.shader.as_ref(),
                    self.mesh.as_ref(),
                    &view_proj,
                    light_dir,
                );

                self.gl_surface.swap_buffers(&self.gl_context).expect("Buffer swap failed");
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.player.rotate_view(dx as f32, dy as f32);
        }
    }
}

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
            self.lifecycle = AppLifecycle::Active(GameSession::build(event_loop));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let AppLifecycle::Active(session) = &mut self.lifecycle {
            session.handle_window_event(event_loop, event);
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let AppLifecycle::Active(session) = &mut self.lifecycle {
            session.handle_device_event(event);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = LowCubeApp::new();
    event_loop.run_app(&mut app).expect("Application run failed");
}
