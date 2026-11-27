use glam::{Mat4, Vec3, Vec4};
use engine::{
    FramePass, GraphicPipeline, Mesh, PassDraft, PointLightCluster, QuadBatch, WgpuStage,
    MONITOR_TILE_H, MONITOR_TILE_W,
};
use crate::menu::PreviewStripLayout;

const MONITOR_SLOTS: u8 = 8;

/// Strict Round-Robin selector advancing AT MOST `1` `240 × 160` phosphor monitor per main frame (`2` private fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundRobinGate {
    cursor: u8,
    primed_mask: u8,
}

impl RoundRobinGate {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cursor: u8, primed_mask: u8) -> Self {
        Self { cursor, primed_mask }
    }

    pub const fn dormant() -> Self {
        Self::new(0, 0)
    }

    #[cfg(test)]
    pub const fn tile_dimensions(self) -> (u32, u32) {
        let _ = self;
        (MONITOR_TILE_W, MONITOR_TILE_H)
    }

    pub const fn slot_primed(self, cam_idx: u8) -> bool {
        ((self.primed_mask >> (cam_idx & 7)) & 1) != 0
    }

    /// Selects AT MOST `1` monitor index (`0..cam_count`) to render on this frame, skipping `active_slot`.
    pub fn step_one(&mut self, cam_count: u8, active_slot: u8) -> Option<u8> {
        let capped = cam_count.min(PreviewStripLayout::MAX_CAMERAS).min(MONITOR_SLOTS);
        if capped == 0 {
            *self = Self::dormant();
            return None;
        }
        let valid_mask = if capped >= 8 { u8::MAX } else { (1u8 << capped) - 1 };
        self.primed_mask &= valid_mask;

        for step in 0..capped {
            let cand = (self.cursor.wrapping_add(step)) % capped;
            if cand + 1 != active_slot && !self.slot_primed(cand) {
                self.primed_mask |= 1u8 << cand;
                self.cursor = (cand + 1) % capped;
                return Some(cand);
            }
        }

        let mut chosen = self.cursor % capped;
        if capped > 1 && chosen + 1 == active_slot {
            chosen = (chosen + 1) % capped;
        }
        self.cursor = (chosen + 1) % capped;
        if chosen + 1 == active_slot {
            None
        } else {
            self.primed_mask |= 1u8 << chosen;
            Some(chosen)
        }
    }
}

/// `wgpu` diegetic `240 × 160` phosphor flight monitor strip (`NoseFlightHud`, `BellyFlightHud`, Docking-Cam, `2` fields).
pub struct PreviewAtlas {
    gate: RoundRobinGate,
    draft: PassDraft,
}

impl PreviewAtlas {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(gate: RoundRobinGate, draft: PassDraft) -> Self {
        Self { gate, draft }
    }

    pub fn dormant() -> Self {
        Self::new(RoundRobinGate::dormant(), PassDraft::empty())
    }

    pub fn render_and_blit(
        &mut self,
        stage: &WgpuStage,
        target_view: &wgpu::TextureView,
        pipeline: &dyn GraphicPipeline,
        coarse_mesh: &dyn Mesh,
        (cam_views, active_slot, viewer_up): (&[Mat4], u8, (Vec3, Vec3)),
        (intercom, lights, viewport): (Option<(Mat4, &QuadBatch)>, &PointLightCluster, (u32, u32)),
    ) {
        let cam_count = cam_views.len().min(PreviewStripLayout::MAX_CAMERAS as usize) as u8;
        if cam_count == 0 {
            self.gate = RoundRobinGate::dormant();
            return;
        }
        let (win_w, win_h) = (viewport.0.max(1), viewport.1.max(1));
        let aspect = (win_w as f32) / (win_h as f32);
        let layout = PreviewStripLayout::new(cam_count + 1, aspect);

        // Strict `max. 1 Monitor pro Frame` (`240 x 160`, shadow-free):
        if let Some(cam_idx) = self.gate.step_one(cam_count, active_slot) {
            self.draft.reset();
            pipeline.render(&mut self.draft, coarse_mesh, &cam_views[cam_idx as usize]);
            stage.submit_monitor_tile(u32::from(cam_idx), &mut self.draft, viewer_up, lights);
        }

        let inv_slots = 1.0 / (MONITOR_SLOTS as f32);
        let eps = 0.5 / ((MONITOR_TILE_W * u32::from(MONITOR_SLOTS)) as f32);
        let mut tiles: Vec<([f32; 4], [f32; 2], f32)> = Vec::with_capacity(cam_count as usize);
        for cam_idx in 0..cam_count {
            let slot = cam_idx + 1;
            if slot == active_slot || !self.gate.slot_primed(cam_idx) {
                continue;
            }
            let (x0, y0, x1, y1) = layout.slot_bounds(slot);
            let u0 = f32::from(cam_idx) * inv_slots + eps;
            let u1 = f32::from(cam_idx + 1) * inv_slots - eps;
            tiles.push(([x0, y0, x1, y1], [u0, u1], 0.34));
        }
        stage.blit_monitor_strip(target_view, &tiles);

        if let Some((bust_vp, bust_batch)) = intercom {
            let (x0, y0, x1, y1) = layout.slot_bounds(0);
            let px0 = (((x0 + 1.0) * 0.5) * (win_w as f32)).round().max(0.0) as u32;
            let px1 = (((x1 + 1.0) * 0.5) * (win_w as f32)).round().max(0.0) as u32;
            let py_top = (((1.0 - y1) * 0.5) * (win_h as f32)).round().max(0.0) as u32;
            let py_bot = (((1.0 - y0) * 0.5) * (win_h as f32)).round().max(0.0) as u32;
            let pw = px1.saturating_sub(px0).max(1).min(win_w.saturating_sub(px0).max(1));
            let ph = py_bot.saturating_sub(py_top).max(1).min(win_h.saturating_sub(py_top).max(1));

            self.draft.reset();
            self.draft.clear_depth_layer();
            self.draft.clear_sky(Vec4::new(0.085, 0.125, 0.175, 1.0));
            self.draft.bind_camera(bust_vp, Vec3::new(0.46, 0.52, 0.72).normalize());
            self.draft.stream_quads(bust_batch);
            stage.submit_overlay_draft(target_view, Some([px0, py_top, pw, ph]), &mut self.draft, lights);
        }
    }
}
