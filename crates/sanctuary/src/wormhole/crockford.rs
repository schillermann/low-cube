const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const MASK_28: u32 = (1u32 << 28) - 1;

/// Domain-separated 12-character Crockford code (`XXXX-XXXX-XXXX`) with 56-bit Feistel scrambling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrockfordStamp {
    domain: u32,
}

impl CrockfordStamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(domain: u32) -> Self {
        Self { domain }
    }

    pub const fn host() -> Self {
        Self::new(0x484F_5354)
    }

    pub const fn punch() -> Self {
        Self::new(0x504E_4348)
    }

    fn round_mix(&self, half: u32, round: u32) -> u32 {
        let mut x = half
            .wrapping_add(self.domain)
            .wrapping_add(round.wrapping_mul(0x9E37_79B9));
        x ^= x >> 15;
        x = x.wrapping_mul(0x85EB_CA6B);
        x ^= x >> 13;
        x = x.wrapping_mul(0xC2B2_AE35);
        (x ^ (x >> 16)) & MASK_28
    }

    pub fn seal56(&self, plain56: u64) -> u64 {
        let mut left = ((plain56 >> 28) as u32) & MASK_28;
        let mut right = (plain56 as u32) & MASK_28;
        for round in 0..4u32 {
            let next_left = right;
            let next_right = (left ^ self.round_mix(right, round)) & MASK_28;
            left = next_left;
            right = next_right;
        }
        ((left as u64) << 28) | (right as u64)
    }

    pub fn open56(&self, cipher56: u64) -> u64 {
        let mut left = ((cipher56 >> 28) as u32) & MASK_28;
        let mut right = (cipher56 as u32) & MASK_28;
        for round in (0..4u32).rev() {
            let prev_right = left;
            let prev_left = (right ^ self.round_mix(prev_right, round)) & MASK_28;
            left = prev_left;
            right = prev_right;
        }
        ((left as u64) << 28) | (right as u64)
    }

    pub fn encode60(&self, plain56: u64, tag4: u8) -> String {
        let scrambled = self.seal56(plain56);
        let word60 = (scrambled << 4) | ((tag4 & 0x0F) as u64);
        let mut out = String::with_capacity(14);
        for idx in 0..12u32 {
            if idx == 4 || idx == 8 {
                out.push('-');
            }
            let shift = 55 - idx * 5;
            let val = ((word60 >> shift) & 0x1F) as usize;
            out.push(ALPHABET[val] as char);
        }
        out
    }

    fn decode_symbol(ch: char) -> Option<u8> {
        match ch.to_ascii_uppercase() {
            '0' | 'O' => Some(0),
            '1' | 'I' | 'L' => Some(1),
            '2'..='9' => Some((ch as u8) - b'0'),
            c @ 'A'..='H' => Some(10 + (c as u8) - b'A'),
            'J' => Some(18),
            'K' => Some(19),
            'M' => Some(20),
            'N' => Some(21),
            c @ 'P'..='T' => Some(22 + (c as u8) - b'P'),
            c @ 'V'..='Z' => Some(27 + (c as u8) - b'V'),
            _ => None,
        }
    }

    pub fn decode60(&self, input: &str) -> Option<(u64, u8)> {
        let mut word60 = 0u64;
        let mut count = 0usize;
        for ch in input.trim().chars() {
            if ch == '-' || ch == '_' || ch.is_ascii_whitespace() {
                continue;
            }
            let sym = Self::decode_symbol(ch)?;
            count += 1;
            if count > 12 {
                return None;
            }
            word60 = (word60 << 5) | (sym as u64);
        }
        if count != 12 {
            return None;
        }
        let tag4 = (word60 & 0x0F) as u8;
        let plain56 = self.open56(word60 >> 4);
        Some((plain56, tag4))
    }

    /// Detects 12-symbol alphanumeric code candidates so typos never trigger blocking DNS lookups.
    pub fn resembles_twelve_char_code(input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.contains('.') || trimmed.contains(':') || trimmed.eq_ignore_ascii_case("localhost") {
            return false;
        }
        let cleaned: usize = trimmed
            .chars()
            .filter(|ch| !matches!(*ch, '-' | '_') && !ch.is_ascii_whitespace())
            .count();
        cleaned == 12 && trimmed.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ' '))
    }
}
