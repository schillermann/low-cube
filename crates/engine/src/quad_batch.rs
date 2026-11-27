use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::packed_quad::{FreeQuad, PackedQuad, Rgb565Swatch};

const QUANT_STEP: f32 = 0.25;
const QUANT_EPS: f32 = 0.004;
const FULL_BRIGHT_AO: u8 = 0xFF;

/// Quantizes axis-aligned `0.25 m`-grid `Quad`s into `8-Byte` `PackedQuad`s (`1` private field).
#[derive(Clone, Copy, Debug)]
pub struct AxisQuadPack {
    origin: [f32; 4],
}

impl AxisQuadPack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: [f32; 4]) -> Self {
        Self { origin }
    }

    fn classify_normal(self, n: Vec3) -> Option<u8> {
        let _ = self;
        let len_sq = n.length_squared();
        if !(0.96..=1.04).contains(&len_sq) {
            return None;
        }
        if n.x > 0.995 {
            Some(0)
        } else if n.x < -0.995 {
            Some(1)
        } else if n.y > 0.995 {
            Some(2)
        } else if n.y < -0.995 {
            Some(3)
        } else if n.z > 0.995 {
            Some(4)
        } else if n.z < -0.995 {
            Some(5)
        } else {
            None
        }
    }

    fn quantize_coord(self, val: f32, base: f32) -> Option<u8> {
        let rel = (val - base) / self.origin[3];
        let rounded = rel.round();
        if (rel - rounded).abs() > QUANT_EPS || !(0.0..=255.0).contains(&rounded) {
            None
        } else {
            Some(rounded as u8)
        }
    }

    fn quantize_span(self, delta: f32) -> Option<u8> {
        let steps = delta / self.origin[3];
        let rounded = steps.round();
        if (steps - rounded).abs() > QUANT_EPS || !(1.0..=32.0).contains(&rounded) {
            None
        } else {
            Some((rounded as u8) - 1)
        }
    }

    /// Attempts lossless `8-Byte` packing for axis-aligned grid quads within this patch's `63.75 m³` frame.
    pub fn try_pack(self, verts: [Vec3; 4], normal: Vec3, color: Vec3, glow: f32) -> Option<PackedQuad> {
        if !(0.0..1.98).contains(&glow) {
            return None;
        }
        let normal_idx = self.classify_normal(normal)?;
        let min_v = verts[0].min(verts[1]).min(verts[2]).min(verts[3]);
        let max_v = verts[0].max(verts[1]).max(verts[2]).max(verts[3]);
        let ext = max_v - min_v;
        for v in verts {
            let on_min = (v - min_v).abs();
            let on_max = (v - max_v).abs();
            if on_min.x.min(on_max.x) > QUANT_EPS
                || on_min.y.min(on_max.y) > QUANT_EPS
                || on_min.z.min(on_max.z) > QUANT_EPS
            {
                return None;
            }
        }
        let (flat_err, span_u, span_v) = match normal_idx {
            0 => (ext.x, ext.y, ext.z),
            1 => (ext.x, ext.z, ext.y),
            2 => (ext.y, ext.z, ext.x),
            3 => (ext.y, ext.x, ext.z),
            4 => (ext.z, ext.x, ext.y),
            _ => (ext.z, ext.y, ext.x),
        };
        if flat_err > QUANT_EPS {
            return None;
        }
        let qx = self.quantize_coord(min_v.x, self.origin[0])?;
        let qy = self.quantize_coord(min_v.y, self.origin[1])?;
        let qz = self.quantize_coord(min_v.z, self.origin[2])?;
        let wu = self.quantize_span(span_u)?;
        let hv = self.quantize_span(span_v)?;
        let tag = if glow > 0.005 && glow < 0.05 {
            1
        } else if glow >= 0.85 {
            3
        } else if glow >= 0.20 {
            2
        } else {
            0
        };
        let swatch = Rgb565Swatch::from_linear(color).code();
        Some(PackedQuad::encoded([qx, qy, qz], normal_idx, [wu, hv], FULL_BRIGHT_AO, tag, swatch))
    }
}

