use glam::Vec3;
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::black_box::{DiskDossier, FileBlackBox};
use super::clipboard::TextClipboard;
use super::glyph::PixelStamp;
use super::widget::{ScreenBounds, ScreenPoint};

/// Directive returned when interacting with the Flight Blackbox diagnostics panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlackBoxDirective {
    StayInBlackBox,
    ReturnToPauseMenu,
    ResumeWithPolaroid,
}

/// User-facing status notice displayed inside the Flight Blackbox panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlackBoxNotice {
    Idle,
    Activated,
    Silenced,
    EmptyBuffer,
    ReportSaved,
}

/// Encapsulated Flight Blackbox menu panel toggling `LiveTape`/`SilentTape` and exporting Polaroid bug reports.
pub struct BlackBoxPanel {
    notice: BlackBoxNotice,
    clipboard: Box<dyn TextClipboard>,
    profile: FileBlackBox,
    report_folder: String,
}

impl BlackBoxPanel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        notice: BlackBoxNotice,
        clipboard: Box<dyn TextClipboard>,
        profile: FileBlackBox,
        report_folder: String,
    ) -> Self {
        Self { notice, clipboard, profile, report_folder }
    }

    pub fn with_clipboard(clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(
            BlackBoxNotice::Idle,
            clipboard,
            FileBlackBox::profile(),
            "reports".to_string(),
        )
    }

    #[cfg(test)]
    pub fn sandboxed(clipboard: Box<dyn TextClipboard>, profile_path: String, report_folder: String) -> Self {
        Self::new(
            BlackBoxNotice::Idle,
            clipboard,
            FileBlackBox::new(profile_path),
            report_folder,
        )
    }

    pub fn toggle_button() -> ScreenBounds {
        ScreenBounds::new(-0.34, 0.04, 0.34, 0.16)
    }

    pub fn report_button() -> ScreenBounds {
        ScreenBounds::new(-0.34, -0.16, 0.34, -0.04)
    }

    pub fn back_button() -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.44, 0.28, -0.32)
    }

    pub fn press_pointer(&mut self, pointer: ScreenPoint, avatar: &mut dyn Avatar) -> BlackBoxDirective {
        if Self::toggle_button().contains(pointer) {
            let now_active = avatar.toggle_black_box();
            self.profile.save_active(now_active);
            self.notice = if now_active {
                BlackBoxNotice::Activated
            } else {
                BlackBoxNotice::Silenced
            };
            BlackBoxDirective::StayInBlackBox
        } else if Self::report_button().contains(pointer) {
            let mut dossier = DiskDossier::in_folder(&self.report_folder);
            if avatar.export_incident_dossier(&mut dossier).is_some() {
                let saved_path = dossier.commit_to_disk();
                let _ = self.clipboard.store_text(&saved_path);
                self.notice = BlackBoxNotice::ReportSaved;
                BlackBoxDirective::ResumeWithPolaroid
            } else {
                self.notice = BlackBoxNotice::EmptyBuffer;
                BlackBoxDirective::StayInBlackBox
            }
        } else if Self::back_button().contains(pointer) {
            BlackBoxDirective::ReturnToPauseMenu
        } else {
            BlackBoxDirective::StayInBlackBox
        }
    }

    pub fn emit(&self, pointer: ScreenPoint, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        let bg = Vec3::new(0.08, 0.10, 0.15);
        let rim = Vec3::new(0.32, 0.39, 0.50);
        let gold = Vec3::new(0.96, 0.82, 0.34);
        let active = avatar.black_box_active();
        let populated = avatar.black_box_populated();
        let (ticks, anomalies) = avatar.black_box_summary();

        ScreenBounds::new(-0.44, -0.54, 0.44, 0.44).emit_framed(bg, rim, 0.008, target);
        PixelStamp::title(gold).emit_centered("FLIGHT LOG", 0.0, 0.35, target);

        let status_line = if active {
            format!("RING BUFFER: {ticks} TICKS  ANOMALIES: {anomalies}")
        } else {
            "MODE: SILENT NULL OBJECT (NO OVERHEAD)".to_string()
        };
        let status_col = if anomalies > 0 {
            Vec3::new(0.96, 0.46, 0.28)
        } else if active {
            Vec3::new(0.56, 0.88, 0.68)
        } else {
            Vec3::new(0.62, 0.68, 0.76)
        };
        PixelStamp::caption(status_col).emit_centered(&status_line, 0.0, 0.24, target);

        let (toggle_label, toggle_col) = if active {
            ("LOG: ON", Vec3::new(0.18, 0.48, 0.28))
        } else {
            ("LOG: OFF", Vec3::new(0.28, 0.30, 0.36))
        };
        Self::toggle_button().emit_button(toggle_label, pointer, toggle_col, active, target);

        let report_col = if populated {
            Vec3::new(0.58, 0.38, 0.14)
        } else {
            Vec3::new(0.22, 0.24, 0.28)
        };
        Self::report_button().emit_button("CREATE LOG REPORT", pointer, report_col, populated, target);

        let (note_text, note_col) = match self.notice {
            BlackBoxNotice::Idle => ("CAPTURES LAST 15S FOR REPLAY REPORTS", Vec3::new(0.58, 0.64, 0.74)),
            BlackBoxNotice::Activated => ("RECORDING LIVE 120HZ RING BUFFER", Vec3::new(0.52, 0.90, 0.64)),
            BlackBoxNotice::Silenced => ("RECORDING STOPPED AND BUFFER CLEARED", Vec3::new(0.86, 0.72, 0.44)),
            BlackBoxNotice::EmptyBuffer => ("PLAY A FEW TICKS WITH LOG ON FIRST", Vec3::new(0.94, 0.46, 0.36)),
            BlackBoxNotice::ReportSaved => ("SAVED TO REPORTS/ AND COPIED PATH", Vec3::new(0.52, 0.90, 0.64)),
        };
        PixelStamp::caption(note_col).emit_centered(note_text, 0.0, -0.24, target);

        Self::back_button().emit_button("BACK TO MENU", pointer, Vec3::new(0.24, 0.28, 0.38), false, target);
    }
}
