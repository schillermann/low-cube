use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::attire::LimbFrame;

/// Encapsulated 5x7 pixel glyph stamp emitting crisp 2D orthographic or 3D frame quads into a MeshTarget.
#[derive(Clone, Copy, Debug)]
pub struct PixelStamp {
    cell_w: f32,
    cell_h: f32,
    color: Vec3,
}

impl PixelStamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell_w: f32, cell_h: f32, color: Vec3) -> Self {
        Self { cell_w, cell_h, color }
    }

    pub const fn caption(color: Vec3) -> Self {
        Self::new(0.0052, 0.0086, color)
    }

    pub const fn title(color: Vec3) -> Self {
        Self::new(0.0074, 0.0124, color)
    }

    const fn mask(r0: u64, r1: u64, r2: u64, r3: u64, r4: u64, r5: u64, r6: u64) -> u64 {
        (r0 << 30) | (r1 << 25) | (r2 << 20) | (r3 << 15) | (r4 << 10) | (r5 << 5) | r6
    }

    fn glyph_bits(ch: char) -> u64 {
        match ch.to_ascii_uppercase() {
            'A' => Self::mask(14, 17, 17, 31, 17, 17, 17),
            'B' => Self::mask(30, 17, 17, 30, 17, 17, 30),
            'C' => Self::mask(14, 17, 16, 16, 16, 17, 14),
            'D' => Self::mask(28, 18, 17, 17, 17, 18, 28),
            'E' => Self::mask(31, 16, 16, 30, 16, 16, 31),
            'F' => Self::mask(31, 16, 16, 30, 16, 16, 16),
            'G' => Self::mask(14, 17, 16, 23, 17, 17, 15),
            'H' => Self::mask(17, 17, 17, 31, 17, 17, 17),
            'I' => Self::mask(14, 4, 4, 4, 4, 4, 14),
            'J' => Self::mask(7, 2, 2, 2, 18, 18, 12),
            'K' => Self::mask(17, 18, 20, 24, 20, 18, 17),
            'L' => Self::mask(16, 16, 16, 16, 16, 16, 31),
            'M' => Self::mask(17, 27, 21, 21, 17, 17, 17),
            'N' => Self::mask(17, 25, 21, 19, 17, 17, 17),
            'O' => Self::mask(14, 17, 17, 17, 17, 17, 14),
            'P' => Self::mask(30, 17, 17, 30, 16, 16, 16),
            'Q' => Self::mask(14, 17, 17, 17, 21, 18, 13),
            'R' => Self::mask(30, 17, 17, 30, 20, 18, 17),
            'S' => Self::mask(14, 17, 16, 14, 1, 17, 14),
            'T' => Self::mask(31, 4, 4, 4, 4, 4, 4),
            'U' => Self::mask(17, 17, 17, 17, 17, 17, 14),
            'V' => Self::mask(17, 17, 17, 17, 17, 10, 4),
            'W' => Self::mask(17, 17, 17, 21, 21, 27, 17),
            'X' => Self::mask(17, 17, 10, 4, 10, 17, 17),
            'Y' => Self::mask(17, 17, 10, 4, 4, 4, 4),
            'Z' => Self::mask(31, 1, 2, 4, 8, 16, 31),
            '0' => Self::mask(14, 17, 19, 21, 25, 17, 14),
            '1' => Self::mask(4, 12, 20, 4, 4, 4, 31),
            '2' => Self::mask(14, 17, 1, 6, 8, 16, 31),
            '3' => Self::mask(30, 1, 2, 6, 1, 17, 14),
            '4' => Self::mask(2, 6, 10, 18, 31, 2, 2),
            '5' => Self::mask(31, 16, 30, 1, 1, 17, 14),
            '6' => Self::mask(6, 8, 16, 30, 17, 17, 14),
            '7' => Self::mask(31, 1, 2, 4, 8, 8, 8),
            '8' => Self::mask(14, 17, 17, 14, 17, 17, 14),
            '9' => Self::mask(14, 17, 17, 15, 1, 2, 12),
            '+' => Self::mask(0, 4, 4, 31, 4, 4, 0),
            '-' => Self::mask(0, 0, 0, 31, 0, 0, 0),
            '_' => Self::mask(0, 0, 0, 0, 0, 0, 31),
            '|' => Self::mask(4, 4, 4, 4, 4, 4, 4),
            '(' => Self::mask(2, 4, 8, 8, 8, 4, 2),
            ')' => Self::mask(8, 4, 2, 2, 2, 4, 8),
            '<' => Self::mask(2, 4, 8, 16, 8, 4, 2),
            '>' => Self::mask(8, 4, 2, 1, 2, 4, 8),
            ':' => Self::mask(0, 4, 4, 0, 4, 4, 0),
            '.' => Self::mask(0, 0, 0, 0, 0, 12, 12),
            '/' => Self::mask(1, 2, 2, 4, 8, 8, 16),
            '@' => Self::mask(14, 17, 23, 21, 23, 16, 14),
            _ => 0,
        }
    }

    /// Emits centered text string at (center_x, center_y) in Normalized Device Coordinates.
    pub fn emit_centered(&self, text: &str, center_x: f32, center_y: f32, target: &mut dyn MeshTarget) {
        let ortho = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::NEG_Z);
        self.emit_on_frame(text, center_x, center_y, 0.0, &ortho, target);
    }

    /// Emits centered text string onto a local 3D LimbFrame plane at depth z.
    pub fn emit_on_frame(&self, text: &str, center_x: f32, center_y: f32, z: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let count = text.chars().count() as f32;
        if count <= 0.0 {
            return;
        }
        let step_x = self.cell_w * 4.0;
        let px_w = self.cell_w * 0.6;
        let px_h = self.cell_h * (5.0 / 7.0);
        let total_w = count * step_x - self.cell_w;
        let start_x = center_x - total_w * 0.5;
        let top_y = center_y + (self.cell_h * 5.0) * 0.5;
        let normal = (frame.point(0.0, 0.0, -1.0) - frame.point(0.0, 0.0, 0.0)).normalize_or_zero();

        for (idx, ch) in text.chars().enumerate() {
            let bits = Self::glyph_bits(ch);
            if bits == 0 {
                continue;
            }
            let gx = start_x + (idx as f32) * step_x;
            for row in 0..7 {
                let row_bits = (bits >> ((6 - row) * 5)) & 0b1_1111;
                let y1 = top_y - (row as f32) * px_h;
                let y0 = y1 - px_h;
                for col in 0..5 {
                    if (row_bits & (1 << (4 - col))) != 0 {
                        let x0 = gx + (col as f32) * px_w;
                        let x1 = x0 + px_w;
                        target.push_quad(Quad::new(
                            frame.point(x0, y0, z),
                            frame.point(x1, y0, z),
                            frame.point(x1, y1, z),
                            frame.point(x0, y1, z),
                            normal,
                            self.color,
                        ));
                    }
                }
            }
        }
    }
}
