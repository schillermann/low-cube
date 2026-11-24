use std::net::SocketAddr;
use glam::Vec3;
use world::SolarProportions;
use crate::link_state::{LinkPhase, LinkSync};
use crate::ticket::JoinTarget;

/// Verified `LWEL` welcome packet parser establishing `LinkPhase::Sealed`, `WelcomeSpawn`, and `EpochSync`.
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
        let LinkPhase::Knocking { ref keys, .. } = *phase else {
            return None;
        };
        let seal = keys.establish(host_pub, true);
        *phase = LinkPhase::Sealed { companion_id: cid, seal, peer: self.src };
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
