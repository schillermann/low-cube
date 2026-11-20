use glam::Vec3;
use world::{BlockMold, CraftBlueprint, MeshTarget};
use crate::attire::{BeltSatchel, BodyPose, HeldRelic, LimbFrame, YouthAttire};
use crate::menu::PixelStamp;
use super::{FolioView, SavedBlueprint};

/// Collaborator rendering the 3D view-space blueprint naming input and crane selection catalog plaques.
pub(super) struct FolioPlaque<'a> {
    view: &'a FolioView,
    owner: (&'a BeltSatchel, YouthAttire),
}

impl<'a> FolioPlaque<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(view: &'a FolioView, owner: (&'a BeltSatchel, YouthAttire)) -> Self {
        Self { view, owner }
    }

    pub(super) fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        match self.view {
            FolioView::Closed => {}
            FolioView::NamingDraft { draft, captured } => self.emit_naming(draft, captured, pose, target),
            FolioView::RigCatalog { entries, cursor, .. } => self.emit_catalog(entries, *cursor, pose, target),
        }
    }

    fn emit_naming(&self, draft: &str, captured: &CraftBlueprint, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let tray = pose.view_frame().shifted(0.0, 0.01, 0.44);
        tray.emit_framed_plaque(
            0.290,
            0.165,
            0.0035,
            Vec3::new(0.06, 0.11, 0.22),
            Vec3::new(0.88, 0.72, 0.26),
            target,
        );
        PixelStamp::new(0.0028, 0.0028, Vec3::new(0.72, 1.48, 1.65))
            .emit_on_frame("SAVE VEHICLE BLUEPRINT", 0.0, 0.130, 0.016, &tray, target);

        let icon_frame = tray.shifted(-0.220, 0.075, -0.012).scaled(0.42);
        HeldRelic::VehicleBlueprint
            .figure(BlockMold::cube())
            .emit_in_slot(true, Vec3::new(0.88, 0.72, 0.26), &icon_frame, target);

        let kind = if captured.spacefaring() { "SPACECRAFT" } else { "VEHICLE" };
        let sub = format!("CAPTURED {}: {} PARTS", kind, captured.part_count());
        PixelStamp::new(0.0021, 0.0021, Vec3::new(1.52, 1.38, 0.78))
            .emit_on_frame(&sub, 0.025, 0.078, 0.016, &tray, target);

        let box_frame = tray.shifted(0.0, 0.012, -0.008);
        box_frame.emit_framed_plaque(
            0.245,
            0.026,
            0.0022,
            Vec3::new(0.04, 0.07, 0.14),
            Vec3::new(0.38, 0.76, 0.98),
            target,
        );
        if draft.is_empty() {
            PixelStamp::new(0.0023, 0.0023, Vec3::new(0.68, 0.88, 1.12))
                .emit_on_frame("TYPE BLUEPRINT NAME|", 0.0, 0.0, 0.016, &box_frame, target);
        } else {
            let shown = format!("{draft}|");
            PixelStamp::new(0.0025, 0.0025, Vec3::new(1.58, 1.56, 1.42))
                .emit_on_frame(&shown, 0.0, 0.0, 0.016, &box_frame, target);
        }

        let preview = SavedBlueprint::new(String::new(), captured.clone());
        let req = preview.manifest_caption();
        PixelStamp::new(0.0018, 0.0018, Vec3::new(1.18, 1.38, 1.52))
            .emit_on_frame(&req, 0.0, -0.058, 0.016, &tray, target);
        PixelStamp::new(0.0019, 0.0019, Vec3::new(1.55, 1.30, 0.48))
            .emit_on_frame("TYPE NAME   ENTER/CLICK: SAVE   BACKSPACE: ERASE   ESC: CANCEL", 0.0, -0.128, 0.016, &tray, target);
    }

    fn emit_catalog(&self, entries: &[SavedBlueprint], cursor: usize, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let tray = pose.view_frame().shifted(0.0, 0.01, 0.44);
        tray.emit_framed_plaque(
            0.295,
            0.235,
            0.0035,
            Vec3::new(0.06, 0.10, 0.19),
            Vec3::new(0.88, 0.72, 0.26),
            target,
        );
        PixelStamp::new(0.0028, 0.0028, Vec3::new(0.72, 1.48, 1.65))
            .emit_on_frame("CRANE BLUEPRINT BUILD", 0.0, 0.205, 0.016, &tray, target);

        if entries.is_empty() {
            PixelStamp::new(0.0023, 0.0023, Vec3::new(1.48, 1.36, 1.12))
                .emit_on_frame("NO SAVED BLUEPRINTS YET", 0.0, 0.030, 0.016, &tray, target);
            PixelStamp::new(0.0019, 0.0019, Vec3::new(1.10, 1.34, 1.52))
                .emit_on_frame("CLICK AN EXISTING VEHICLE WITH BLUEPRINT BLOCK TO SAVE", 0.0, -0.020, 0.016, &tray, target);
            PixelStamp::new(0.0019, 0.0019, Vec3::new(1.55, 1.30, 0.48))
                .emit_on_frame("CLICK / ESC: CLOSE", 0.0, -0.205, 0.016, &tray, target);
            return;
        }

        let sel = cursor % entries.len();
        let visible = entries.len().min(8);
        let start = sel.saturating_sub(visible - 1).min(entries.len().saturating_sub(visible));
        for (vis_idx, (i, entry)) in entries.iter().enumerate().skip(start).take(visible).enumerate() {
            self.emit_row(entry, i == sel, vis_idx, &tray, target);
        }

        PixelStamp::new(0.0019, 0.0019, Vec3::new(1.55, 1.30, 0.48)).emit_on_frame(
            "WASD/WHEEL: SELECT   CLICK/ENTER/E: BUILD   DEL: ERASE   ESC: CLOSE",
            0.0,
            -0.212,
            0.016,
            &tray,
            target,
        );
    }

    fn emit_row(&self, entry: &SavedBlueprint, selected: bool, vis_idx: usize, tray: &LimbFrame, target: &mut dyn MeshTarget) {
        let (satchel, attire) = self.owner;
        let ready = entry.buildable(satchel, attire);
        let row_y = 0.156 - (vis_idx as f32) * 0.042;
        let row_frame = tray.shifted(0.0, row_y, if selected { -0.012 } else { 0.0 });
        let row_bg = if selected { Vec3::new(0.14, 0.22, 0.34) } else { Vec3::new(0.09, 0.13, 0.21) };
        let row_rim = if selected {
            Vec3::new(0.96, 0.80, 0.26)
        } else if ready {
            Vec3::new(0.28, 0.62, 0.38)
        } else {
            Vec3::new(0.36, 0.24, 0.18)
        };
        row_frame.emit_framed_plaque(0.272, 0.018, 0.0020, row_bg, row_rim, target);

        let icon_frame = row_frame.shifted(-0.234, 0.0, -0.010).scaled(0.36);
        HeldRelic::VehicleBlueprint
            .figure(BlockMold::cube())
            .emit_in_slot(selected, row_rim, &icon_frame, target);

        let status = if ready { "READY" } else { "NEED PARTS" };
        let heading = format!("{} ({} PARTS) - {}", entry.title_text(), entry.blueprint().part_count(), status);
        let title_col = if ready { Vec3::new(1.05, 1.56, 0.96) } else { Vec3::new(1.48, 1.34, 1.10) };
        PixelStamp::new(0.0019, 0.0019, title_col)
            .emit_on_frame(&heading, 0.022, 0.007, 0.016, &row_frame, target);

        let req_col = if ready { Vec3::new(0.88, 1.48, 0.90) } else { Vec3::new(1.54, 0.94, 0.68) };
        let req_text = entry.requirement_summary(satchel, attire);
        PixelStamp::new(0.0017, 0.0017, req_col)
            .emit_on_frame(&req_text, 0.022, -0.008, 0.016, &row_frame, target);
    }
}
