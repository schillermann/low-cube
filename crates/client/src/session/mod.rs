mod display;
mod stage;

use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

use world::PlanetaryWorld;
use engine::{GraphicPipeline, PlanetaryMesh, ShaderProgram};
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::menu::{ModalDirective, SessionModal};
use crate::surface::PlatformSurface;
use self::display::{FramePace, RenderPass, SessionDisplay};
use self::stage::SessionStage;

/// Interface representing an active, interactive display session.
pub trait Session {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
    fn handle_device_event(&mut self, event: DeviceEvent);
}

/// Fully initialized active game session with code-free primary constructor.
pub struct GameSession {
    display: SessionDisplay,
    stage: SessionStage,
    pace: FramePace,
}

impl GameSession {
    /// Primary constructor doing zero computation or I/O.
    fn with_pace(display: SessionDisplay, stage: SessionStage, pace: FramePace) -> Self {
        Self { display, stage, pace }
    }

    /// Secondary constructor assembling display and stage passes with real-time frame pacing.
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
        Self::with_pace(
            SessionDisplay::new(
                surface,
                RenderPass::new(primary_pipeline, primary_shader, primary_mesh),
                RenderPass::new(tron_pipeline, tron_shader, tron_mesh),
                tron_active,
            ),
            SessionStage::new(world, avatar, controls, modal),
            FramePace::realtime(),
        )
    }

    fn dispatch_gameplay_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        if self.stage.matches_tron(trigger) {
            if pressed {
                self.display.toggle_tron(self.stage.world(), self.stage.avatar());
            }
        } else if self.stage.dispatch_avatar_or_control(trigger, pressed) {
            self.display.refresh_modified_and_meadow(&mut self.stage);
        }
    }
}

impl Session for GameSession {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.display.resize(size.width, size.height),
            WindowEvent::CursorMoved { position, .. } => {
                let point = self.display.surface().normalize_cursor(position.x, position.y);
                self.stage.move_cursor(point);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                if self.stage.modal_open() {
                    if pressed {
                        if button == MouseButton::Middle {
                            self.stage.handle_modal_trigger(InputTrigger::WheelClick);
                        } else if matches!(self.stage.press_modal_mouse(self.display.surface()), ModalDirective::ExitGame) {
                            event_loop.exit();
                        }
                    } else {
                        self.stage.release_modal_mouse();
                    }
                } else if button == MouseButton::Middle {
                    if pressed {
                        self.display.capture_cursor();
                    }
                    self.dispatch_gameplay_trigger(InputTrigger::WheelClick, pressed);
                } else if pressed {
                    self.display.capture_cursor();
                    match button {
                        MouseButton::Left => {
                            let _ = self.stage.use_left();
                            self.display.refresh_modified_and_meadow(&mut self.stage);
                        }
                        MouseButton::Right => {
                            let _ = self.stage.use_right();
                            self.display.refresh_modified_and_meadow(&mut self.stage);
                        }
                        _ => {}
                    }
                } else if matches!(button, MouseButton::Left | MouseButton::Right) && self.stage.release_hand() {
                    self.display.refresh_modified_and_meadow(&mut self.stage);
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
                if self.stage.modal_open() {
                    self.stage.handle_modal_trigger(trigger);
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
                    if pressed {
                        self.stage.toggle_escape(self.display.surface());
                    }
                    return;
                }
                if self.stage.modal_open() {
                    if pressed {
                        self.stage.handle_modal_key(code);
                    }
                    return;
                }
                self.dispatch_gameplay_trigger(InputTrigger::Key(code), pressed);
            }
            WindowEvent::RedrawRequested => {
                let dt = self.pace.advance();
                self.stage.step_frame(dt, &mut self.display);
                self.display.render_stage(&mut self.stage);
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        if !self.stage.modal_open() {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                self.stage.rotate_avatar(dx as f32, dy as f32);
            }
        }
    }
}
