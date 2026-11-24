use std::collections::HashMap;
use std::net::SocketAddr;
use glam::{Quat, Vec3};
use world::{
    Callsign, CompanionPresence, CosmicFrame, PresenceAttire, PresenceKinematics, SurvivorRecord,
    SurvivorWelcome,
};
use crate::aura::{HostAura, HostIdentity};
use crate::glide::CompanionGlide;
use crate::host_wire::STRUCTURED_WELCOME_TAG;
use crate::link_state::WelcomeSpawn;
#[cfg(not(test))]
use crate::passport::HexKey;
use crate::passport::{KnockFrame, KnockIntent};
use crate::seal::HandshakeIdentity;
use crate::spatial_chronicle::HorizonStream;
use crate::survivor_ledger::SurvivorStore;
use crate::ward::{GuestWard, WardVitals};
use super::GuestCohort;

/// Polymorphic server identity response either confirming an existing passport or issuing a fresh secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityGrant {
    Confirmed(Callsign),
    Issued { secret: [u8; 32], callsign: Callsign },
}

impl IdentityGrant {
    pub const fn callsign(&self) -> Callsign {
        match *self {
            Self::Confirmed(c) => c,
            Self::Issued { callsign, .. } => callsign,
        }
    }

    /// Appends the resolved callsign and (when newly issued) AEAD-sealed 32-byte secret to `LWEL`, returning flag bits.
    pub fn inscribe_welcome_tail(
        &self,
        cid: u32,
        host_keys: &HandshakeIdentity,
        ephem_pub: [u8; 32],
        out: &mut Vec<u8>,
    ) -> u8 {
        self.callsign().inscribe(out);
        match *self {
            Self::Confirmed(_) => 0x04,
            Self::Issued { ref secret, .. } => {
                let mut ephem_seal = host_keys.establish(ephem_pub, false);
                let mut sealed_grant = Vec::with_capacity(62);
                ephem_seal.seal(cid, secret, &mut sealed_grant);
                out.extend_from_slice(&sealed_grant);
                0x04 | 0x08
            }
        }
    }
}

/// Complete admission outcome produced by `GuestCohort` without nullable `Option` fields (`<= 4` fields).
pub struct CohortAdmission {
    slot: u32,
    spawn: WelcomeSpawn,
    welcome: SurvivorWelcome,
    grant: IdentityGrant,
}

impl CohortAdmission {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        slot: u32,
        spawn: WelcomeSpawn,
        welcome: SurvivorWelcome,
        grant: IdentityGrant,
    ) -> Self {
        Self { slot, spawn, welcome, grant }
    }

    pub fn into_legacy_tuple(self) -> (u32, Option<Vec3>, SurvivorWelcome) {
        (self.slot, self.spawn.assigned_point(), self.welcome)
    }

    /// Inscribes a complete structured `LWEL` datagram without exposing internal admission fields.
    pub fn inscribe_welcome(
        &self,
        identity: &HostIdentity,
        epoch_secs: f64,
        guest_pub: [u8; 32],
        out: &mut Vec<u8>,
    ) {
        out.clear();
        out.reserve(240);
        out.extend_from_slice(b"LWEL");
        out.extend_from_slice(&identity.keys().public_bytes());
        out.extend_from_slice(&self.slot.to_le_bytes());
        out.extend_from_slice(&identity.blocks_per_edge().to_le_bytes());
        identity.proportions().inscribe_tail(out);
        let beacon_pt = self.spawn.assigned_point();
        let core_flags = u8::from(beacon_pt.is_some()) | 0x02;
        out.push(STRUCTURED_WELCOME_TAG);
        let flags_idx = out.len();
        out.push(core_flags);
        out.extend_from_slice(&epoch_secs.to_le_bytes());
        if let Some(pos) = beacon_pt {
            for c in [pos.x, pos.y, pos.z] {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        self.welcome.inscribe(out);
        let grant_flags = self.grant.inscribe_welcome_tail(
            self.slot,
            identity.keys(),
            guest_pub,
            out,
        );
        out[flags_idx] = core_flags | grant_flags;
    }
}

/// Slot ledger and idempotent handshake grant cache preventing duplicate identity minting across UDP retries (`<= 4` fields).
pub struct SlotLedger {
    slots: HashMap<[u8; 32], u32>,
    minted_knocks: Vec<([u8; 32], [u8; 32])>,
    next_slot: u32,
}

impl SlotLedger {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        slots: HashMap<[u8; 32], u32>,
        minted_knocks: Vec<([u8; 32], [u8; 32])>,
        next_slot: u32,
    ) -> Self {
        Self { slots, minted_knocks, next_slot }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new(), Vec::new(), 2)
    }

    pub fn from_index(slots: HashMap<[u8; 32], u32>, next_slot: u32) -> Self {
        Self::new(slots, Vec::new(), next_slot.max(2))
    }

    pub fn slot_for(&mut self, survivor_pub: [u8; 32]) -> u32 {
        if let Some(&existing) = self.slots.get(&survivor_pub) {
            existing
        } else {
            let id = self.next_slot;
            self.next_slot = self.next_slot.wrapping_add(1).max(2);
            self.slots.insert(survivor_pub, id);
            id
        }
    }

    /// Resolves `(slot, authoritative_pubkey, optional_minted_secret, retry_of_mint)`.
    pub fn resolve_knock(
        &mut self,
        knock: KnockFrame,
        store: &dyn SurvivorStore,
    ) -> (u32, [u8; 32], Option<[u8; 32]>, bool) {
        let should_mint = knock.needs_minted_identity(store);
        let (ephem_pub, _) = knock.transfer(|pk, cs| (pk, cs));
        if !should_mint {
            let slot = self.slot_for(ephem_pub);
            return (slot, ephem_pub, None, false);
        }
        if let Some(&(_, cached_secret)) = self
            .minted_knocks
            .iter()
            .find(|&&(ephem, _)| ephem == ephem_pub)
        {
            let minted_pub = HandshakeIdentity::from_secret_bytes(cached_secret).public_bytes();
            let slot = self.slot_for(minted_pub);
            return (slot, minted_pub, Some(cached_secret), true);
        }
        let minted = HandshakeIdentity::generate();
        let secret = minted.secret_bytes();
        let minted_pub = minted.public_bytes();
        if self.minted_knocks.len() >= 64 {
            self.minted_knocks.remove(0);
        }
        self.minted_knocks.push((ephem_pub, secret));
        let slot = self.slot_for(minted_pub);
        (slot, minted_pub, Some(secret), false)
    }
}

