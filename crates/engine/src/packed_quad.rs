use glam::Vec3;

/// Immutable 16-bit `R5G6B5` color swatch (`1` private field, `Copy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb565Swatch {
    code: u16,
}

impl Rgb565Swatch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: u16) -> Self {
        Self { code }
    }

    /// Quantizes a linear `[0.0, 1.0]` RGB triplet into a 16-bit `R5G6B5` swatch.
    pub fn from_linear(color: Vec3) -> Self {
        let c = color.clamp(Vec3::ZERO, Vec3::ONE);
        let r = (c.x * 31.0 + 0.5) as u16;
        let g = (c.y * 63.0 + 0.5) as u16;
        let b = (c.z * 31.0 + 0.5) as u16;
        Self::new(((r & 31) << 11) | ((g & 63) << 5) | (b & 31))
    }

    pub const fn code(self) -> u16 {
        self.code
    }

    pub fn linear_rgb(self) -> Vec3 {
        let r = ((self.code >> 11) & 31) as f32 / 31.0;
        let g = ((self.code >> 5) & 63) as f32 / 63.0;
        let b = (self.code & 31) as f32 / 31.0;
        Vec3::new(r, g, b)
    }
}

/// 8-Byte (`64-Bit` / `2 × u32`) GPU vertex-pulled voxel face (`23×` smaller than `184 B` legacy vertex/index quads).
///
/// Bit layout:
/// - `words[0]`: `qx:8 | qy:8 | qz:8 | normal:3 | w_minus_1:5` (`32` bits)
/// - `words[1]`: `h_minus_1:5 | ao_corners:8 | surface_tag:3 | rgb565:16` (`32` bits)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PackedQuad {
    words: [u32; 2],
}

impl PackedQuad {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(words: [u32; 2]) -> Self {
        Self { words }
    }

    /// Encodes a merged greedy or quantized axis-aligned rectangle into `8` bytes.
    pub const fn encoded(
        coords: [u8; 3],
        normal_idx: u8,
        span_minus_1: [u8; 2],
        ao_pack: u8,
        surface_tag: u8,
        rgb565: u16,
    ) -> Self {
        let w0 = (coords[0] as u32)
            | ((coords[1] as u32) << 8)
            | ((coords[2] as u32) << 16)
            | (((normal_idx & 7) as u32) << 24)
            | (((span_minus_1[0] & 31) as u32) << 27);
        let w1 = ((span_minus_1[1] & 31) as u32)
            | ((ao_pack as u32) << 5)
            | (((surface_tag & 7) as u32) << 13)
            | ((rgb565 as u32) << 16);
        Self::new([w0, w1])
    }

    pub const fn raw_words(self) -> [u32; 2] {
        self.words
    }

    /// Tell-Don't-Ask bit-field inspection without procedural getters.
    pub fn transfer<R>(
        self,
        inspector: impl FnOnce([u8; 3], u8, [u8; 2], u8, u8, u16) -> R,
    ) -> R {
        let [w0, w1] = self.words;
        let coords = [(w0 & 0xFF) as u8, ((w0 >> 8) & 0xFF) as u8, ((w0 >> 16) & 0xFF) as u8];
        let normal_idx = ((w0 >> 24) & 7) as u8;
        let span_m1 = [((w0 >> 27) & 31) as u8, (w1 & 31) as u8];
        let ao_pack = ((w1 >> 5) & 0xFF) as u8;
        let surface_tag = ((w1 >> 13) & 7) as u8;
        let rgb565 = ((w1 >> 16) & 0xFFFF) as u16;
        inspector(coords, normal_idx, span_m1, ao_pack, surface_tag, rgb565)
    }

    /// Reconstructs the 4 world-space CCW corners `(v0, v1, v2, v3)` and outward normal for verification.
    pub fn reconstruct_corners(self, patch_origin: [f32; 4]) -> ([Vec3; 4], Vec3) {
        self.transfer(|coords, normal_idx, span_m1, _, _, _| {
            let step = patch_origin[3];
            let base = Vec3::new(
                patch_origin[0] + (coords[0] as f32) * step,
                patch_origin[1] + (coords[1] as f32) * step,
                patch_origin[2] + (coords[2] as f32) * step,
            );
            let su = ((span_m1[0] as f32) + 1.0) * step;
            let sv = ((span_m1[1] as f32) + 1.0) * step;
            let (au, av, n) = match normal_idx {
                0 => (Vec3::Y, Vec3::Z, Vec3::X),
                1 => (Vec3::Z, Vec3::Y, Vec3::NEG_X),
                2 => (Vec3::Z, Vec3::X, Vec3::Y),
                3 => (Vec3::X, Vec3::Z, Vec3::NEG_Y),
                4 => (Vec3::X, Vec3::Y, Vec3::Z),
                _ => (Vec3::Y, Vec3::X, Vec3::NEG_Z),
            };
            let du = au * su;
            let dv = av * sv;
            ([base, base + du, base + du + dv, base + dv], n)
        })
    }
}

/// Index-free storage-pulled quad (`5 × vec4<f32> = 80 B`) for tilted facets, EVA limbs, UI glyphs, and plumes (`2` fields).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FreeQuad {
    corners: [[f32; 4]; 4],
    color_pad: [f32; 4],
}

impl FreeQuad {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(corners: [[f32; 4]; 4], color_pad: [f32; 4]) -> Self {
        Self { corners, color_pad }
    }

    pub const fn from_vertices(
        verts: [Vec3; 4],
        normal: Vec3,
        color: Vec3,
        glow: f32,
    ) -> Self {
        Self::new(
            [
                [verts[0].x, verts[0].y, verts[0].z, normal.x],
                [verts[1].x, verts[1].y, verts[1].z, normal.y],
                [verts[2].x, verts[2].y, verts[2].z, normal.z],
                [verts[3].x, verts[3].y, verts[3].z, glow],
            ],
            [color.x, color.y, color.z, 0.0],
        )
    }
}
