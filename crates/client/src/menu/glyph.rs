use glam::Vec3;
use world::{MeshTarget, Quad};

/// Encapsulated 3x5 pixel glyph stamp emitting crisp 2D orthographic quads into a MeshTarget.
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

    const fn mask(r0: u16, r1: u16, r2: u16, r3: u16, r4: u16) -> u16 {
        (r0 << 12) | (r1 << 9) | (r2 << 6) | (r3 << 3) | r4
    }

    fn glyph_bits(ch: char) -> u16 {
        match ch.to_ascii_uppercase() {
            'A' => Self::mask(2, 5, 7, 5, 5),
            'B' => Self::mask(6, 5, 6, 5, 6),
            'C' => Self::mask(3, 4, 4, 4, 3),
            'D' => Self::mask(6, 5, 5, 5, 6),
            'E' => Self::mask(7, 4, 6, 4, 7),
            'F' => Self::mask(7, 4, 6, 4, 4),
            'G' => Self::mask(3, 4, 5, 5, 3),
            'H' => Self::mask(5, 5, 7, 5, 5),
            'I' => Self::mask(7, 2, 2, 2, 7),
            'J' => Self::mask(1, 1, 1, 5, 2),
            'K' => Self::mask(5, 5, 6, 5, 5),
            'L' => Self::mask(4, 4, 4, 4, 7),
            'M' => Self::mask(5, 7, 7, 5, 5),
            'N' => Self::mask(5, 7, 7, 7, 5),
            'O' | '0' => Self::mask(2, 5, 5, 5, 2),
            'P' => Self::mask(6, 5, 6, 4, 4),
            'Q' => Self::mask(2, 5, 5, 6, 3),
            'R' => Self::mask(6, 5, 6, 5, 5),
            'S' | '5' => Self::mask(3, 4, 2, 1, 6),
            'T' => Self::mask(7, 2, 2, 2, 2),
            'U' => Self::mask(5, 5, 5, 5, 7),
            'V' => Self::mask(5, 5, 5, 5, 2),
            'W' => Self::mask(5, 5, 7, 7, 5),
            'X' => Self::mask(5, 5, 2, 5, 5),
            'Y' => Self::mask(5, 5, 2, 2, 2),
            'Z' => Self::mask(7, 1, 2, 4, 7),
            '1' => Self::mask(2, 6, 2, 2, 7),
            '2' => Self::mask(6, 1, 2, 4, 7),
            '3' => Self::mask(6, 1, 2, 1, 6),
            '4' => Self::mask(5, 5, 7, 1, 1),
            '6' => Self::mask(3, 4, 6, 5, 2),
            '7' => Self::mask(7, 1, 2, 2, 2),
            '8' => Self::mask(2, 5, 2, 5, 2),
            '9' => Self::mask(2, 5, 3, 1, 6),
            '+' => Self::mask(0, 2, 7, 2, 0),
            '-' => Self::mask(0, 0, 7, 0, 0),
            '<' => Self::mask(1, 2, 4, 2, 1),
            '>' => Self::mask(4, 2, 1, 2, 4),
            ':' => Self::mask(0, 2, 0, 2, 0),
            '.' => Self::mask(0, 0, 0, 0, 2),
            '/' => Self::mask(1, 1, 2, 4, 4),
            _ => 0,
        }
    }

    /// Emits centered text string at (center_x, center_y) in Normalized Device Coordinates.
    pub fn emit_centered(&self, text: &str, center_x: f32, center_y: f32, target: &mut dyn MeshTarget) {
        let count = text.chars().count() as f32;
        if count <= 0.0 {
            return;
        }
        let step_x = self.cell_w * 4.0;
        let total_w = count * step_x - self.cell_w;
        let start_x = center_x - total_w * 0.5;
        let top_y = center_y + (self.cell_h * 5.0) * 0.5;

        for (idx, ch) in text.chars().enumerate() {
            let bits = Self::glyph_bits(ch);
            if bits == 0 {
                continue;
            }
            let gx = start_x + (idx as f32) * step_x;
            for row in 0..5 {
                let row_bits = (bits >> ((4 - row) * 3)) & 0b111;
                let y1 = top_y - (row as f32) * self.cell_h;
                let y0 = y1 - self.cell_h;
                for col in 0..3 {
                    if (row_bits & (1 << (2 - col))) != 0 {
                        let x0 = gx + (col as f32) * self.cell_w;
                        let x1 = x0 + self.cell_w;
                        target.push_quad(Quad::new(
                            Vec3::new(x0, y0, 0.0),
                            Vec3::new(x1, y0, 0.0),
                            Vec3::new(x1, y1, 0.0),
                            Vec3::new(x0, y1, 0.0),
                            Vec3::Z,
                            self.color,
                        ));
                    }
                }
            }
        }
    }
}
