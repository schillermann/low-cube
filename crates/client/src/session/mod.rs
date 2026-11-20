mod display;
#[cfg(test)]
mod headless;
mod overlay;
mod pulse;
#[cfg(test)]
mod pulse_tests;
mod stage;

use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::controls::InputTrigger;
use crate::menu::ModalDirective;
pub use self::display::{PolaroidScenePass, SessionDisplay, ShadedScenePass};
#[cfg(test)]
pub use self::headless::{HeadlessSession, HeadlessSurface};
use self::pulse::EpochPulse;
pub use self::stage::SessionStage;

/// Interface representing an active, interactive display session.
pub trait Session {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
    fn handle_device_event(&mut self, event: DeviceEvent);
    fn pace_event_loop(&mut self, event_loop: &ActiveEventLoop);
}

/// Fully initialized active game session with code-free primary constructor.
pub struct GameSession {
    stage: SessionStage,
    display: SessionDisplay,
    pulse: EpochPulse,
}

impl GameSession {
    /// Primary constructor doing zero computation or I/O.
    fn with_pulse(stage: SessionStage, display: SessionDisplay, pulse: EpochPulse) -> Self {
        Self { stage, display, pulse }
    }

    /// Secondary constructor assembling stage, display, and unified monotonic epoch pulse.
    pub fn new(stage: SessionStage, display: SessionDisplay) -> Self {
        Self::with_pulse(stage, display, EpochPulse::realtime())
    }

    fn dispatch_gameplay_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        if self.stage.dispatch_avatar_or_control(trigger, pressed) {
            self.stage.refresh_surface(&mut self.display);
        }
    }
}

impl Session for GameSession {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(true) => {
                self.stage.focus_regained();
                self.pulse.mark_urgent();
            }
            WindowEvent::Resized(size) => {
                self.display.resize(size.width, size.height);
                self.pulse.mark_urgent();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = self.display.surface().normalize_cursor(position.x, position.y);
                self.stage.move_cursor(point);
                if self.stage.modal_open() {
                    self.pulse.mark_urgent();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.pulse.mark_urgent();
                let pressed = state == ElementState::Pressed;
                if self.stage.modal_open() {
                    if pressed {
                        if button == MouseButton::Middle {
                            self.stage.handle_modal_trigger(InputTrigger::WheelClick);
                        } else if matches!(self.stage.press_modal_mouse(&mut self.display), ModalDirective::ExitGame) {
                            event_loop.exit();
                        }
                    } else {
                        self.stage.release_modal_mouse();
                        let _ = self.stage.release_hand();
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
                            if self.stage.use_left() {
                                self.stage.refresh_surface(&mut self.display);
                            }
                        }
                        MouseButton::Right => {
                            if self.stage.use_right() {
                                self.stage.refresh_surface(&mut self.display);
                            }
                        }
                        _ => {}
                    }
                } else if matches!(button, MouseButton::Left | MouseButton::Right) && self.stage.release_hand() {
                    self.stage.refresh_surface(&mut self.display);
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
                self.pulse.mark_urgent();
                let trigger = if scroll > 0.0 { InputTrigger::WheelUp } else { InputTrigger::WheelDown };
                if self.stage.modal_open() {
                    self.stage.handle_modal_trigger(trigger);
                } else {
                    self.dispatch_gameplay_trigger(trigger, true);
                    self.dispatch_gameplay_trigger(trigger, false);
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                self.stage.update_ctrl(state.control_key() || state.super_key());
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(code), logical_key, state, text, .. },
                ..
            } => {
                self.pulse.mark_urgent();
                let pressed = state == ElementState::Pressed;
                if matches!(
                    code,
                    KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::SuperLeft | KeyCode::SuperRight
                ) {
                    self.stage.update_ctrl(pressed);
                }
                if code == KeyCode::Escape {
                    if pressed {
                        self.stage.toggle_escape(self.display.surface());
                    }
                    return;
                }
                if self.stage.modal_open() {
                    if pressed {
                        if matches!(self.stage.handle_modal_key(code, &mut self.display), ModalDirective::ExitGame) {
                            event_loop.exit();
                            return;
                        }
                        if !self.stage.ctrl_active() {
                            let typed = text.as_deref().or_else(|| match &logical_key {
                                winit::keyboard::Key::Character(s) => Some(s.as_str()),
                                _ => None,
                            });
                            if let Some(chars) = typed {
                                self.stage.handle_modal_text(chars);
                            }
                        }
                    } else {
                        self.stage.release_control_key(code);
                    }
                    return;
                }
                let was_typing_avatar = self.stage.typing_avatar_text();
                self.dispatch_gameplay_trigger(InputTrigger::Key(code), pressed);
                if pressed
                    && was_typing_avatar
                    && self.stage.typing_avatar_text()
                    && !self.stage.ctrl_active()
                    && !matches!(code, KeyCode::Backspace | KeyCode::Enter | KeyCode::NumpadEnter)
                {
                    let typed = text.as_deref().or_else(|| match &logical_key {
                        winit::keyboard::Key::Character(s) => Some(s.as_str()),
                        _ => None,
                    });
                    if let Some(chars) = typed {
                        let _ = self.stage.handle_avatar_text(chars);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let (work_started, beat) = self.pulse.begin_frame();
                self.stage.step_frame(beat, &mut self.display);
                self.stage.render_onto(&mut self.display, beat.sight_scale());
                self.pulse.complete_frame(work_started, &self.display);
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        if !self.stage.modal_open() {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                self.stage.rotate_avatar(dx as f32, dy as f32);
                self.pulse.mark_urgent();
            }
        }
    }

    fn pace_event_loop(&mut self, event_loop: &ActiveEventLoop) {
        self.pulse.pace_event_loop(self.stage.modal_open(), &self.display, event_loop);
    }
}
