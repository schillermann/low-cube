use crate::packed_quad::PackedQuad;

/// `8 × 8` binary greedy 2D rectangle merger emitting `8-Byte` `PackedQuad`s via `u8` bit-scans (`3` private fields).
pub struct SliceBitMerge {
    active_rows: [u8; 8],
    signatures: [[u32; 8]; 8],
    normal_idx: u8,
}

impl SliceBitMerge {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(active_rows: [u8; 8], signatures: [[u32; 8]; 8], normal_idx: u8) -> Self {
        Self { active_rows, signatures, normal_idx }
    }

    /// Greedily merges identical `(rgb565, surface_tag, ao_pack)` cells across `(su, sv)` into maximal rectangles.
    pub fn emit_merged(mut self, slice_d: u8, out: &mut Vec<PackedQuad>) {
        for sv in 0..8usize {
            while self.active_rows[sv] != 0 {
                let su = self.active_rows[sv].trailing_zeros() as usize;
                let sig = self.signatures[sv][su];
                let mut w = 1usize;
                while su + w < 8
                    && ((self.active_rows[sv] >> (su + w)) & 1) != 0
                    && self.signatures[sv][su + w] == sig
                {
                    w += 1;
                }
                let run_mask: u8 = (((1u16 << w) - 1) as u8) << su;
                self.active_rows[sv] &= !run_mask;

                let mut h = 1usize;
                while sv + h < 8 && (self.active_rows[sv + h] & run_mask) == run_mask {
                    let mut row_matches = true;
                    let mut k = 0usize;
                    while k < w {
                        if self.signatures[sv + h][su + k] != sig {
                            row_matches = false;
                            break;
                        }
                        k += 1;
                    }
                    if !row_matches {
                        break;
                    }
                    self.active_rows[sv + h] &= !run_mask;
                    h += 1;
                }

                let coords = self.face_coords(slice_d, su as u8, sv as u8);
                let ao_pack = (sig & 0xFF) as u8;
                let tag = ((sig >> 8) & 7) as u8;
                let rgb565 = ((sig >> 16) & 0xFFFF) as u16;
                out.push(PackedQuad::encoded(
                    coords,
                    self.normal_idx,
                    [(w - 1) as u8, (h - 1) as u8],
                    ao_pack,
                    tag,
                    rgb565,
                ));
            }
        }
    }

    const fn face_coords(&self, d: u8, su: u8, sv: u8) -> [u8; 3] {
        match self.normal_idx {
            0 => [d + 1, su, sv],
            1 => [d, sv, su],
            2 => [sv, d + 1, su],
            3 => [su, d, sv],
            4 => [su, sv, d + 1],
            _ => [sv, su, d],
        }
    }
}
