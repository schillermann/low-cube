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

struct RenderPass {
    pipeline: Box<dyn GraphicPipeline>,
    shader: Box<dyn ShaderProgram>,
    mesh: PlanetaryMesh,
}

struct SessionDisplay {
    surface: Box<dyn PlatformSurface>,
    primary: RenderPass,
    tron: RenderPass,
    tron_active: bool,
}

impl SessionDisplay {
    fn refresh_face_slice(&mut self, world: &dyn PlanetaryWorld, idx: usize) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            world.emit_face(idx, &mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron.mesh.slice_at(idx), &tron_buf);
            self.tron.mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::empty();
            world.emit_face(idx, &mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(idx), &cpu_buf);
            self.primary.mesh.store_slice(idx, next);
        }
    }

    fn refresh_modified_face(&mut self, world: &mut dyn PlanetaryWorld) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            let idx = world.emit_modified_face(&mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron.mesh.slice_at(idx), &tron_buf);
            self.tron.mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::empty();
            let idx = world.emit_modified_face(&mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(idx), &cpu_buf);
            self.primary.mesh.store_slice(idx, next);
        }
    }

    fn refresh_world_mesh(&mut self, world: &dyn PlanetaryWorld) {
        for idx in 0..(world.face_count() * 2) {
            self.refresh_face_slice(world, idx);
        }
    }

    fn toggle_tron(&mut self, world: &dyn PlanetaryWorld) {
        self.tron_active = !self.tron_active;
        self.refresh_world_mesh(world);
    }

    fn render_stage(&self, stage: &SessionStage) {
        if self.tron_active {
            let mut avatar_tron = CpuTronBatch::empty();
            stage.avatar.emit_appearance(&mut avatar_tron);
            let composed = CompositeMesh::new(&self.tron.mesh, &avatar_tron);
            self.surface.render_frame(
                self.tron.pipeline.as_ref(),
                self.tron.shader.as_ref(),
                &composed,
                stage.avatar.as_ref(),
                self.primary.shader.as_ref(),
                stage.modal.as_ref(),
            );
        } else {
            let mut avatar_batch = CpuMeshBatch::empty();
            stage.avatar.emit_appearance(&mut avatar_batch);
            let composed = CompositeMesh::new(&self.primary.mesh, &avatar_batch);
            self.surface.render_frame(
                self.primary.pipeline.as_ref(),
                self.primary.shader.as_ref(),
                &composed,
                stage.avatar.as_ref(),
                self.primary.shader.as_ref(),
                stage.modal.as_ref(),
            );
        }
    }
}

struct SessionStage {
    world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>,
    controls: Box<dyn KeyboardControls>,
    modal: Box<dyn SessionModal>,
}

/// Fully initialized active game session with code-free primary constructor.
pub struct GameSession {
    display: SessionDisplay,
    stage: SessionStage,
}

impl GameSession {
    /// Primary constructor doing zero computation or I/O.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface: Box<dyn PlatformSurface>,
        primary_pipeline: Box<dyn GraphicPipeline>,
        primary_shader: Box<dyn ShaderProgram>,
        primary_mesh: PlanetaryMesh,
        tron_pipeline: Box<dyn GraphicPipeline>,
        tron_shader: Box<dyn ShaderProgram>,
        tron_mesh: PlanetaryMesh,
        tron_active: bool,
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
        modal: Box<dyn SessionModal>,
    ) -> Self {
        Self {
            display: SessionDisplay {
                surface,
                primary: RenderPass {
                    pipeline: primary_pipeline,
                    shader: primary_shader,
                    mesh: primary_mesh,
                },
                tron: RenderPass {
                    pipeline: tron_pipeline,
                    shader: tron_shader,
                    mesh: tron_mesh,
                },
                tron_active,
            },
            stage: SessionStage {
                world,
                avatar,
                controls,
                modal,
            },
        }
    }

    fn dispatch_gameplay_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        if self.stage.controls.matches_action(ControlAction::TronMode, trigger) {
            if pressed {
                self.display.toggle_tron(self.stage.world.as_ref());
            }
        } else if !pressed || !self.stage.avatar.dispatch_trigger(trigger) {
            self.stage.controls.register_trigger(trigger, pressed);
        }
    }
}

