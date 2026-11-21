use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::{Quat, Vec3};
use world::{
    CompanionPresence, DrivenCraft, PilotCraft, PlanetaryWorld, PresenceAttire, PresenceKinematics,
    WildlifePulse,
};
use crate::aura::{HostAura, HostIdentity, HostPresence, PeerCrack};
use crate::chronicle_log::ChronicleLog;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::ward::{GuestWard, WardVitals};

/// Connected companion wards indexed by `companion_id` and X25519 public key.
pub struct GuestCohort {
    wards: HashMap<u32, GuestWard>,
    key_index: HashMap<[u8; 32], u32>,
    next_id: u32,
    pulse_clock: f32,
}

impl GuestCohort {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(wards: HashMap<u32, GuestWard>, key_index: HashMap<[u8; 32], u32>, next_id: u32, pulse_clock: f32) -> Self {
        Self { wards, key_index, next_id, pulse_clock }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new(), HashMap::new(), 2, 0.0)
    }

    pub fn ward_mut(&mut self, cid: u32) -> Option<&mut GuestWard> {
        self.wards.get_mut(&cid)
    }

    pub fn record_guest_gore(&mut self, cid: u32, impulse: Vec3) {
        if let Some(ward) = self.wards.get_mut(&cid) {
            ward.record_gore(impulse);
        }
    }

    pub fn push_pedestrian_observers(&self, world: &dyn PlanetaryWorld, out: &mut Vec<(u32, Vec3, bool)>) {
        for (&cid, ward) in &self.wards {
            if let Some((pos, crouching)) = ward.pedestrian_observer(world) {
                out.push((cid, pos, crouching));
            }
        }
    }

    pub fn guest_companions(&self, out: &mut Vec<CompanionPresence>) {
        out.clear();
        for ward in self.wards.values() {
            out.push(ward.smooth_presence());
        }
    }

    pub fn guest_crafts(&self, out: &mut Vec<DrivenCraft>) {
        out.clear();
        for ward in self.wards.values() {
            ward.push_smooth_craft(out);
        }
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        for ward in self.wards.values_mut() {
            ward.blend_subtick(subtick_secs);
        }
    }

    pub fn admit_guest(&mut self, identity: &HostIdentity, world: &dyn PlanetaryWorld, peer: SocketAddr, guest_pub: [u8; 32]) -> u32 {
        let cid = if let Some(&existing) = self.key_index.get(&guest_pub) {
            existing
        } else {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(2);
            self.key_index.insert(guest_pub, id);
            id
        };
        let seal = identity.keys().establish(guest_pub, false);
        let spawn = world.constrain_to_surface(Vec3::new(0.0, 64.0, 0.0), 0.0);
        let kin = PresenceKinematics::new(spawn, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        let pres = CompanionPresence::new(cid, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire());
        self.wards.entry(cid).or_insert_with(|| GuestWard::new(seal, peer, pres, WardVitals::fresh()));
        cid
    }

    pub fn advance_wards(&mut self, dt: f32, abandoned_out: &mut Vec<(u32, PilotCraft)>) -> bool {
        for ward in self.wards.values_mut() {
            ward.step(dt);
        }
        let expired: Vec<u32> = self
            .wards
            .iter()
            .filter_map(|(&cid, w)| (!w.alive()).then_some(cid))
            .collect();
        for cid in expired {
            if let Some(dead) = self.wards.remove(&cid) {
                abandoned_out.push((cid, dead.into_pilot_craft()));
            }
        }
        self.pulse_clock += dt;
        if self.pulse_clock >= 0.025 {
            self.pulse_clock = 0.0;
            true
        } else {
            false
        }
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, chronicle: &ChronicleLog, aura: &mut HostAura) {
        let (mut payload, mut scratch, mut roster, mut crafts) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        if let HostPresence::Occupied(p) = aura.presence() {
            roster.push(p);
            aura.push_host_craft(&mut crafts);
        }
        for ward in self.wards.values() {
            roster.push(ward.presence());
            ward.push_authoritative_craft(&mut crafts);
        }
        let sounder = aura.wildlife().presence();
        let peer_cracks = aura.drain_cracks();
        let total_facts = chronicle.total();
        for (&cid, ward) in &mut self.wards {
            if let Some((base_seq, facts)) = chronicle.batch_from(ward.ack_seq()) {
                SanctuaryEnvelope::HostChronicle { base_seq, facts }.inscribe(&mut payload);
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
            let others: Vec<CompanionPresence> = roster.iter().copied().filter(|p| !p.same_slot(cid)).collect();
            let other_crafts: Vec<DrivenCraft> = crafts.iter().copied().filter(|c| !c.same_slot(cid)).collect();
            let cracks: Vec<CrackNotice> = peer_cracks
                .iter()
                .copied()
                .filter(|c| !c.authored_by(cid))
                .map(PeerCrack::notice)
                .collect();
            let wildlife = WildlifePulse::new(sounder, ward.gore_stamp());
            SanctuaryEnvelope::HostPulse {
                total_facts,
                cracks,
                companions: others,
                crafts: other_crafts,
                wildlife,
            }
            .inscribe(&mut payload);
            ward.send_sealed(socket, cid, &payload, &mut scratch);
        }
    }
}
