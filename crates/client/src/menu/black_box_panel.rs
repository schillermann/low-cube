use glam::Vec3;
use world::MeshTarget;
use crate::avatar::BlackBoxDeck;
use crate::black_box::{DiskDossier, FileBlackBox};
use super::clipboard::TextClipboard;
use super::glyph::PixelStamp;
use super::view::{MenuView, TitledCard};
use super::widget::{ScreenBounds, ScreenPoint};

/// Directive returned when interacting with the Flight Blackbox diagnostics panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlackBoxDirective {
    StayInBlackBox,
    ReturnToSettings,
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

/// Stateless screen bounds layout for the Flight Blackbox panel.
pub struct BlackBoxLayout;

impl BlackBoxLayout {
    /// Primary constructor initializing the Flight Blackbox button layout.
    pub const fn new() -> Self {
        Self
    }

    pub fn toggle_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.34, 0.04, 0.34, 0.16)
    }

    pub fn report_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.34, -0.16, 0.34, -0.04)
    }

    pub fn back_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.44, 0.28, -0.32)
    }
}

/// Inner content of the Flight Blackbox panel decorated by `TitledCard`.
pub struct BlackBoxBody<'a> {
    pointer: ScreenPoint,
    notice: BlackBoxNotice,
    deck: &'a dyn BlackBoxDeck,
}

impl<'a> BlackBoxBody<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pointer: ScreenPoint, notice: BlackBoxNotice, deck: &'a dyn BlackBoxDeck) -> Self {
        Self { pointer, notice, deck }
    }
}

impl MenuView for BlackBoxBody<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let layout = BlackBoxLayout::new();
        let active = self.deck.black_box_active();
        let populated = self.deck.black_box_populated();
        let (ticks, anomalies) = self.deck.black_box_summary();

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
        layout.toggle_button().emit_button(toggle_label, self.pointer, toggle_col, active, target);

        let report_col = if populated {
            Vec3::new(0.58, 0.38, 0.14)
        } else {
            Vec3::new(0.22, 0.24, 0.28)
        };
        layout.report_button().emit_button("CREATE LOG REPORT", self.pointer, report_col, populated, target);

        let (note_text, note_col) = match self.notice {
            BlackBoxNotice::Idle => ("CAPTURES LAST 15S FOR REPLAY REPORTS", Vec3::new(0.58, 0.64, 0.74)),
            BlackBoxNotice::Activated => ("RECORDING LIVE 120HZ RING BUFFER", Vec3::new(0.52, 0.90, 0.64)),
            BlackBoxNotice::Silenced => ("RECORDING STOPPED AND BUFFER CLEARED", Vec3::new(0.86, 0.72, 0.44)),
            BlackBoxNotice::EmptyBuffer => ("PLAY A FEW TICKS WITH LOG ON FIRST", Vec3::new(0.94, 0.46, 0.36)),
            BlackBoxNotice::ReportSaved => ("SAVED TO REPORTS/ AND COPIED PATH", Vec3::new(0.52, 0.90, 0.64)),
        };
        PixelStamp::caption(note_col).emit_centered(note_text, 0.0, -0.24, target);

        layout.back_button().emit_button("BACK", self.pointer, Vec3::new(0.24, 0.28, 0.38), false, target);
    }
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

    pub fn press_pointer(&mut self, pointer: ScreenPoint, deck: &mut dyn BlackBoxDeck) -> BlackBoxDirective {
        let layout = BlackBoxLayout::new();
        if layout.toggle_button().contains(pointer) {
            let now_active = deck.toggle_black_box();
            self.profile.save_active(now_active);
            self.notice = if now_active {
                BlackBoxNotice::Activated
            } else {
                BlackBoxNotice::Silenced
            };
            BlackBoxDirective::StayInBlackBox
        } else if layout.report_button().contains(pointer) {
            let mut dossier = DiskDossier::in_folder(&self.report_folder);
            if deck.export_incident_dossier(&mut dossier).is_some() {
                let saved_path = dossier.commit_to_disk();
                let _ = self.clipboard.store_text(&saved_path);
                self.notice = BlackBoxNotice::ReportSaved;
                BlackBoxDirective::ResumeWithPolaroid
            } else {
                self.notice = BlackBoxNotice::EmptyBuffer;
                BlackBoxDirective::StayInBlackBox
            }
        } else if layout.back_button().contains(pointer) {
            BlackBoxDirective::ReturnToSettings
        } else {
            BlackBoxDirective::StayInBlackBox
        }
    }

    pub fn view<'a>(&'a self, pointer: ScreenPoint, deck: &'a dyn BlackBoxDeck) -> TitledCard<'static, BlackBoxBody<'a>> {
        TitledCard::new(
            ScreenBounds::new(-0.44, -0.54, 0.44, 0.44),
            "FLIGHT LOG",
            0.35,
            BlackBoxBody::new(pointer, self.notice, deck),
        )
    }
}