impl Session for GameSession {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.display.surface.resize(size.width, size.height),
            WindowEvent::CursorMoved { position, .. } => {
                let point = self.display.surface.normalize_cursor(position.x, position.y);
                self.stage.modal.move_cursor(point);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                if self.stage.modal.open() {
                    if pressed {
                        if button == MouseButton::Middle
                            && self.stage.modal.handle_trigger(InputTrigger::WheelClick, self.stage.avatar.as_mut())
                        {
                            self.stage.modal.sync_controls(self.stage.controls.as_mut());
                        } else {
                            let directive = self.stage.modal.press_mouse(self.display.surface.as_ref(), self.stage.avatar.as_mut());
                            self.stage.modal.sync_controls(self.stage.controls.as_mut());
                            if matches!(directive, ModalDirective::ExitGame) {
                                event_loop.exit();
                            }
                        }
                    } else {
                        self.stage.modal.release_mouse();
                    }
                } else if button == MouseButton::Middle {
                    if pressed {
                        self.display.surface.capture_cursor();
                    }
                    self.dispatch_gameplay_trigger(InputTrigger::WheelClick, pressed);
                } else if pressed {
                    self.display.surface.capture_cursor();
                    match button {
                        MouseButton::Left => {
                            let _ = self.stage.avatar.use_left_hand(self.stage.world.as_mut());
                            self.display.refresh_modified_face(self.stage.world.as_mut());
                        }
                        MouseButton::Right => {
                            let _ = self.stage.avatar.use_right_hand(self.stage.world.as_mut());
                            self.display.refresh_modified_face(self.stage.world.as_mut());
                        }
                        _ => {}
                    }
                } else if matches!(button, MouseButton::Left | MouseButton::Right)
                    && self.stage.avatar.release_hand(self.stage.world.as_mut())
                {
                    self.display.refresh_modified_face(self.stage.world.as_mut());
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                let scroll = if dy.abs() >= dx.abs() { dy } else { dx };
                if scroll.abs() <= 1e-5 {
                    return;
                }
                let trigger = if scroll > 0.0 { InputTrigger::WheelUp } else { InputTrigger::WheelDown };
                if self.stage.modal.open() {
                    if self.stage.modal.handle_trigger(trigger, self.stage.avatar.as_mut()) {
                        self.stage.modal.sync_controls(self.stage.controls.as_mut());
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
                    if pressed && (self.stage.modal.open() || !self.stage.avatar.dispatch_trigger(InputTrigger::Key(code))) {
                        self.stage.modal.toggle_escape(self.display.surface.as_ref());
                    }
                    return;
                }
                if self.stage.modal.open() {
                    if pressed && self.stage.modal.handle_key(code, self.stage.avatar.as_mut()) {
                        self.stage.modal.sync_controls(self.stage.controls.as_mut());
                    }
                    return;
                }
                self.dispatch_gameplay_trigger(InputTrigger::Key(code), pressed);
            }
            WindowEvent::RedrawRequested => {
                let dt = 1.0 / 60.0;
                if self.stage.modal.open() {
                    self.stage.modal.step(dt);
                } else {
                    self.stage.avatar.step_motion(dt, self.stage.controls.as_ref(), self.stage.world.as_ref());
                    let dug = self.stage.avatar.step_digging(self.stage.world.as_mut());
                    let regrown = self.stage.world.step_surface(dt);
                    if dug || regrown {
                        self.display.refresh_modified_face(self.stage.world.as_mut());
                    }
                }

                self.display.render_stage(&self.stage);
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        if !self.stage.modal.open() {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                self.stage.avatar.rotate_view(dx as f32, dy as f32);
            }
        }
    }
}
