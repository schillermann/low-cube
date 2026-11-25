use crate::PalmMolds;

/// Network-replicated companion wardrobe, physique proportions, and dual-palm mold orientations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresenceAttire {
    garments: [u8; 9],
    regions: [u8; 4],
    molds: PalmMolds,
}

impl PresenceAttire {
    pub(super) const WIRE_SPAN: usize = 9 + 4 + 6;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(garments: [u8; 9], regions: [u8; 4], molds: PalmMolds) -> Self {
        Self { garments, regions, molds }
    }

    /// Secondary constructor initializing squire-code garments, neutral proportions, and cube palms.
    pub const fn squire() -> Self {
        Self::new([0; 9], [0; 4], PalmMolds::cubes())
    }

    /// Secondary constructor pairing garment and proportion bytes with unrotated cube palms.
    pub const fn with_garments(garments: [u8; 9], regions: [u8; 4]) -> Self {
        Self::new(garments, regions, PalmMolds::cubes())
    }

    /// Packs 4-bit crown style and two 6-bit (`0..=63`) hand item codes into `garments[4..=5]`.
    pub const fn pack_hands(crown: u8, right: u8, left: u8) -> (u8, u8) {
        let b4 = (crown & 0x0F)
            | (((right >> 4) & 1) << 4)
            | (((left >> 4) & 1) << 5)
            | (((right >> 5) & 1) << 6)
            | (((left >> 5) & 1) << 7);
        let b5 = (right & 0x0F) | ((left & 0x0F) << 4);
        (b4, b5)
    }

    /// Unpacks two 6-bit (`0..=63`) `(right, left)` hand item codes from `garments[4..=5]`.
    pub const fn unpack_hands(b4: u8, b5: u8) -> (u8, u8) {
        let right = (b5 & 0x0F) | (((b4 >> 4) & 1) << 4) | (((b4 >> 6) & 1) << 5);
        let left = (b5 >> 4) | (((b4 >> 5) & 1) << 4) | (((b4 >> 7) & 1) << 5);
        (right, left)
    }

    pub const fn held_codes(self) -> (u8, u8) {
        Self::unpack_hands(self.garments[4], self.garments[5])
    }

    pub fn transfer<T>(self, f: impl FnOnce([u8; 9], [u8; 4], PalmMolds) -> T) -> T {
        f(self.garments, self.regions, self.molds)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.garments);
        out.extend_from_slice(&self.regions);
        out.extend_from_slice(&self.molds.wire_bytes());
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < Self::WIRE_SPAN {
            return None;
        }
        let mut garments = [0u8; 9];
        garments.copy_from_slice(&bytes[0..9]);
        let mut regions = [0u8; 4];
        regions.copy_from_slice(&bytes[9..13]);
        let mut molds_raw = [0u8; 6];
        molds_raw.copy_from_slice(&bytes[13..19]);
        let molds = PalmMolds::from_wire_bytes(molds_raw);
        Some((Self::new(garments, regions, molds), Self::WIRE_SPAN))
    }
}
