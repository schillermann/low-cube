use glam::Vec3;
use crate::CosmicFrame;
use super::SatchelManifest;

/// Persistent survivor checkpoint uniting Hill-bubble `CosmicFrame`, local `Vec3`, and `SatchelManifest` (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurvivorRecord {
    frame: CosmicFrame,
    position: Vec3,
    manifest: SatchelManifest,
}

impl SurvivorRecord {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(frame: CosmicFrame, position: Vec3, manifest: SatchelManifest) -> Self {
        Self { frame, position, manifest }
    }

    pub const fn frame(&self) -> CosmicFrame {
        self.frame
    }

    pub const fn position(&self) -> Vec3 {
        self.position
    }

    pub const fn manifest(&self) -> SatchelManifest {
        self.manifest
    }

    pub const fn with_locus(self, frame: CosmicFrame, position: Vec3) -> Self {
        Self::new(frame, position, self.manifest)
    }

    pub const fn with_manifest(self, manifest: SatchelManifest) -> Self {
        Self::new(self.frame, self.position, manifest)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        self.frame.inscribe(out);
        for c in [self.position.x, self.position.y, self.position.z] {
            out.extend_from_slice(&c.to_le_bytes());
        }
        self.manifest.inscribe_sparse(out);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let (frame, f_used) = CosmicFrame::decode(bytes)?;
        if bytes.len() < f_used + 12 {
            return None;
        }
        let px = f32::from_le_bytes(bytes[f_used..f_used + 4].try_into().ok()?);
        let py = f32::from_le_bytes(bytes[f_used + 4..f_used + 8].try_into().ok()?);
        let pz = f32::from_le_bytes(bytes[f_used + 8..f_used + 12].try_into().ok()?);
        let position = Vec3::new(px, py, pz);
        if !position.is_finite() {
            return None;
        }
        let (manifest, m_used) = SatchelManifest::decode_sparse(&bytes[f_used + 12..])?;
        Some((Self::new(frame, position, manifest), f_used + 12 + m_used))
    }
}

/// Polymorphic server welcome assignment distinguishing fresh starter kits from restored survivor checkpoints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurvivorWelcome {
    FreshKit(SatchelManifest),
    Restored(SurvivorRecord),
}

impl SurvivorWelcome {
    pub const fn manifest(&self) -> SatchelManifest {
        match *self {
            Self::FreshKit(m) => m,
            Self::Restored(r) => r.manifest(),
        }
    }

    pub const fn restored_record(&self) -> Option<SurvivorRecord> {
        match *self {
            Self::FreshKit(_) => None,
            Self::Restored(r) => Some(r),
        }
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        match *self {
            Self::FreshKit(m) => {
                out.push(1);
                m.inscribe_sparse(out);
            }
            Self::Restored(r) => {
                out.push(2);
                r.inscribe(out);
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let tag = *bytes.first()?;
        match tag {
            1 => {
                let (m, used) = SatchelManifest::decode_sparse(&bytes[1..])?;
                Some((Self::FreshKit(m), 1 + used))
            }
            2 => {
                let (r, used) = SurvivorRecord::decode(&bytes[1..])?;
                Some((Self::Restored(r), 1 + used))
            }
            _ => None,
        }
    }
}
