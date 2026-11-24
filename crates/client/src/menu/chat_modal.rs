use glam::{Mat4, Vec3};
use glow::HasContext;
use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use world::{MeshTarget, PlanetaryWorld, ProximityUtterance, Quad};
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
use super::glyph::PixelStamp;
use super::portal::PortalInvite;
use super::widget::ScreenPoint;
use super::{ModalDirective, SessionModal};

const FEED_LINE_CAP: usize = 6;
const FEED_TTL_SECS: f32 = 12.0;
const DRAFT_CHAR_CAP: usize = 64;

/// Polymorphic proximity chat input state avoiding `Option<String>` fields.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ChatCompose {
    Idle,
    Composing { draft: String, swallow_open_char: bool },
}

/// `SessionModal` decorator providing local proximity text chat (`Enter` / `T`) without global server spam.
pub struct ProximityChatModal<M> {
    origin: M,
    compose: ChatCompose,
    feed: Vec<(String, f32)>,
    outbox: Vec<String>,
}

impl<M> ProximityChatModal<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(
        origin: M,
        compose: ChatCompose,
        feed: Vec<(String, f32)>,
        outbox: Vec<String>,
    ) -> Self {
        Self { origin, compose, feed, outbox }
    }

    pub const fn wrapping(origin: M) -> Self {
        Self::new(origin, ChatCompose::Idle, Vec::new(), Vec::new())
    }

    const fn composing(&self) -> bool {
        matches!(self.compose, ChatCompose::Composing { .. })
    }

    fn append_feed(&mut self, line: String) {
        if self.feed.len() >= FEED_LINE_CAP {
            self.feed.remove(0);
        }
        self.feed.push((line, FEED_TTL_SECS));
    }

    fn emit_backdrop(bounds: [f32; 4], color: Vec3, alpha: f32, target: &mut dyn MeshTarget) {
        let [x0, y0, x1, y1] = bounds;
        target.push_quad(Quad::glowing(
            Vec3::new(x0, y0, 0.0),
            Vec3::new(x1, y0, 0.0),
            Vec3::new(x1, y1, 0.0),
            Vec3::new(x0, y1, 0.0),
            Vec3::new(0.0, 0.0001, 1.0),
            color,
            -alpha.clamp(0.05, 0.95),
        ));
    }

    fn emit_left_text(stamp: PixelStamp, text: &str, left_x: f32, center_y: f32, target: &mut dyn MeshTarget) {
        let count = text.chars().count() as f32;
        if count <= 0.0 {
            return;
        }
        let width = count * (0.0044 * 4.0) - 0.0044;
        stamp.emit_centered(text, left_x + width * 0.5, center_y, target);
    }

    fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        let base_x = -0.94;
        let mut y = -0.74;
        if let ChatCompose::Composing { ref draft, .. } = self.compose {
            Self::emit_backdrop([-0.96, -0.88, 0.12, -0.80], Vec3::new(0.05, 0.08, 0.12), 0.78, target);
            let prompt = format!("NEARBY: {draft}_");
            let stamp = PixelStamp::new(0.0044, 0.0076, Vec3::new(0.42, 0.96, 0.88));
            Self::emit_left_text(stamp, &prompt, base_x, -0.84, target);
        }
        for (line, ttl) in self.feed.iter().rev() {
            let fade = (*ttl / 2.5).clamp(0.25, 1.0);
            Self::emit_backdrop([-0.96, y - 0.028, 0.12, y + 0.028], Vec3::new(0.04, 0.06, 0.09), 0.48 * fade, target);
            let stamp = PixelStamp::new(0.0044, 0.0076, Vec3::new(0.92, 0.95, 0.98) * fade);
            Self::emit_left_text(stamp, line, base_x, y, target);
            y += 0.064;
        }
    }
}

impl<M: SessionModal> SessionModal for ProximityChatModal<M> {
    fn open(&self) -> bool {
        self.composing() || self.origin.open()
    }

    #[cfg(test)]
    fn dialing_active(&self) -> bool {
        self.origin.dialing_active()
    }

    fn toggle_gameplay_trigger(
        &mut self,
        trigger: InputTrigger,
        surface: &dyn PlatformSurface,
        avatar: &mut dyn Avatar,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        if self.origin.open() || self.composing() {
            return self.origin.toggle_gameplay_trigger(trigger, surface, avatar, world);
        }
        if matches!(trigger, InputTrigger::Key(KeyCode::Enter | KeyCode::KeyT)) {
            let swallow = matches!(trigger, InputTrigger::Key(KeyCode::KeyT));
            self.compose = ChatCompose::Composing { draft: String::new(), swallow_open_char: swallow };
            return true;
        }
        self.origin.toggle_gameplay_trigger(trigger, surface, avatar, world)
    }

