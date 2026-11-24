mod disk;
mod hex_key;
mod knock;

use std::net::SocketAddr;
use world::Callsign;
use crate::seal::HandshakeIdentity;
use crate::ticket::JoinTarget;

pub use self::disk::{DiskPassports, PassportDocument};
pub use self::hex_key::HexKey;
pub use self::knock::{KnockFrame, KnockIntent, KnockPassport, KNOCK_PASSPORT_TAG};

/// Polymorphic client credential status for a target realm avoiding nullable `Option<[u8; 32]>` fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassportClaim {
    Unassigned,
    Issued([u8; 32]),
    Pinned([u8; 32]),
}

impl PassportClaim {
    pub fn into_credentials(self) -> (HandshakeIdentity, KnockIntent) {
        match self {
            Self::Unassigned => (HandshakeIdentity::generate(), KnockIntent::RequestIdentity),
            Self::Issued(sec) => (HandshakeIdentity::from_secret_bytes(sec), KnockIntent::PresentIssued),
            Self::Pinned(sec) => (HandshakeIdentity::from_secret_bytes(sec), KnockIntent::PinnedKey),
        }
    }
}

/// Server-issued identity credential binding a realm's X25519 host key and endpoint to an assigned secret (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassportStamp {
    host_key: [u8; 32],
    endpoint: SocketAddr,
    secret: [u8; 32],
    callsign: Callsign,
}

impl PassportStamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        host_key: [u8; 32],
        endpoint: SocketAddr,
        secret: [u8; 32],
        callsign: Callsign,
    ) -> Self {
        Self { host_key, endpoint, secret, callsign }
    }

    pub fn same_host_as(&self, other: &Self) -> bool {
        self.host_key == other.host_key
    }

    pub fn matches_target(&self, target: JoinTarget) -> bool {
        match target {
            JoinTarget::Ticket(ticket) => self.host_key == ticket.host_key(),
            JoinTarget::Direct(addr) => self.endpoint == addr,
            JoinTarget::Wormhole(_) => false,
        }
    }

    pub const fn issued_claim(&self) -> PassportClaim {
        PassportClaim::Issued(self.secret)
    }

    pub fn record_into(&self, claim: &mut PassportClaim, callsign: &mut Callsign) {
        *claim = PassportClaim::Issued(self.secret);
        if self.callsign.present() {
            *callsign = self.callsign;
        }
    }

    pub fn inscribe_toml_line(&self, out: &mut String) {
        out.push_str(&format!(
            "{} = \"{}|{}|{}\"\n",
            HexKey::new(self.host_key).encoded(),
            HexKey::new(self.secret).encoded(),
            self.endpoint,
            self.callsign.as_str(),
        ));
    }
}

/// Focused client keyring contract storing per-server issued identities and preferred player callsign.
pub trait PassportVault: Send {
    fn preferred_callsign(&self) -> Callsign;
    fn claim_for(&self, target: JoinTarget) -> PassportClaim;
    fn store_issued(&mut self, stamp: PassportStamp);
}

/// Stateless single-session `PassportVault` for programmatic links and isolated tests (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct FixedPassport {
    claim: PassportClaim,
    callsign: Callsign,
}

impl FixedPassport {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(claim: PassportClaim, callsign: Callsign) -> Self {
        Self { claim, callsign }
    }

    pub const fn unassigned() -> Self {
        Self::new(PassportClaim::Unassigned, Callsign::unnamed())
    }

    pub const fn unassigned_named(callsign: Callsign) -> Self {
        Self::new(PassportClaim::Unassigned, callsign)
    }

    pub fn pinned(keys: &HandshakeIdentity) -> Self {
        Self::new(PassportClaim::Pinned(keys.secret_bytes()), Callsign::unnamed())
    }
}

impl PassportVault for FixedPassport {
    fn preferred_callsign(&self) -> Callsign {
        self.callsign
    }

    fn claim_for(&self, _target: JoinTarget) -> PassportClaim {
        self.claim
    }

    fn store_issued(&mut self, stamp: PassportStamp) {
        stamp.record_into(&mut self.claim, &mut self.callsign);
    }
}

/// In-memory `PassportVault` indexing server-issued identities by `host_key` and direct `SocketAddr` (`<= 4` fields).
#[derive(Clone, Debug)]
pub struct MemoryPassports {
    stamps: Vec<PassportStamp>,
    callsign: Callsign,
}

impl MemoryPassports {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(stamps: Vec<PassportStamp>, callsign: Callsign) -> Self {
        Self { stamps, callsign }
    }

    pub const fn with_callsign(callsign: Callsign) -> Self {
        Self::new(Vec::new(), callsign)
    }

    pub fn stamp_count(&self) -> usize {
        self.stamps.len()
    }

    pub fn inscribe_toml(&self, out: &mut String) {
        out.push_str("# Low-Cube Server-Issued Passports\n[profile]\n");
        out.push_str(&format!("name = \"{}\"\n\n[passports]\n", self.callsign.as_str()));
        for stamp in &self.stamps {
            stamp.inscribe_toml_line(out);
        }
    }
}

impl PassportVault for MemoryPassports {
    fn preferred_callsign(&self) -> Callsign {
        self.callsign
    }

    fn claim_for(&self, target: JoinTarget) -> PassportClaim {
        self.stamps
            .iter()
            .rev()
            .find(|s| s.matches_target(target))
            .map_or(PassportClaim::Unassigned, PassportStamp::issued_claim)
    }

    fn store_issued(&mut self, stamp: PassportStamp) {
        self.stamps.retain(|existing| !existing.same_host_as(&stamp));
        self.stamps.push(stamp);
    }
}
