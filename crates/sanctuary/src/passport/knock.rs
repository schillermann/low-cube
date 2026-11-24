use std::net::{SocketAddr, UdpSocket};
use world::Callsign;
use crate::seal::{HandshakeIdentity, SessionSeal};
use crate::survivor_ledger::SurvivorStore;
use crate::ticket::JoinTarget;
use super::{PassportStamp, PassportVault};

pub const KNOCK_PASSPORT_TAG: u8 = 0x50;
pub const SEALED_GRANT_SPAN: usize = 62;

/// Polymorphic client knock intent distinguishing first-time identity requests from returning passports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnockIntent {
    RequestIdentity,
    PresentIssued,
    PinnedKey,
}

impl KnockIntent {
    const fn wire_code(self) -> u8 {
        match self {
            Self::PinnedKey => 0,
            Self::RequestIdentity => 1,
            Self::PresentIssued => 2,
        }
    }

    const fn from_wire(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::PinnedKey),
            1 => Some(Self::RequestIdentity),
            2 => Some(Self::PresentIssued),
            _ => None,
        }
    }
}

/// Parsed `LKNK` datagram uniting guest X25519 public key, `KnockIntent`, and preferred `Callsign` (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnockFrame {
    guest_pub: [u8; 32],
    intent: KnockIntent,
    callsign: Callsign,
}

impl KnockFrame {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(guest_pub: [u8; 32], intent: KnockIntent, callsign: Callsign) -> Self {
        Self { guest_pub, intent, callsign }
    }

    pub fn needs_minted_identity(&self, store: &dyn SurvivorStore) -> bool {
        match self.intent {
            KnockIntent::RequestIdentity => true,
            KnockIntent::PresentIssued => store.lookup(self.guest_pub).is_none(),
            KnockIntent::PinnedKey => false,
        }
    }

    pub fn transfer<R>(self, f: impl FnOnce([u8; 32], Callsign) -> R) -> R {
        f(self.guest_pub, self.callsign)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(64);
        out.extend_from_slice(b"LKNK");
        out.extend_from_slice(&self.guest_pub);
        out.push(KNOCK_PASSPORT_TAG);
        out.push(self.intent.wire_code());
        self.callsign.inscribe(out);
    }

    pub fn decode(pkt: &[u8]) -> Option<Self> {
        if pkt.len() < 36 || &pkt[0..4] != b"LKNK" {
            return None;
        }
        let mut guest_pub = [0u8; 32];
        guest_pub.copy_from_slice(&pkt[4..36]);
        if pkt.len() == 36 || pkt[36] != KNOCK_PASSPORT_TAG {
            return Some(Self::new(guest_pub, KnockIntent::PinnedKey, Callsign::unnamed()));
        }
        if pkt.len() < 39 {
            return None;
        }
        let intent = KnockIntent::from_wire(pkt[37])?;
        let (raw_callsign, _) = Callsign::decode(&pkt[38..])?;
        Some(Self::new(guest_pub, intent, Callsign::sanitized(raw_callsign.as_str())))
    }
}

/// Client handshake credential session pairing X25519 keys with a `PassportVault` (`<= 4` fields).
pub struct KnockPassport {
    keys: HandshakeIdentity,
    intent: KnockIntent,
    vault: Box<dyn PassportVault>,
}

impl KnockPassport {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(keys: HandshakeIdentity, intent: KnockIntent, vault: Box<dyn PassportVault>) -> Self {
        Self { keys, intent, vault }
    }

    pub fn from_vault(target: JoinTarget, vault: Box<dyn PassportVault>) -> Self {
        let (keys, intent) = vault.claim_for(target).into_credentials();
        Self::new(keys, intent, vault)
    }

    pub fn retarget(&mut self, target: JoinTarget) {
        let (keys, intent) = self.vault.claim_for(target).into_credentials();
        self.keys = keys;
        self.intent = intent;
    }

    pub fn emit_knock(&self, target: JoinTarget, socket: &UdpSocket) {
        let mut pkt = Vec::with_capacity(64);
        KnockFrame::new(
            self.keys.public_bytes(),
            self.intent,
            self.vault.preferred_callsign(),
        )
        .inscribe(&mut pkt);
        match target {
            JoinTarget::Direct(addr) => {
                let _ = socket.send_to(&pkt, addr);
            }
            JoinTarget::Ticket(ticket) => {
                let _ = socket.send_to(&pkt, ticket.direct());
            }
            JoinTarget::Wormhole(_) => {}
        }
    }

    /// Verifies an `LWEL` welcome, decrypting and persisting any server-issued identity before sealing the session.
    pub fn accept_welcome(
        &mut self,
        cid: u32,
        host_pub: [u8; 32],
        peer: SocketAddr,
        assigned_callsign: Callsign,
        sealed_grant: Option<&[u8]>,
    ) -> Option<(SessionSeal, Callsign)> {
        if let Some(grant_frame) = sealed_grant {
            if grant_frame.len() != SEALED_GRANT_SPAN {
                return None;
            }
            let mut ephem_seal = self.keys.establish(host_pub, true);
            let (grant_cid, plain) = ephem_seal.open_frame(grant_frame)?;
            if grant_cid != cid || plain.len() != 32 {
                return None;
            }
            let mut issued_secret = [0u8; 32];
            issued_secret.copy_from_slice(&plain);
            self.vault
                .store_issued(PassportStamp::new(host_pub, peer, issued_secret, assigned_callsign));
            let issued_keys = HandshakeIdentity::from_secret_bytes(issued_secret);
            return Some((issued_keys.establish(host_pub, true), assigned_callsign));
        }
        if self.intent == KnockIntent::PresentIssued {
            self.vault.store_issued(PassportStamp::new(
                host_pub,
                peer,
                self.keys.secret_bytes(),
                assigned_callsign,
            ));
        }
        Some((self.keys.establish(host_pub, true), assigned_callsign))
    }
}
