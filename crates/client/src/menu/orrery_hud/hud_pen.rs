use std::f32::consts::TAU;
use glam::{Vec2, Vec3};
use world::{MeshTarget, Quad};
use crate::menu::glyph::PixelStamp;

/// Aspect-corrected 2D vector pen for spacecraft HUD rings, tethers, chevrons, and captions (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct HudPen {
    anchor: Vec2,
    inv_aspect: f32,
}

impl HudPen {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(anchor: Vec2, inv_aspect: f32) -> Self {
        Self { anchor, inv_aspect }
    }

    fn ux(self) -> f32 {
        self.inv_aspect.max(0.25)
    }

    pub(super) fn metric_delta(self, a: Vec2, b: Vec2) -> Vec2 {
        Vec2::new((b.x - a.x) / self.ux(), b.y - a.y)
    }

    pub fn step_ndc(self, origin: Vec2, metric_offset: Vec2) -> Vec2 {
        Vec2::new(origin.x + metric_offset.x * self.ux(), origin.y + metric_offset.y)
    }

    pub fn emit_beam(
        self,
        a: Vec2,
        b: Vec2,
        half_w: f32,
        color: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let ux = self.ux();
        let metric = Vec2::new((b.x - a.x) / ux, b.y - a.y);
        let len = metric.length();
        if len <= 1e-5 {
            return;
        }
        let dir = metric / len;
        let perp = Vec2::new(-dir.y * half_w * ux, dir.x * half_w);
        let n = if glow < 0.0 { Vec3::new(0.0, 1.0, 1.0) } else { Vec3::Z };
        target.push_quad(Quad::glowing(
            Vec3::new(a.x - perp.x, a.y - perp.y, 0.0),
            Vec3::new(b.x - perp.x, b.y - perp.y, 0.0),
            Vec3::new(b.x + perp.x, b.y + perp.y, 0.0),
            Vec3::new(a.x + perp.x, a.y + perp.y, 0.0),
            n,
            color,
            glow,
        ));
    }

    pub fn emit_dashed_beam(
        self,
        a: Vec2,
        b: Vec2,
        dashes: usize,
        half_w: f32,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let count = dashes.max(1);
        let denom = (count * 2 - 1) as f32;
        for i in 0..count {
            let t0 = ((i * 2) as f32) / denom;
            let t1 = ((i * 2 + 1) as f32) / denom;
            self.emit_beam(a.lerp(b, t0), a.lerp(b, t1), half_w, color, 0.96, target);
        }
    }

    pub fn emit_anchor_tether(
        self,
        target_ndc: Vec2,
        gaps: (f32, f32),
        dashes: usize,
        half_w: f32,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let delta = self.metric_delta(self.anchor, target_ndc);
        let span = delta.length();
        if span > gaps.0 + gaps.1 + 0.008 {
            let dir = delta / span;
            let a = self.step_ndc(self.anchor, dir * gaps.0);
            let b = self.step_ndc(target_ndc, -dir * gaps.1);
            self.emit_dashed_beam(a, b, dashes, half_w, color, target);
        }
    }

    pub fn emit_ring(
        self,
        center: Vec2,
        radius: f32,
        segments: usize,
        half_w: f32,
        color: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let n = segments.max(6);
        for i in 0..n {
            let a0 = (i as f32) * TAU / (n as f32);
            let a1 = ((i + 1) as f32) * TAU / (n as f32);
            let p0 = self.step_ndc(center, Vec2::new(a0.cos() * radius, a0.sin() * radius));
            let p1 = self.step_ndc(center, Vec2::new(a1.cos() * radius, a1.sin() * radius));
            self.emit_beam(p0, p1, half_w, color, glow, target);
        }
    }

    pub fn emit_anchor_ring(
        self,
        radius: f32,
        segments: usize,
        half_w: f32,
        color: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        self.emit_ring(self.anchor, radius, segments, half_w, color, glow, target);
    }

    pub fn emit_anchor_spoke(
        self,
        dir: Vec2,
        radii: (f32, f32),
        half_w: f32,
        color: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let a = self.step_ndc(self.anchor, dir * radii.0);
        let b = self.step_ndc(self.anchor, dir * radii.1);
        self.emit_beam(a, b, half_w, color, glow, target);
    }

    pub fn emit_anchor_notch_toward(
        self,
        ndc: Vec2,
        radii: (f32, f32),
        half_w: f32,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let delta = self.metric_delta(self.anchor, ndc);
        let len = delta.length();
        if len > 0.04 {
            self.emit_anchor_spoke(delta / len, radii, half_w, color, 0.96, target);
        }
    }

    pub fn emit_rim_chevron(
        self,
        rim_pt: Vec2,
        outward: Vec2,
        size: f32,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let side = Vec2::new(-outward.y, outward.x);
        let tip = self.step_ndc(rim_pt, outward * size);
        let w0 = self.step_ndc(rim_pt, -outward * (size * 0.35) + side * (size * 0.72));
        let w1 = self.step_ndc(rim_pt, -outward * (size * 0.35) - side * (size * 0.72));
        self.emit_beam(w0, tip, 0.0028, color, 0.98, target);
        self.emit_beam(w1, tip, 0.0028, color, 0.98, target);
    }

    pub fn emit_caption(
        self,
        text: &str,
        center: Vec2,
        glyph_size: (f32, f32),
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        PixelStamp::new(glyph_size.0 * self.ux(), glyph_size.1, color)
            .emit_centered(text, center.x, center.y, target);
    }

    pub fn emit_anchor_caption(
        self,
        text: &str,
        metric_offset: Vec2,
        glyph_size: (f32, f32),
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let pt = self.step_ndc(self.anchor, metric_offset);
        self.emit_caption(text, pt, glyph_size, color, target);
    }
}
