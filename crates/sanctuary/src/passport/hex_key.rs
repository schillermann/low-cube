/// Stack-allocated 32-byte key hex codec providing instance formatting without static utility functions (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HexKey {
    bytes: [u8; 32],
}

impl HexKey {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    pub fn parsed(hex: &str) -> Option<Self> {
        let trimmed = hex.trim();
        if trimmed.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (idx, slot) in out.iter_mut().enumerate() {
            *slot = u8::from_str_radix(&trimmed[idx * 2..idx * 2 + 2], 16).ok()?;
        }
        Some(Self::new(out))
    }

    pub const fn raw(self) -> [u8; 32] {
        self.bytes
    }

    pub fn short_tag(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}",
            self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]
        )
    }

    pub fn encoded(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::with_capacity(64);
        for b in &self.bytes {
            let _ = write!(&mut out, "{b:02x}");
        }
        out
    }
}
