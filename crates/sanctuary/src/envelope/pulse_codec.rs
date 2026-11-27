use world::{
    CompanionPresence, CosmicFrame, CraftIdentity, DrivenCraft, PilotCraft, SkySignal, VesselBerth,
};
use super::{CrackNotice, SanctuaryEnvelope};

const SKY_WATCH_TAG: u8 = 0xD5;
const BERTH_TRAILER_TAG: u8 = 0xB7;
const SIGNAL_TRAILER_TAG: u8 = 0xD9;

/// Focused binary wire codec for `GuestPulse` and `HostPulse` envelopes (`0`-byte trailers when idle).
pub(super) struct PulseCodec;

impl PulseCodec {
    pub(super) fn inscribe_guest(
        ack_seq: u32,
        presence: &CompanionPresence,
        craft: &PilotCraft,
        frame: CosmicFrame,
        berth: VesselBerth,
        sky_watch: bool,
        out: &mut Vec<u8>,
    ) {
        out.push(1);
        out.extend_from_slice(&ack_seq.to_le_bytes());
        presence.inscribe(out);
        craft.inscribe(out);
        if !frame.earth_centered() || berth.aboard_any() || sky_watch {
            frame.inscribe(out);
            if berth.aboard_any() {
                berth.inscribe(out);
            }
            if sky_watch {
                out.push(SKY_WATCH_TAG);
            }
        }
    }

    pub(super) fn decode_guest(bytes: &[u8]) -> Option<SanctuaryEnvelope> {
        if bytes.len() < 5 {
            return None;
        }
        let ack_seq = u32::from_le_bytes(bytes[1..5].try_into().ok()?);
        let (presence, used) = CompanionPresence::decode(&bytes[5..])?;
        let tail = &bytes[5 + used..];
        let (craft, craft_used) = if tail.is_empty() {
            (PilotCraft::Pedestrian, 0)
        } else {
            PilotCraft::decode(tail)?
        };
        let frame_tail = &tail[craft_used..];
        let (frame, berth, sky_watch) = if frame_tail.is_empty() {
            (CosmicFrame::earth(), VesselBerth::planetary(), false)
        } else {
            let (f, f_used) = CosmicFrame::decode(frame_tail)?;
            let b_tail = &frame_tail[f_used..];
            match b_tail.first().copied() {
                None => (f, VesselBerth::planetary(), false),
                Some(SKY_WATCH_TAG) => (f, VesselBerth::planetary(), true),
                Some(_) => {
                    let (b, b_used) = VesselBerth::decode(b_tail)?;
                    let watch = b_tail.get(b_used).copied() == Some(SKY_WATCH_TAG);
                    (f, b, watch)
                }
            }
        };
        Some(SanctuaryEnvelope::GuestPulse { ack_seq, presence, craft, frame, berth, sky_watch })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn inscribe_host(
        total_facts: u32,
        cracks: &[CrackNotice],
        companions: &[CompanionPresence],
        crafts: &[DrivenCraft],
        berths: &[(u32, CraftIdentity)],
        signals: &[SkySignal],
        out: &mut Vec<u8>,
    ) {
        out.push(4);
        out.extend_from_slice(&total_facts.to_le_bytes());
        out.push(cracks.len().min(16) as u8);
        for crack in cracks.iter().take(16) {
            crack.inscribe(out);
        }
        out.push(companions.len().min(64) as u8);
        for comp in companions.iter().take(64) {
            comp.inscribe(out);
        }
        let count_idx = out.len();
        out.push(0);
        let mut written = 0u8;
        let mut scratch = Vec::new();
        for craft in crafts.iter().take(16) {
            scratch.clear();
            craft.inscribe(&mut scratch);
            if out.len() + scratch.len() > 1_200 {
                break;
            }
            out.extend_from_slice(&scratch);
            written += 1;
        }
        out[count_idx] = written;
        if !berths.is_empty() {
            let bcount = berths.len().min(64);
            out.push(BERTH_TRAILER_TAG);
            out.push(bcount as u8);
            for &(slot, id) in berths.iter().take(bcount) {
                out.extend_from_slice(&slot.to_le_bytes());
                id.inscribe(out);
            }
        }
        if !signals.is_empty() {
            let scount = signals.len().min(8);
            out.push(SIGNAL_TRAILER_TAG);
            out.push(scount as u8);
            for sig in signals.iter().take(scount) {
                sig.inscribe(out);
            }
        }
    }

    pub(super) fn decode_host(bytes: &[u8]) -> Option<SanctuaryEnvelope> {
        if bytes.len() < 7 {
            return None;
        }
        let total_facts = u32::from_le_bytes(bytes[1..5].try_into().ok()?);
        let crack_count = *bytes.get(5)? as usize;
        let mut cursor = 6;
        let mut cracks = Vec::with_capacity(crack_count.min(16));
        for _ in 0..crack_count {
            let (crack, used) = CrackNotice::decode(&bytes[cursor..])?;
            cracks.push(crack);
            cursor += used;
        }
        let comp_count = *bytes.get(cursor)? as usize;
        cursor += 1;
        let mut companions = Vec::with_capacity(comp_count.min(64));
        for _ in 0..comp_count {
            let (comp, used) = CompanionPresence::decode(&bytes[cursor..])?;
            companions.push(comp);
            cursor += used;
        }
        let mut crafts = Vec::new();
        if cursor < bytes.len() {
            let craft_count = *bytes.get(cursor)? as usize;
            cursor += 1;
            crafts.reserve(craft_count.min(16));
            for _ in 0..craft_count {
                let (craft, used) = DrivenCraft::decode(&bytes[cursor..])?;
                crafts.push(craft);
                cursor += used;
            }
        }
        let mut berths = Vec::new();
        if bytes.get(cursor).copied() == Some(BERTH_TRAILER_TAG) {
            let bcount = *bytes.get(cursor + 1)? as usize;
            cursor += 2;
            if bytes.len() < cursor + bcount * 12 {
                return None;
            }
            berths.reserve(bcount.min(64));
            for _ in 0..bcount {
                let slot = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().ok()?);
                let (id, _) = CraftIdentity::decode(&bytes[cursor + 4..cursor + 12])?;
                berths.push((slot, id));
                cursor += 12;
            }
        }
        let mut signals = Vec::new();
        if bytes.get(cursor).copied() == Some(SIGNAL_TRAILER_TAG) {
            let scount = *bytes.get(cursor + 1)? as usize;
            if (1..=8).contains(&scount) && bytes.len() >= cursor + 2 + scount * SkySignal::WIRE_SPAN {
                let mut probe = cursor + 2;
                let mut parsed = Vec::with_capacity(scount);
                let mut ok = true;
                for _ in 0..scount {
                    if let Some((sig, used)) = SkySignal::decode(&bytes[probe..]) {
                        parsed.push(sig);
                        probe += used;
                    } else {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    signals = parsed;
                }
            }
        }
        Some(SanctuaryEnvelope::HostPulse { total_facts, cracks, companions, crafts, berths, signals })
    }
}
