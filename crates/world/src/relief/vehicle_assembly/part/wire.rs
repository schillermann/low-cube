use crate::block::{BlockMold, VoxelShape};

/// Binary wire codec, canonical sort key, and FNV-1a digest step for a single `VehiclePart`.
pub(super) struct PartWire {
    offset: (i32, i32),
    floor_offset: f32,
    stamp: [u8; 3],
}

impl PartWire {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(offset: (i32, i32), floor_offset: f32, stamp: [u8; 3]) -> Self {
        Self { offset, floor_offset, stamp }
    }

    pub(super) fn inscribe(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.offset.0.clamp(-128, 128) as i16).to_le_bytes());
        out.extend_from_slice(&(self.offset.1.clamp(-128, 128) as i16).to_le_bytes());
        out.extend_from_slice(&self.floor_offset.to_le_bytes());
        out.extend_from_slice(&self.stamp);
    }

    pub(super) fn decode(bytes: &[u8]) -> Option<((i32, i32), f32, Box<dyn VoxelShape>, usize)> {
        if bytes.len() < 11 {
            return None;
        }
        let ox = i16::from_le_bytes(bytes[0..2].try_into().ok()?) as i32;
        let oy = i16::from_le_bytes(bytes[2..4].try_into().ok()?) as i32;
        let floor_offset = f32::from_le_bytes(bytes[4..8].try_into().ok()?);
        if !floor_offset.is_finite() || ox.abs() > 128 || oy.abs() > 128 || floor_offset.abs() > 32.0 {
            return None;
        }
        let stamp = [bytes[8], bytes[9], bytes[10]];
        Some(((ox, oy), floor_offset, BlockMold::shape_from_stamp(stamp), 11))
    }

    pub(super) fn canonical_key(&self) -> (i32, i32, i32, [u8; 3]) {
        (
            self.offset.0,
            self.offset.1,
            (self.floor_offset * 16.0).round() as i32,
            self.stamp,
        )
    }

    pub(super) fn digest_mix(&self, mut h: u64) -> u64 {
        for b in (self.offset.0 as i16).to_le_bytes() {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        for b in (self.offset.1 as i16).to_le_bytes() {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        for b in ((self.floor_offset * 16.0).round() as i16).to_le_bytes() {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        for b in self.stamp {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        h
    }
}