    fn sync_world_chart(&mut self, avatar: &dyn Avatar, world: &dyn PlanetaryWorld) {
        self.origin.sync_world_chart(avatar, world);
    }

    fn sync_portal_chat(&mut self, world: &mut dyn PlanetaryWorld) {
        for msg in self.outbox.drain(..) {
            world.utter_nearby(&msg);
        }
        let mut incoming: Vec<ProximityUtterance> = Vec::new();
        world.drain_utterances(&mut incoming);
        for utterance in incoming {
            self.append_feed(utterance.formatted_line());
        }
        self.origin.sync_portal_chat(world);
    }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
        if self.composing() {
            self.compose = ChatCompose::Idle;
            surface.capture_cursor();
            return ModalDirective::ContinueSession;
        }
        self.origin.toggle_escape(surface)
    }

    fn focus_regained(&mut self) -> ModalDirective { self.origin.focus_regained() }
    fn move_cursor(&mut self, point: ScreenPoint) { self.origin.move_cursor(point); }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        if self.composing() {
            return ModalDirective::ContinueSession;
        }
        self.origin.press_mouse(surface, avatar)
    }

    fn release_mouse(&mut self) { self.origin.release_mouse(); }

    fn handle_key(&mut self, code: KeyCode, ctrl_held: bool, avatar: &mut dyn Avatar) -> ModalDirective {
        if let ChatCompose::Composing { ref mut draft, ref mut swallow_open_char } = self.compose {
            *swallow_open_char = false;
            match code {
                KeyCode::Backspace => {
                    let _ = draft.pop();
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    let trimmed = draft.trim().to_string();
                    self.compose = ChatCompose::Idle;
                    if !trimmed.is_empty() {
                        self.append_feed(format!("YOU: {trimmed}"));
                        self.outbox.push(trimmed);
                    }
                }
                KeyCode::Escape => {
                    self.compose = ChatCompose::Idle;
                }
                _ => {}
            }
            return ModalDirective::ContinueSession;
        }
        self.origin.handle_key(code, ctrl_held, avatar)
    }

    fn handle_text(&mut self, text: &str) {
        if let ChatCompose::Composing { ref mut draft, ref mut swallow_open_char } = self.compose {
            if *swallow_open_char {
                *swallow_open_char = false;
                if text.eq_ignore_ascii_case("t") {
                    return;
                }
            }
            for ch in text.chars() {
                if !ch.is_control() && draft.chars().count() < DRAFT_CHAR_CAP {
                    draft.push(ch);
                }
            }
            return;
        }
        self.origin.handle_text(text);
    }

    fn handle_trigger(&mut self, trigger: InputTrigger, avatar: &mut dyn Avatar) -> bool {
        if self.composing() {
            return false;
        }
        self.origin.handle_trigger(trigger, avatar)
    }

    fn sync_controls(&self, controls: &mut dyn KeyboardControls) { self.origin.sync_controls(controls); }
    fn announce_portal(&mut self, invite: PortalInvite) { self.origin.announce_portal(invite); }
    fn announce_auto_punch(&mut self) { self.origin.announce_auto_punch(); }
    fn announce_punch_code(&mut self, punch_code: String) { self.origin.announce_punch_code(punch_code); }
    fn complete_join(&mut self, surface: &dyn PlatformSurface) { self.origin.complete_join(surface); }
    fn report_join_failure(&mut self) { self.origin.report_join_failure(); }

    fn step(&mut self, dt: f32) {
        for (_, ttl) in &mut self.feed {
            *ttl -= dt.max(0.0);
        }
        self.feed.retain(|(_, ttl)| *ttl > 0.0);
        self.origin.step(dt);
    }

    #[cfg(test)]
    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        if self.origin.open() {
            self.origin.emit_ui(avatar, target);
            return;
        }
        if self.composing() || !self.feed.is_empty() {
            self.emit_overlay(target);
            return;
        }
        self.origin.emit_ui(avatar, target);
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        if !self.composing() {
            self.origin.emit_studio(target);
        }
    }

    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32) {
        self.origin.draw_layer(gl, shader, avatar, aspect);
        if self.origin.open() || (!self.composing() && self.feed.is_empty()) {
            return;
        }
        let mut ui_batch = CpuMeshBatch::with_quad_capacity(512);
        self.emit_overlay(&mut ui_batch);
        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
        shader.bind(gl, &Mat4::IDENTITY, Vec3::Z);
        ui_batch.draw(gl);
        unsafe {
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
        }
    }
}
