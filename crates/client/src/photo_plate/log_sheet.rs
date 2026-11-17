use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::attire::LimbFrame;
use crate::menu::PixelStamp;

/// Tilted diagnostic log-report document rendered at screen center when a player exports a Flight Log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IncidentSheet {
    ticks: u16,
    anomalies: u8,
}

impl IncidentSheet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ticks: u16, anomalies: u8) -> Self {
        Self { ticks, anomalies }
    }

    fn emit_rect(
        frame: &LimbFrame,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        target.push_quad(Quad::new(
            frame.point(min_x, min_y, 0.0),
            frame.point(max_x, min_y, 0.0),
            frame.point(max_x, max_y, 0.0),
            frame.point(min_x, max_y, 0.0),
            Vec3::Z,
            color,
        ));
    }

    pub fn emit(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let shadow = Vec3::new(0.04, 0.05, 0.08);
        let board = Vec3::new(0.24, 0.27, 0.33);
        let paper = Vec3::new(0.95, 0.94, 0.89);
        let header_bg = Vec3::new(0.14, 0.18, 0.26);
        let rule_blue = Vec3::new(0.78, 0.83, 0.88);
        let margin_red = Vec3::new(0.84, 0.46, 0.44);
        let clip_metal = Vec3::new(0.68, 0.72, 0.78);

        // Drop shadow, clipboard backing, and ruled log sheet
        Self::emit_rect(frame, -0.33, -0.47, 0.37, 0.42, shadow, target);
        Self::emit_rect(frame, -0.355, -0.455, 0.355, 0.435, board, target);
        Self::emit_rect(frame, -0.338, -0.438, 0.338, 0.405, paper, target);

        // Top metallic clipboard clamp and dark dossier title banner
        Self::emit_rect(frame, -0.11, 0.392, 0.11, 0.452, clip_metal, target);
        Self::emit_rect(frame, -0.07, 0.406, 0.07, 0.436, header_bg, target);
        Self::emit_rect(frame, -0.338, 0.295, 0.338, 0.392, header_bg, target);
        PixelStamp::new(0.0060, 0.0086, Vec3::new(0.96, 0.84, 0.38))
            .emit_on_frame("LOG REPORT", 0.0, 0.342, 0.0, frame, target);

        // Vertical red margin line and horizontal ruled log lines
        Self::emit_rect(frame, -0.268, -0.235, -0.262, 0.285, margin_red, target);
        for idx in 0..9 {
            let y = 0.245 - (idx as f32) * 0.056;
            Self::emit_rect(frame, -0.318, y, 0.318, y + 0.004, rule_blue, target);
        }

        // Telemetry status badge and summary metrics
        let (badge_col, status_line) = if self.anomalies > 0 {
            (Vec3::new(0.76, 0.22, 0.18), format!("ALERT: {} ANOMALIES", self.anomalies))
        } else {
            (Vec3::new(0.18, 0.48, 0.28), "STATE: VERIFIED OK".to_string())
        };
        Self::emit_rect(frame, -0.245, 0.196, 0.295, 0.240, badge_col, target);
        PixelStamp::new(0.0044, 0.0062, Vec3::ONE)
            .emit_on_frame(&status_line, 0.025, 0.218, 0.0, frame, target);

        let ticks_line = format!("TICKS LOGGED: {}", self.ticks);
        PixelStamp::new(0.0044, 0.0062, Vec3::new(0.14, 0.18, 0.26))
            .emit_on_frame(&ticks_line, 0.015, 0.156, 0.0, frame, target);

        // Stylized TOML log lines (section tag + indented key/value bars) on the ruled paper
        let tag_col = Vec3::new(0.22, 0.36, 0.56);
        let text_bar = Vec3::new(0.36, 0.40, 0.48);
        let val_bar = Vec3::new(0.20, 0.50, 0.44);
        let spans: [(f32, f32, bool); 6] = [
            (0.092, 0.20, true),
            (0.036, 0.42, false),
            (-0.020, 0.36, false),
            (-0.076, 0.22, true),
            (-0.132, 0.46, false),
            (-0.188, 0.40, false),
        ];
        for (idx, (y, width, section_header)) in spans.into_iter().enumerate() {
            let bullet_col = if self.anomalies > 0 && idx == 4 {
                Vec3::new(0.84, 0.32, 0.20)
            } else {
                 Vec3::new(0.48, 0.54, 0.62)
            };
            Self::emit_rect(frame, -0.302, y, -0.282, y + 0.020, bullet_col, target);
            if section_header {
                Self::emit_rect(frame, -0.242, y, -0.242 + width, y + 0.022, tag_col, target);
            } else {
                Self::emit_rect(frame, -0.226, y, -0.090, y + 0.018, text_bar, target);
                Self::emit_rect(frame, -0.068, y, -0.068 + width * 0.75, y + 0.018, val_bar, target);
            }
        }

        // Bottom stamp footer matching saved report destination
        Self::emit_rect(frame, -0.318, -0.252, 0.318, -0.246, Vec3::new(0.58, 0.62, 0.70), target);
        PixelStamp::new(0.0050, 0.0072, Vec3::new(0.14, 0.18, 0.26))
            .emit_on_frame("SAVED: INCIDENT LOG.TOML", 0.0, -0.305, 0.0, frame, target);
        PixelStamp::new(0.0042, 0.0060, Vec3::new(0.30, 0.34, 0.42))
            .emit_on_frame("FOLDER: REPORTS/ + CLIPBOARD", 0.0, -0.382, 0.0, frame, target);
    }
}
