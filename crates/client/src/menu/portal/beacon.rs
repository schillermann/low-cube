use glam::Vec3;
use world::MeshTarget;
use crate::avatar::AvatarMotor;
use crate::menu::clipboard::TextClipboard;
use crate::menu::glyph::PixelStamp;
use crate::menu::view::MenuView;
use crate::menu::widget::ScreenBounds;
use super::invite::PortalNotice;

/// Polymorphic guest NAT hole-punch beacon keeping visual & clipboard state alive across menu closes.
#[derive(Clone, Debug, PartialEq)]
pub enum PunchBeacon {
    Silent,
    Listening {
        reply_code: String,
        remaining: f32,
    },
}

impl PunchBeacon {
    pub const fn open(reply_code: String) -> Self {
        Self::Listening { reply_code, remaining: 180.0 }
    }

    pub const fn listening(&self) -> bool {
        matches!(self, Self::Listening { .. })
    }

    pub fn advance(&mut self, dt: f32) {
        if let Self::Listening { ref mut remaining, .. } = *self {
            *remaining = (*remaining - dt).max(0.0);
        }
    }

    pub fn copy_into(&self, clipboard: &dyn TextClipboard) -> PortalNotice {
        match *self {
            Self::Silent => PortalNotice::Idle,
            Self::Listening { ref reply_code, .. } => {
                let chat_line = format!("CO-OP REPLY CODE: {reply_code}");
                if clipboard.store_text(&chat_line) {
                    PortalNotice::ReplyCopied
                } else {
                    PortalNotice::ClipboardUnavailable
                }
            }
        }
    }

    fn pulse_glyph(&self, remaining: f32) -> &'static str {
        let phase = (((180.0 - remaining).max(0.0) * 2.4) as usize) % 3;
        match phase {
            0 => "[*..]",
            1 => "[.*.]",
            _ => "[..*]",
        }
    }

    fn countdown_label(&self, remaining: f32) -> String {
        let total = remaining.ceil().max(0.0) as u32;
        format!("{:02}:{:02}", total / 60, total % 60)
    }

    pub fn emit_hud_banner(&self, target: &mut dyn MeshTarget) {
        let Self::Listening { ref reply_code, remaining } = *self else {
            return;
        };
        let pill = ScreenBounds::new(-0.66, 0.80, 0.66, 0.96);
        pill.emit_framed(
            Vec3::new(0.05, 0.11, 0.16),
            Vec3::new(0.32, 0.78, 0.62),
            0.006,
            target,
        );
        let top = format!(
            "CO-OP SIGNAL OPEN {} {} - TELL HOST: {}",
            self.pulse_glyph(remaining),
            self.countdown_label(remaining),
            reply_code,
        );
        PixelStamp::caption(Vec3::new(0.96, 0.84, 0.36)).emit_centered(&top, 0.0, 0.90, target);
        PixelStamp::caption(Vec3::new(0.68, 0.90, 0.78)).emit_centered(
            "CODE COPIED • KEEP PLAYING WHILE HOST CONNECTS • [ESC] CO-OP",
            0.0,
            0.84,
            target,
        );
    }

    pub fn emit_waiting_hero(&self, code_box: ScreenBounds, target: &mut dyn MeshTarget) {
        let Self::Listening { ref reply_code, remaining } = *self else {
            return;
        };
        code_box.emit_framed(
            Vec3::new(0.04, 0.08, 0.13),
            Vec3::new(0.96, 0.82, 0.34),
            0.008,
            target,
        );
        PixelStamp::title(Vec3::new(0.98, 0.88, 0.42))
            .emit_centered(reply_code, 0.0, code_box.center_y(), target);
        let live_line = format!(
            "2. SIGNAL OPEN {} {} - WAITING FOR HOST PUNCH",
            self.pulse_glyph(remaining),
            self.countdown_label(remaining),
        );
        PixelStamp::caption(Vec3::new(0.42, 0.90, 0.62)).emit_centered(&live_line, 0.0, 0.02, target);
        PixelStamp::caption(Vec3::new(0.76, 0.82, 0.90)).emit_centered(
            "CLOSING THIS MENU KEEPS YOUR CONNECTION OPEN",
            0.0,
            -0.04,
            target,
        );
    }

    pub fn emit_hub_summary(&self, target: &mut dyn MeshTarget) {
        let Self::Listening { ref reply_code, remaining } = *self else {
            return;
        };
        let head = format!("REPLY: {reply_code}");
        let sub = format!(
            "SIGNAL OPEN {} {} - WAITING FOR HOST",
            self.pulse_glyph(remaining),
            self.countdown_label(remaining),
        );
        PixelStamp::title(Vec3::new(0.96, 0.82, 0.34)).emit_centered(&head, 0.0, -0.18, target);
        PixelStamp::caption(Vec3::new(0.42, 0.90, 0.62)).emit_centered(&sub, 0.0, -0.29, target);
    }
}

/// Encapsulated guest signal beacon and transient portal notice state.
#[derive(Clone, Debug, PartialEq)]
pub struct PortalStatus {
    beacon: PunchBeacon,
    notice: PortalNotice,
}

impl PortalStatus {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(beacon: PunchBeacon, notice: PortalNotice) -> Self {
        Self { beacon, notice }
    }

    pub const fn initial(notice: PortalNotice) -> Self {
        Self::new(PunchBeacon::Silent, notice)
    }

    pub const fn listening(&self) -> bool {
        self.beacon.listening()
    }

    pub const fn beacon(&self) -> &PunchBeacon {
        &self.beacon
    }

    pub const fn notice(&self) -> PortalNotice {
        self.notice
    }

    pub fn advance(&mut self, dt: f32) {
        self.beacon.advance(dt);
    }

    pub fn open_beacon(&mut self, reply_code: String, clipboard: &dyn TextClipboard) {
        self.beacon = PunchBeacon::open(reply_code);
        self.notice = self.beacon.copy_into(clipboard);
    }

    pub fn copy_beacon(&mut self, clipboard: &dyn TextClipboard) -> bool {
        if !self.beacon.listening() {
            return false;
        }
        self.notice = self.beacon.copy_into(clipboard);
        true
    }

    pub fn silence_beacon(&mut self) {
        self.beacon = PunchBeacon::Silent;
        if matches!(self.notice, PortalNotice::ReplyCopied) {
            self.notice = PortalNotice::Idle;
        }
    }

    pub fn record_notice(&mut self, next: PortalNotice) {
        self.notice = next;
    }
}

/// Focused `MenuView` adapter emitting the gameplay avatar reticle.
pub struct ReticleHud<'a> {
    motor: &'a dyn AvatarMotor,
    aspect: f32,
}

impl<'a> ReticleHud<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(motor: &'a dyn AvatarMotor, aspect: f32) -> Self {
        Self { motor, aspect }
    }
}

impl MenuView for ReticleHud<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.motor.emit_reticle_for_aspect(self.aspect, target);
    }
}

/// Decorator over any `MenuView` overlaying the live Co-op NAT signal banner while awaiting the host.
pub struct BeaconHud<'a, V> {
    beacon: &'a PunchBeacon,
    inner: V,
}

impl<'a, V: MenuView> BeaconHud<'a, V> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(beacon: &'a PunchBeacon, inner: V) -> Self {
        Self { beacon, inner }
    }
}

impl<V: MenuView> MenuView for BeaconHud<'_, V> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.inner.emit(target);
        self.beacon.emit_hud_banner(target);
    }
}
