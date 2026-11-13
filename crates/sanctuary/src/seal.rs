use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, Tag};
use x25519_dalek::{PublicKey, StaticSecret};

/// Ephemeral or persistent X25519 Diffie-Hellman keypair for authenticated session establishment.
pub struct HandshakeIdentity {
    secret: StaticSecret,
    public: PublicKey,
}

impl HandshakeIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(secret: StaticSecret, public: PublicKey) -> Self {
        Self { secret, public }
    }

    /// Secondary constructor drawing 32 fresh OS cryptographic entropy bytes.
    pub fn generate() -> Self {
        let mut raw = [0u8; 32];
        getrandom::fill(&mut raw).expect("OS CSPRNG entropy unavailable");
        let secret = StaticSecret::from(raw);
        let public = PublicKey::from(&secret);
        Self::new(secret, public)
    }

    pub fn public_bytes(&self) -> [u8; 32] {
        self.public.to_bytes()
    }

    pub fn portal_id(&self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out.copy_from_slice(&self.public.as_bytes()[0..8]);
        out
    }

    /// Derives a symmetric ChaCha20-Poly1305 `SessionSeal` bound to both parties' public keys.
    pub fn establish(&self, remote_pub: [u8; 32], initiator: bool) -> SessionSeal {
        let peer_key = PublicKey::from(remote_pub);
        let shared = self.secret.diffie_hellman(&peer_key);
        let (guest_pub, host_pub) = if initiator {
            (self.public_bytes(), remote_pub)
        } else {
            (remote_pub, self.public_bytes())
        };
        let mut derived = [0u8; 32];
        for i in 0..32 {
            let mix = guest_pub[i]
                .wrapping_mul(131)
                .wrapping_add(host_pub[31 - i])
                ^ (i as u8).wrapping_mul(17);
            derived[i] = shared.as_bytes()[i] ^ mix;
        }
        let aead = ChaCha20Poly1305::new(Key::from_slice(&derived));
        let start_seq = if initiator { 1 } else { (1u64 << 62) + 1 };
        SessionSeal::new(aead, start_seq, 0, 0)
    }
}

/// Authenticated ChaCha20-Poly1305 datagram seal with monotonic nonces and a 64-packet replay guard.
pub struct SessionSeal {
    aead: ChaCha20Poly1305,
    tx_seq: u64,
    rx_high: u64,
    rx_window: u64,
}

impl SessionSeal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(aead: ChaCha20Poly1305, tx_seq: u64, rx_high: u64, rx_window: u64) -> Self {
        Self { aead, tx_seq, rx_high, rx_window }
    }

    fn build_nonce(companion_id: u32, seq: u64) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        nonce[0..4].copy_from_slice(&companion_id.to_le_bytes());
        nonce[4..12].copy_from_slice(&seq.to_le_bytes());
        nonce
    }

    /// Seals `payload` into `[b'L', b'S', companion_id (4B), seq (8B), tag (16B), ciphertext (..)]`.
    pub fn seal(&mut self, companion_id: u32, payload: &[u8], out: &mut Vec<u8>) {
        let seq = self.tx_seq;
        self.tx_seq = self.tx_seq.wrapping_add(1);
        out.clear();
        out.reserve(30 + payload.len());
        out.extend_from_slice(b"LS");
        out.extend_from_slice(&companion_id.to_le_bytes());
        out.extend_from_slice(&seq.to_le_bytes());
        let aad = [
            out[0], out[1], out[2], out[3], out[4], out[5], out[6],
            out[7], out[8], out[9], out[10], out[11], out[12], out[13],
        ];
        let mut body = payload.to_vec();
        let nonce_bytes = Self::build_nonce(companion_id, seq);
        let tag = self
            .aead
            .encrypt_in_place_detached(Nonce::from_slice(&nonce_bytes), &aad, &mut body)
            .expect("AEAD seal failed");
        out.extend_from_slice(tag.as_slice());
        out.extend_from_slice(&body);
    }

    fn fresh_sequence(&self, seq: u64) -> bool {
        if seq == 0 {
            return false;
        }
        if seq > self.rx_high {
            true
        } else {
            let delta = self.rx_high - seq;
            delta < 64 && (self.rx_window & (1u64 << delta)) == 0
        }
    }

    fn commit_sequence(&mut self, seq: u64) {
        if seq > self.rx_high {
            let shift = seq - self.rx_high;
            self.rx_window = if shift >= 64 { 1 } else { (self.rx_window << shift) | 1 };
            self.rx_high = seq;
        } else {
            let delta = self.rx_high - seq;
            self.rx_window |= 1u64 << delta;
        }
    }

    /// Authenticates, anti-replay checks, and decrypts a sealed frame, returning `(companion_id, plaintext)`.
    pub fn open_frame(&mut self, frame: &[u8]) -> Option<(u32, Vec<u8>)> {
        if frame.len() < 30 || &frame[0..2] != b"LS" {
            return None;
        }
        let companion_id = u32::from_le_bytes(frame[2..6].try_into().ok()?);
        let seq = u64::from_le_bytes(frame[6..14].try_into().ok()?);
        if !self.fresh_sequence(seq) {
            return None;
        }
        let aad = &frame[0..14];
        let tag = Tag::from_slice(&frame[14..30]);
        let mut body = frame[30..].to_vec();
        let nonce_bytes = Self::build_nonce(companion_id, seq);
        self.aead
            .decrypt_in_place_detached(Nonce::from_slice(&nonce_bytes), aad, &mut body, tag)
            .ok()?;
        self.commit_sequence(seq);
        Some((companion_id, body))
    }
}
