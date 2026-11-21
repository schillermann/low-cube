use glam::Vec3;
use world::{MeshTarget, OpticMount};
use super::glyph::PixelStamp;
use super::widget::ScreenBounds;

/// Polymorphic active channel selection across cockpit sight and hull camera pods.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveChannel {
    Cockpit,
    Camera(u8, OpticMount),
}

impl ActiveChannel {
    const fn slot_index(self) -> u8 {
        match self {
            Self::Cockpit => 0,
            Self::Camera(index, _) => index,
        }
    }
}

/// Polymorphic flight-guidance telemetry state displayed in the rocket nose camera viewfinder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuidanceReadout {
    Absent,
    Engaged,
    Manual,
}

/// Camera-channel strip telemetry uniting active slot, total hull camera count, and nose guidance readout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraFeed {
    active: ActiveChannel,
    total: u8,
    guidance: GuidanceReadout,
}

impl CameraFeed {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(active: ActiveChannel, total: u8, guidance: GuidanceReadout) -> Self {
        Self { active, total, guidance }
    }
}

/// Shared NDC geometry for the bottom-center strip (`[0: 3D Intercom Avatar] + [1..N: Hull Cameras]`).
#[derive(Clone, Copy, Debug)]
pub struct PreviewStripLayout {
    slot_count: u8,
    inv_aspect: f32,
}

impl PreviewStripLayout {
    pub const MAX_CAMERAS: u8 = 8;
    pub const MAX_SLOTS: u8 = Self::MAX_CAMERAS + 1;
    pub const TILE_ASPECT: f32 = 1.62;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_parts(slot_count: u8, inv_aspect: f32) -> Self {
        Self { slot_count, inv_aspect }
    }

    pub fn new(slot_count: u8, aspect: f32) -> Self {
        Self::from_parts(slot_count.min(Self::MAX_SLOTS), (1.0 / aspect.max(0.5)).clamp(0.25, 1.5))
    }

    pub fn slot_bounds(self, slot: u8) -> (f32, f32, f32, f32) {
        let ux = self.inv_aspect;
        let tile_h = 0.100;
        let tile_w = tile_h * Self::TILE_ASPECT * ux;
        let gap = 0.022 * ux;
        let n = f32::from(self.slot_count.max(1));
        let bar_w = n * tile_w + (n - 1.0).max(0.0) * gap;
        let x0 = -bar_w * 0.5 + f32::from(slot) * (tile_w + gap);
        let bar_y = -0.888;
        (x0, bar_y - tile_h * 0.5, x0 + tile_w, bar_y + tile_h * 0.5)
    }
}

/// Intercom avatar frame in Slot 0, status badge in active camera slot, and hollow frames around coarse previews.
#[derive(Clone, Copy, Debug)]
pub struct ViewfinderHud {
    feed: CameraFeed,
    inv_aspect: f32,
}

impl ViewfinderHud {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_parts(feed: CameraFeed, inv_aspect: f32) -> Self {
        Self { feed, inv_aspect }
    }

    pub fn new(feed: CameraFeed, aspect: f32) -> Self {
        Self::from_parts(feed, (1.0 / aspect.max(0.5)).clamp(0.25, 1.5))
    }

    pub fn emit(self, target: &mut dyn MeshTarget) {
        if self.feed.total == 0 {
            return;
        }
        let ux = self.inv_aspect;
        let total_slots = self.feed.total.min(PreviewStripLayout::MAX_CAMERAS) + 1;
        let layout = PreviewStripLayout::from_parts(total_slots, ux);
        let active_slot = self.feed.active.slot_index();

        self.emit_intercom_frame(layout.slot_bounds(0), active_slot == 0, target);
        for slot in 1..total_slots {
            let bounds = layout.slot_bounds(slot);
            if slot == active_slot {
                self.emit_active_camera_badge(bounds, target);
            } else {
                self.emit_inactive_window(bounds, Vec3::new(0.26, 0.36, 0.48), target);
            }
        }
        if matches!(self.feed.active, ActiveChannel::Camera(_, OpticMount::RocketNose)) {
            self.emit_nose_guidance_hud(target);
        }
    }

