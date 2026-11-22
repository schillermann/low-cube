use crate::avatar::Avatar;
use crate::drive::{CourseMode, NavigationCourse, OrrerySnapshot, TransferArc};
use crate::menu::widget::ScreenPoint;
use crate::surface::PlatformSurface;
use super::viewport::{ChartSelection, ChartViewport};

/// Live interactive session state of an open `SolarChartModal` (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub(super) struct ActiveChart {
    pub(super) snapshot: OrrerySnapshot,
    pub(super) viewport: ChartViewport,
    pub(super) selection: ChartSelection,
    pub(super) mode_and_accel: (CourseMode, f32),
}

impl ActiveChart {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        snapshot: OrrerySnapshot,
        viewport: ChartViewport,
        selection: ChartSelection,
        mode_and_accel: (CourseMode, f32),
    ) -> Self {
        Self { snapshot, viewport, selection, mode_and_accel }
    }

    pub(super) fn max_span(&self) -> f64 {
        self.snapshot.transfer(|atlas, _, _, _| {
            atlas.ephemeris().map_or(4_000.0, |e| e.proportions().astronomical_unit() * 36.0)
        })
    }

    pub(super) fn replot_selected(&mut self, avatar: &mut dyn Avatar) {
        let ChartSelection::Locked(body) = self.selection else {
            return;
        };
        let (mode, burn_accel) = self.mode_and_accel;
        let next_course = self.snapshot.transfer(|atlas, ship_pos, (_, vel), prev| {
            if let Some(arc) = TransferArc::intercept_body(ship_pos, vel, body, mode, burn_accel, atlas) {
                NavigationCourse::Charted(arc)
            } else {
                prev
            }
        });
        self.snapshot = self.snapshot.transfer(|atlas, pos, motion, _| {
            OrrerySnapshot::new(atlas, pos, motion, next_course)
        });
        avatar.install_navigation_course(next_course);
    }

    /// Processes a mouse press inside the open chart; returns `true` when the chart requested closing.
    pub(super) fn press_pointer(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> bool {
        let pt: ScreenPoint = self.viewport.cursor();
        let ux = 9.0 / 16.0;
        if ChartViewport::close_button().contains(pt) {
            surface.capture_cursor();
            return true;
        }
        if ChartViewport::mode_button().contains(pt) {
            self.mode_and_accel.0 = self.mode_and_accel.0.toggled();
            self.replot_selected(avatar);
            return false;
        }
        if ChartViewport::zoom_in_button().contains(pt) {
            self.viewport.zoom(0.62, self.max_span());
            return false;
        }
        if ChartViewport::zoom_out_button().contains(pt) {
            self.viewport.zoom(1.60, self.max_span());
            return false;
        }
        if ChartViewport::ship_button().contains(pt) {
            let ship = self.snapshot.transfer(|_, p, _, _| p.as_dvec3());
            self.viewport.recenter_on(ship);
            return false;
        }
        let atlas = self.snapshot.transfer(|a, _, _, _| a);
        if let Some(body) = self.viewport.pick_body(atlas, ux) {
            self.selection = ChartSelection::Locked(body);
            self.replot_selected(avatar);
        } else if ChartViewport::inside_canvas(pt) {
            self.viewport.begin_drag();
        }
        false
    }
}
