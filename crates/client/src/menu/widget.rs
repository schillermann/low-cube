use glam::Vec3;
use world::{MeshTarget, Quad};
use super::glyph::PixelStamp;

/// Immutable Normalized Device Coordinate cursor point in [-1.0, 1.0].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    x: f32,
    y: f32,
}

impl ScreenPoint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn center() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn horizontal_delta(self, previous: Self) -> f32 {
        self.x - previous.x
    }
}

/// Axis-aligned Normalized Device Coordinate rectangle capable of hit-testing and self-drawing.
#[derive(Clone, Copy, Debug)]
pub struct ScreenBounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl ScreenBounds {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub fn contains(&self, pt: ScreenPoint) -> bool {
        pt.x >= self.min_x && pt.x <= self.max_x && pt.y >= self.min_y && pt.y <= self.max_y
    }

    pub fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) * 0.5
    }

    pub fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) * 0.5
    }

    pub fn emit_fill(&self, color: Vec3, target: &mut dyn MeshTarget) {
        target.push_quad(Quad::new(
            Vec3::new(self.min_x, self.min_y, 0.0),
            Vec3::new(self.max_x, self.min_y, 0.0),
            Vec3::new(self.max_x, self.max_y, 0.0),
            Vec3::new(self.min_x, self.max_y, 0.0),
            Vec3::Z,
            color,
        ));
    }

    pub fn emit_framed(&self, fill: Vec3, rim: Vec3, border: f32, target: &mut dyn MeshTarget) {
        Self::new(
            self.min_x - border,
            self.min_y - border,
            self.max_x + border,
            self.max_y + border,
        )
        .emit_fill(rim, target);
        self.emit_fill(fill, target);
    }

    pub fn emit_button(
        &self,
        label: &str,
        pointer: ScreenPoint,
        fill: Vec3,
        selected: bool,
        target: &mut dyn MeshTarget,
    ) {
        let hovered = self.contains(pointer);
        let active_fill = if selected {
            (fill * 1.45).clamp(Vec3::ZERO, Vec3::ONE)
        } else if hovered {
            (fill * 1.25).clamp(Vec3::ZERO, Vec3::ONE)
        } else {
            fill
        };
        let rim = if selected {
            Vec3::new(0.95, 0.78, 0.28)
        } else if hovered {
            Vec3::new(0.75, 0.82, 0.90)
        } else {
            Vec3::new(0.26, 0.30, 0.36)
        };
        self.emit_framed(active_fill, rim, 0.006, target);
        let text_col = if selected {
            Vec3::new(1.0, 0.96, 0.78)
        } else {
            Vec3::new(0.94, 0.95, 0.97)
        };
        PixelStamp::caption(text_col).emit_centered(label, self.center_x(), self.center_y(), target);
    }

    pub fn emit_swatch(
        &self,
        label: &str,
        pointer: ScreenPoint,
        swatch_col: Vec3,
        selected: bool,
        target: &mut dyn MeshTarget,
    ) {
        let hovered = self.contains(pointer);
        let rim = if selected {
            Vec3::new(0.98, 0.84, 0.25)
        } else if hovered {
            Vec3::new(0.90, 0.92, 0.96)
        } else {
            Vec3::new(0.22, 0.24, 0.28)
        };
        let border = if selected { 0.010 } else { 0.005 };
        self.emit_framed(swatch_col, rim, border, target);
        let luma = swatch_col.dot(Vec3::new(0.299, 0.587, 0.114));
        let text_col = if luma > 0.55 {
            Vec3::new(0.12, 0.10, 0.08)
        } else {
            Vec3::new(0.96, 0.94, 0.90)
        };
        PixelStamp::caption(text_col).emit_centered(label, self.center_x(), self.center_y(), target);
    }

    pub fn emit_gauge(&self, label: &str, ratio: f32, fill: Vec3, target: &mut dyn MeshTarget) {
        self.emit_framed(Vec3::new(0.11, 0.13, 0.17), Vec3::new(0.25, 0.29, 0.35), 0.004, target);
        let clamped = ratio.clamp(0.05, 1.0);
        let fill_max_x = self.min_x + (self.max_x - self.min_x) * clamped;
        Self::new(self.min_x, self.min_y, fill_max_x, self.max_y).emit_fill(fill, target);
        PixelStamp::caption(Vec3::new(0.95, 0.96, 0.98)).emit_centered(
            label,
            self.center_x(),
            self.center_y(),
            target,
        );
    }
}