    fn emit_nose_guidance_hud(self, target: &mut dyn MeshTarget) {
        let engaged = match self.feed.guidance {
            GuidanceReadout::Absent => return,
            GuidanceReadout::Engaged => true,
            GuidanceReadout::Manual => false,
        };
        let ux = self.inv_aspect;
        let (x0, y0, x1, y1) = (-0.345 * ux, 0.852, 0.345 * ux, 0.932);
        let cy = (y0 + y1) * 0.5;
        let (rim, lamp, lamp_rim, text_col, label) = if engaged {
            (
                Vec3::new(0.24, 0.94, 0.68),
                Vec3::new(0.18, 0.98, 0.56),
                Vec3::new(0.05, 0.24, 0.14),
                Vec3::new(0.66, 1.00, 0.84),
                "FLUGCOMPUTER: EIN",
            )
        } else {
            (
                Vec3::new(0.98, 0.46, 0.18),
                Vec3::new(0.98, 0.26, 0.18),
                Vec3::new(0.26, 0.06, 0.05),
                Vec3::new(1.00, 0.70, 0.34),
                "FLUGCOMPUTER: AUS",
            )
        };
        ScreenBounds::new(x0, y0, x1, y1).emit_fill(Vec3::new(0.04, 0.09, 0.15), target);
        ScreenBounds::new(x0, y0, x1, y1).emit_outline(rim, 0.0042 * ux, 0.0042, target);
        let dot_x = x0 + 0.036 * ux;
        ScreenBounds::new(dot_x - 0.011 * ux, cy - 0.011, dot_x + 0.011 * ux, cy + 0.011)
            .emit_framed(lamp, lamp_rim, 0.0020, target);
        PixelStamp::new(0.0062 * ux, 0.0086, text_col).emit_centered(label, 0.020 * ux, cy, target);
    }

    fn emit_intercom_frame(
        self,
        (x0, y0, x1, y1): (f32, f32, f32, f32),
        selected: bool,
        target: &mut dyn MeshTarget,
    ) {
        if !selected {
            self.emit_inactive_window((x0, y0, x1, y1), Vec3::new(0.24, 0.44, 0.56), target);
            return;
        }
        let ux = self.inv_aspect;
        let (bx, by) = (0.0048 * ux, 0.0048);
        ScreenBounds::new(x0, y0, x1, y1).emit_outline(Vec3::new(0.76, 0.94, 1.00), bx, by, target);
        ScreenBounds::new(x0 - bx, y0 - by, x1 + bx, y1 + by).emit_outline(
            Vec3::new(0.04, 0.08, 0.14),
            0.0024 * ux,
            0.0024,
            target,
        );
        ScreenBounds::new(x0, y0 - by - 0.007, x1, y0 - by - 0.002)
            .emit_fill(Vec3::new(0.98, 0.84, 0.28), target);
    }

    fn emit_inactive_window(
        self,
        (x0, y0, x1, y1): (f32, f32, f32, f32),
        rim_color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let ux = self.inv_aspect;
        let (bx, by) = (0.0028 * ux, 0.0028);
        ScreenBounds::new(x0, y0, x1, y1).emit_outline(rim_color, bx, by, target);
        ScreenBounds::new(x0 - bx, y0 - by, x1 + bx, y1 + by).emit_outline(
            Vec3::new(0.03, 0.05, 0.09),
            0.0018 * ux,
            0.0018,
            target,
        );
    }

    fn emit_active_camera_badge(self, (x0, y0, x1, y1): (f32, f32, f32, f32), target: &mut dyn MeshTarget) {
        let ActiveChannel::Camera(index, mount) = self.feed.active else {
            return;
        };
        let ux = self.inv_aspect;
        let inner = ScreenBounds::new(x0, y0, x1, y1);
        let (bx, by) = (0.0048 * ux, 0.0048);
        let (cx, cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);

        inner.emit_fill(Vec3::new(0.05, 0.11, 0.18), target);
        inner.emit_outline(Vec3::new(0.38, 0.94, 1.00), bx, by, target);
        ScreenBounds::new(x0 - bx, y0 - by, x1 + bx, y1 + by).emit_outline(
            Vec3::new(0.04, 0.08, 0.14),
            0.0024 * ux,
            0.0024,
            target,
        );
        ScreenBounds::new(x0, y0 - by - 0.007, x1, y0 - by - 0.002)
            .emit_fill(Vec3::new(0.98, 0.84, 0.28), target);

        let top_y = cy + 0.020;
        let dot_x = x0 + 0.025 * ux;
        ScreenBounds::new(dot_x - 0.007 * ux, top_y - 0.007, dot_x + 0.007 * ux, top_y + 0.007)
            .emit_framed(Vec3::new(0.98, 0.22, 0.20), Vec3::new(0.24, 0.05, 0.06), 0.0016, target);
        let cam_label = format!("CAM {index}");
        PixelStamp::new(0.0052 * ux, 0.0064, Vec3::new(0.76, 0.90, 0.98))
            .emit_centered(&cam_label, cx + 0.015 * ux, top_y, target);
        PixelStamp::new(0.0056 * ux, 0.0072, Vec3::new(0.42, 0.94, 1.00))
            .emit_centered(mount.label(), cx, cy - 0.019, target);
    }
}