/// Unified vertex-pulled quad collector (`8-Byte` `PackedQuad`s + index-free `FreeQuad`s, `3` private fields).
#[derive(Clone, Debug)]
pub struct QuadBatch {
    packed: Vec<PackedQuad>,
    free: Vec<FreeQuad>,
    origin: [f32; 4],
}

impl QuadBatch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(packed: Vec<PackedQuad>, free: Vec<FreeQuad>, origin: [f32; 4]) -> Self {
        Self { packed, free, origin }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new(), [f32::NAN, 0.0, 0.0, QUANT_STEP])
    }

    pub fn anchored_at(origin_xyz: Vec3, step: f32) -> Self {
        Self::new(Vec::new(), Vec::new(), [origin_xyz.x, origin_xyz.y, origin_xyz.z, step])
    }

    pub fn with_quad_capacity(quads: usize) -> Self {
        Self::new(
            Vec::with_capacity(quads),
            Vec::with_capacity(quads / 4),
            [f32::NAN, 0.0, 0.0, QUANT_STEP],
        )
    }

    pub fn clear(&mut self) {
        self.packed.clear();
        self.free.clear();
        self.origin = [f32::NAN, 0.0, 0.0, QUANT_STEP];
    }

    pub fn push_packed(&mut self, quad: PackedQuad) {
        self.packed.push(quad);
    }

    pub fn packed_slice(&self) -> &[PackedQuad] {
        &self.packed
    }

    pub fn free_slice(&self) -> &[FreeQuad] {
        &self.free
    }

    pub const fn patch_origin(&self) -> [f32; 4] {
        if self.origin[0].is_nan() {
            [0.0, 0.0, 0.0, QUANT_STEP]
        } else {
            self.origin
        }
    }

    pub fn vacant(&self) -> bool {
        self.packed.is_empty() && self.free.is_empty()
    }

    pub fn total_quad_count(&self) -> usize {
        self.packed.len() + self.free.len()
    }

    pub fn packed_bytes(&self) -> &[u8] {
        if self.packed.is_empty() {
            &[]
        } else {
            // Sound: `PackedQuad` is `#[repr(C)]` of `[u32; 2]` (`8` initialized bytes, no padding).
            unsafe {
                core::slice::from_raw_parts(
                    self.packed.as_ptr().cast::<u8>(),
                    self.packed.len() * core::mem::size_of::<PackedQuad>(),
                )
            }
        }
    }

    pub fn free_bytes(&self) -> &[u8] {
        if self.free.is_empty() {
            &[]
        } else {
            // Sound: `FreeQuad` is `#[repr(C)]` of `20 × f32` (`80` initialized bytes, no padding).
            unsafe {
                core::slice::from_raw_parts(
                    self.free.as_ptr().cast::<u8>(),
                    self.free.len() * core::mem::size_of::<FreeQuad>(),
                )
            }
        }
    }
}

impl MeshTarget for QuadBatch {
    fn push_quad(&mut self, quad: Quad) {
        let glow = quad.radiate(|g| g);
        quad.transfer(|v0, v1, v2, v3, normal, color| {
            let verts = [v0, v1, v2, v3];
            if self.origin[0].is_nan() && (0.0..1.98).contains(&glow) {
                let min_v = v0.min(v1).min(v2).min(v3);
                let snapped = (min_v / QUANT_STEP).floor() * QUANT_STEP - Vec3::splat(16.0);
                self.origin = [snapped.x, snapped.y, snapped.z, QUANT_STEP];
            }
            if !self.origin[0].is_nan() {
                if let Some(packed) = AxisQuadPack::new(self.origin).try_pack(verts, normal, color, glow) {
                    self.packed.push(packed);
                    return;
                }
            }
            self.free.push(FreeQuad::from_vertices(verts, normal, color, glow));
        });
    }
}
