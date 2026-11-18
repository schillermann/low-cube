use glam::Vec3;

/// Encapsulated 12-byte little-endian wire inscription of a finite 3D vector.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WireVector {
    vector: Vec3,
}

impl WireVector {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(crate) const fn new(vector: Vec3) -> Self {
        Self { vector }
    }

    pub(crate) fn inscribe(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.vector.x.to_le_bytes());
        out.extend_from_slice(&self.vector.y.to_le_bytes());
        out.extend_from_slice(&self.vector.z.to_le_bytes());
    }
}

/// Encapsulated byte-slice reader decoding a finite 12-byte little-endian `Vec3`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WireSlice<'a> {
    bytes: &'a [u8],
}

impl<'a> WireSlice<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub(crate) fn vec3(self) -> Option<Vec3> {
        if self.bytes.len() < 12 {
            return None;
        }
        let v = Vec3::new(
            f32::from_le_bytes(self.bytes[0..4].try_into().ok()?),
            f32::from_le_bytes(self.bytes[4..8].try_into().ok()?),
            f32::from_le_bytes(self.bytes[8..12].try_into().ok()?),
        );
        v.is_finite().then_some(v)
    }
}