impl GuestCohort {
    pub fn admit_guest(
        &mut self,
        identity: &HostIdentity,
        aura: &mut HostAura,
        peer: SocketAddr,
        guest_pub: [u8; 32],
    ) -> (u32, Option<Vec3>, SurvivorWelcome) {
        let knock = KnockFrame::new(guest_pub, KnockIntent::PinnedKey, Callsign::unnamed());
        self.admit_knock(identity, aura, peer, knock).into_legacy_tuple()
    }

    pub fn admit_knock(
        &mut self,
        identity: &HostIdentity,
        aura: &mut HostAura,
        peer: SocketAddr,
        knock: KnockFrame,
    ) -> CohortAdmission {
        let (_, desired_callsign) = knock.transfer(|pk, cs| (pk, cs));
        let (cid, survivor_pub, minted_secret, knock_retry) =
            self.identities.resolve_knock(knock, self.survivors.as_ref());
        if self.wards.get(&cid).is_some_and(|(w, _)| w.established_traffic()) {
            self.commit_survivor(cid, aura, true);
            let _ = self.wards.remove(&cid);
        }
        let active: Vec<([u8; 32], Callsign)> = self
            .wards
            .values()
            .map(|(w, _)| (w.guest_pubkey(), w.callsign()))
            .collect();
        let welcome = match self.wards.get(&cid) {
            Some((w, _)) if knock_retry && !w.established_traffic() => SurvivorWelcome::FreshKit(w.manifest()),
            _ => self.survivors.admit_or_restore(survivor_pub),
        };
        let callsign = self.survivors.resolve_callsign(survivor_pub, desired_callsign, &active);
        let half = (identity.blocks_per_edge() as f32) * 0.5;
        let (kin, beacon_stand) = match welcome.restored_record() {
            Some(rec) => (PresenceKinematics::new(rec.position(), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)), None),
            None => aura.beacons_mut().allocate_kinematics(half),
        };
        if minted_secret.is_some() && !knock_retry {
            let spawn_pos = kin.transfer(|p, _, _, _, _| p);
            let initial = SurvivorRecord::with_callsign(CosmicFrame::earth(), spawn_pos, welcome.manifest(), callsign);
            self.survivors.commit(survivor_pub, initial, true);
            #[cfg(not(test))]
            println!("[SANCTUARY] Issued identity #{} to '{}' (slot P{cid})", HexKey::new(survivor_pub).short_tag(), callsign.as_str());
        } else if let Some(prev) = self.survivors.lookup(survivor_pub) {
            if prev.callsign() != callsign {
                self.survivors.commit(survivor_pub, prev.renamed(callsign), true);
            }
            #[cfg(not(test))]
            println!("[SANCTUARY] Restored survivor '{}' (#{}, slot P{cid})", callsign.as_str(), HexKey::new(survivor_pub).short_tag());
        }
        if !self.wards.contains_key(&cid) {
            let seal = identity.keys().establish(survivor_pub, false);
            let pres = CompanionPresence::new(cid, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire());
            let frame = welcome.restored_record().map_or(CosmicFrame::earth(), |r| r.frame());
            let vitals = WardVitals::named_survivor(survivor_pub, callsign, welcome.manifest());
            let ward = GuestWard::with_locus(seal, (peer, frame), CompanionGlide::awaiting_first(pres), vitals);
            self.wards.insert(cid, (ward, HorizonStream::empty()));
        }
        let grant = match minted_secret {
            Some(secret) => IdentityGrant::Issued { secret, callsign },
            None => IdentityGrant::Confirmed(callsign),
        };
        CohortAdmission::new(cid, WelcomeSpawn::from_optional(beacon_stand), welcome, grant)
    }
}
