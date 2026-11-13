use std::time::{SystemTime, UNIX_EPOCH};

/// Keyed 128-bit SipRound entropy source preventing GPU seed-cracking of subterranean mineral veins.
#[derive(Clone, Copy, Debug)]
pub struct EntropyWell {
    k0: u64,
    k1: u64,
    counter: u64,
}

impl EntropyWell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(k0: u64, k1: u64, counter: u64) -> Self {
        Self { k0, k1, counter }
    }

    /// Secondary constructor mixing hardware monotonic/epoch timing with a zone tag.
    pub fn ambient(zone_tag: u64) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x9e37_79b9_7f4a_7c15, |d| d.as_nanos() as u64);
        let stack_jitter = (&nanos as *const u64 as usize) as u64;
        let k0 = nanos.wrapping_mul(0xff51_afd7_ed55_8ccd) ^ zone_tag.rotate_left(17);
        let k1 = stack_jitter.wrapping_mul(0xc4ce_b9fe_1a85_ec53) ^ nanos.rotate_right(29);
        Self::new(k0 ^ 0x736f_6d65_7073_6575, k1 ^ 0x646f_7261_6e64_6f6d, 1)
    }

    /// Draws the next unpredictable 64-bit word mixed with spatial coordinate salt via SipRounds.
    pub fn next_word(&mut self, salt: u64) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let msg = salt ^ self.counter.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let mut v0 = self.k0 ^ 0x736f_6d65_7073_6575;
        let mut v1 = self.k1 ^ 0x646f_7261_6e64_6f6d;
        let mut v2 = self.k0 ^ 0x6c79_6765_6e65_7261;
        let mut v3 = self.k1 ^ 0x7465_6462_7974_6573 ^ msg;
        for _ in 0..2 {
            v0 = v0.wrapping_add(v1);
            v1 = v1.rotate_left(13) ^ v0;
            v0 = v0.rotate_left(32);
            v2 = v2.wrapping_add(v3);
            v3 = v3.rotate_left(16) ^ v2;
            v0 = v0.wrapping_add(v3);
            v3 = v3.rotate_left(21) ^ v0;
            v2 = v2.wrapping_add(v1);
            v1 = v1.rotate_left(17) ^ v2;
            v2 = v2.rotate_left(32);
        }
        v0 ^= msg;
        v2 ^= 0xff;
        for _ in 0..4 {
            v0 = v0.wrapping_add(v1);
            v1 = v1.rotate_left(13) ^ v0;
            v0 = v0.rotate_left(32);
            v2 = v2.wrapping_add(v3);
            v3 = v3.rotate_left(16) ^ v2;
            v0 = v0.wrapping_add(v3);
            v3 = v3.rotate_left(21) ^ v0;
            v2 = v2.wrapping_add(v1);
            v1 = v1.rotate_left(17) ^ v2;
            v2 = v2.rotate_left(32);
        }
        v0 ^ v1 ^ v2 ^ v3
    }
}
