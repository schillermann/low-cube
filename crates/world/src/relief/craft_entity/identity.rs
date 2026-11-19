use glam::Vec3;
use super::super::vehicle_assembly::VehiclePart;

/// Immutable 64-bit entity identity of a commissioned multi-block vehicle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CraftIdentity {
    code: u64,
}

impl CraftIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: u64) -> Self {
        Self { code }
    }

    pub const fn unassigned() -> Self {
        Self::new(0)
    }

    pub const fn assigned(self) -> bool {
        self.code != 0
    }

    pub fn from_parts(origin: Vec3, parts: &[VehiclePart]) -> Self {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for q in [
            (origin.x * 4.0).round() as i32,
            (origin.y * 4.0).round() as i32,
            (origin.z * 4.0).round() as i32,
        ] {
            for b in q.to_le_bytes() {
                h = (h ^ (b as u64)).wrapping_mul(0x1000_0000_01b3);
            }
        }
        for part in parts {
            h = part.digest_mix(h);
        }
        Self::new(if h == 0 { 1 } else { h })
    }

    pub fn inscribe(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.code.to_le_bytes());
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 8 {
            return None;
        }
        let code = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        Some((Self::new(code), 8))
    }
}
