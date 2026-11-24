use std::net::SocketAddr;
use glam::Vec3;
use world::{Callsign, SolarProportions, SurvivorWelcome};
use crate::host_wire::STRUCTURED_WELCOME_TAG;
use crate::link_state::{LinkPhase, LinkSync};
use crate::ticket::JoinTarget;

const SEALED_GRANT_SPAN: usize = 62;

/// Verified `LWEL` welcome packet parser establishing `LinkPhase::Sealed`, `WelcomeSpawn`, `EpochSync`, and `SurvivorWelcome`.
pub struct WelcomeFrame<'a> {
    src: SocketAddr,
    target: JoinTarget,
    pkt: &'a [u8],
}

impl<'a> WelcomeFrame<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(src: SocketAddr, target: JoinTarget, pkt: &'a [u8]) -> Self {
        Self { src, target, pkt }
    }

    pub fn accept(self, phase: &mut LinkPhase, sync: &mut LinkSync) -> Option<SolarProportions> {
        let pkt = self.pkt;
        if pkt.len() < 44 || &pkt[0..4] != b"LWEL" {
            return None;
        }
        let mut host_pub = [0u8; 32];
        host_pub.copy_from_slice(&pkt[4..36]);
        if let JoinTarget::Ticket(ticket) = self.target {
            if ticket.host_key() != host_pub {
                return None;
            }
        }
        let cid = u32::from_le_bytes(pkt[36..40].try_into().ok()?);
        let blocks = u32::from_le_bytes(pkt[40..44].try_into().ok()?);
        let proportions = SolarProportions::decode_tail(blocks, &pkt[44..]);
        if pkt.len() > 80 && pkt[60] == STRUCTURED_WELCOME_TAG {
            let flags = pkt[61];
            let epoch_secs = f64::from_le_bytes(pkt[62..70].try_into().ok()?);
            let mut cursor = 70;
            let mut parsed_spawn = None;
            if flags & 0x01 != 0 {
                if pkt.len() < cursor + 12 {
                    return None;
                }
                let sx = f32::from_le_bytes(pkt[cursor..cursor + 4].try_into().ok()?);
                let sy = f32::from_le_bytes(pkt[cursor + 4..cursor + 8].try_into().ok()?);
                let sz = f32::from_le_bytes(pkt[cursor + 8..cursor + 12].try_into().ok()?);
                let pt = Vec3::new(sx, sy, sz);
                if pt.is_finite() {
                    parsed_spawn = Some(pt);
                }
                cursor += 12;
            }
            let mut parsed_welcome = None;
            if flags & 0x02 != 0 {
                let (welcome, used) = SurvivorWelcome::decode(&pkt[cursor..])?;
                parsed_welcome = Some(welcome);
                cursor += used;
            }
            let assigned_callsign = if flags & 0x04 != 0 {
                let (cs, used) = Callsign::decode(&pkt[cursor..])?;
                cursor += used;
                cs
            } else {
                Callsign::unnamed()
            };
            let sealed_grant = if flags & 0x08 != 0 {
                if pkt.len() < cursor + SEALED_GRANT_SPAN {
                    return None;
                }
                Some(&pkt[cursor..cursor + SEALED_GRANT_SPAN])
            } else {
                None
            };
            let LinkPhase::Knocking { ref mut passport, .. } = *phase else {
                return None;
            };
            let (seal, callsign) = passport.accept_welcome(cid, host_pub, self.src, assigned_callsign, sealed_grant)?;
            *phase = LinkPhase::Sealed { companion_id: cid, callsign, seal, peer: self.src };
            sync.assign_slot(cid);
            sync.assign_welcome_epoch(epoch_secs);
            if let Some(pt) = parsed_spawn {
                sync.assign_spawn_point(pt, cid);
            }
            if let Some(welcome) = parsed_welcome {
                sync.assign_survivor_welcome(welcome, cid);
            }
            return Some(proportions);
        }
        let LinkPhase::Knocking { ref mut passport, .. } = *phase else {
            return None;
        };
        let (seal, callsign) = passport.accept_welcome(cid, host_pub, self.src, Callsign::unnamed(), None)?;
        *phase = LinkPhase::Sealed { companion_id: cid, callsign, seal, peer: self.src };
        sync.assign_slot(cid);
        if pkt.len() >= 72 {
            let sx = f32::from_le_bytes(pkt[60..64].try_into().ok()?);
            let sy = f32::from_le_bytes(pkt[64..68].try_into().ok()?);
            let sz = f32::from_le_bytes(pkt[68..72].try_into().ok()?);
            let pt = Vec3::new(sx, sy, sz);
            if pt.is_finite() {
                sync.assign_spawn_point(pt, cid);
            }
        }
        let epoch_slice = if pkt.len() == 68 {
            Some(&pkt[60..68])
        } else if pkt.len() >= 80 {
            Some(&pkt[72..80])
        } else {
            None
        };
        if let Some(ep_bytes) = epoch_slice {
            let epoch_secs = f64::from_le_bytes(ep_bytes.try_into().ok()?);
            sync.assign_welcome_epoch(epoch_secs);
        }
        Some(proportions)
    }
}
