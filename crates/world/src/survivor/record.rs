use glam::Vec3;
use crate::{Callsign, CosmicFrame};
use super::SatchelManifest;

/// Persistent survivor checkpoint uniting Hill-bubble `CosmicFrame`, local `Vec3`, `SatchelManifest`, and `Callsign` (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurvivorRecord {
    frame: CosmicFrame,
    position: Vec3,
    manifest: SatchelManifest,
    callsign: Callsign,
}

impl SurvivorRecord {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_callsign(
        frame: CosmicFrame,
        position: Vec3,
        manifest: SatchelManifest,
        callsign: Callsign,
    ) -> Self {
        Self { frame, position, manifest, callsign }
    }

    /// Secondary constructor initializing an unnamed checkpoint.
    pub const fn new(frame: CosmicFrame, position: Vec3, manifest: SatchelManifest) -> Self {
        Self::with_callsign(frame, position, manifest, Callsign::unnamed())
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

    pub const fn callsign(&self) -> Callsign {
        self.callsign
    }

    pub const fn with_locus(self, frame: CosmicFrame, position: Vec3) -> Self {
        Self::with_callsign(frame, position, self.manifest, self.callsign)
    }

    pub const fn with_manifest(self, manifest: SatchelManifest) -> Self {
        Self::with_callsign(self.frame, self.position, manifest, self.callsign)
    }

    pub const fn renamed(self, callsign: Callsign) -> Self {
        Self::with_callsign(self.frame, self.position, self.manifest, callsign)
    }

    pub fn inscribe_roster_entry(&self, short_tag: &str, pubkey_hex: &str, out: &mut String) {
        let name = if self.callsign.present() {
            self.callsign.as_str()
        } else {
            "unnamed"
        };
        let locus = match self.frame {
            CosmicFrame::Body(id) => format!("body_{}", id.raw_code()),
            CosmicFrame::BeltAsteroid { id, .. } => format!("belt_rock_{}", id.raw_code()),
            CosmicFrame::DeepSpace(sec) => format!("deep_space_{}_{}_{}", sec[0], sec[1], sec[2]),
        };
        out.push_str(&format!(
            "\n[survivor.{short_tag}]\nname = \"{name}\"\nidentity_pub = \"{pubkey_hex}\"\nframe = \"{locus}\"\nposition = [{:.2}, {:.2}, {:.2}]\nitems = \"{}\"\n",
            self.position.x,
            self.position.y,
            self.position.z,
            self.manifest.inventory_summary(),
        ));
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        self.frame.inscribe(out);
        for c in [self.position.x, self.position.y, self.position.z] {
            out.extend_from_slice(&c.to_le_bytes());
        }
        self.manifest.inscribe_sparse(out);
        self.callsign.inscribe(out);
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
        let manifest_end = f_used + 12 + m_used;
        let (callsign, c_used) = Callsign::decode(&bytes[manifest_end..])?;
        Some((
            Self::with_callsign(frame, position, manifest, callsign),
            manifest_end + c_used,
        ))
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
