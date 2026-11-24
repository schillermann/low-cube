use glam::Vec3;
use world::{CompanionPresence, DrivenCraft};

pub const NEARBY_SIGHT_METERS: f32 = 160.0;
pub const CHAT_EARSHOT_METERS: f32 = 96.0;
pub const PACKET_COMPANION_CAP: usize = 16;
pub const INTIMATE_NEIGHBOR_CAP: usize = 8;

/// Narrow contract selecting which companions and vehicles belong in a recipient's local `HostPulse`.
pub trait PulseAudience {
    fn select_for(
        &self,
        recipient_slot: u32,
        anchor_pos: Vec3,
        cursor: &mut usize,
        out_companions: &mut Vec<CompanionPresence>,
        out_crafts: &mut Vec<DrivenCraft>,
    );
}

/// Unfiltered world snapshot source excluding only the recipient's own avatar and vehicle.
pub struct WorldAudience<'a> {
    companions: &'a [CompanionPresence],
    crafts: &'a [DrivenCraft],
}

impl<'a> WorldAudience<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(companions: &'a [CompanionPresence], crafts: &'a [DrivenCraft]) -> Self {
        Self { companions, crafts }
    }
}

impl PulseAudience for WorldAudience<'_> {
    fn select_for(
        &self,
        recipient_slot: u32,
        _anchor_pos: Vec3,
        _cursor: &mut usize,
        out_companions: &mut Vec<CompanionPresence>,
        out_crafts: &mut Vec<DrivenCraft>,
    ) {
        out_companions.clear();
        out_crafts.clear();
        for &comp in self.companions {
            if !comp.same_slot(recipient_slot) {
                out_companions.push(comp);
            }
        }
        for &craft in self.crafts {
            if !craft.same_slot(recipient_slot) {
                out_crafts.push(craft);
            }
        }
    }
}

/// `PulseAudience` decorator culling all distant actors (`0 Hz` beyond `radius_meters` or on other planets).
pub struct ProximityAudience<A> {
    origin: A,
    radius_meters: f32,
}

impl<A> ProximityAudience<A> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: A, radius_meters: f32) -> Self {
        Self { origin, radius_meters }
    }
}

impl<A: PulseAudience> PulseAudience for ProximityAudience<A> {
    fn select_for(
        &self,
        recipient_slot: u32,
        anchor_pos: Vec3,
        cursor: &mut usize,
        out_companions: &mut Vec<CompanionPresence>,
        out_crafts: &mut Vec<DrivenCraft>,
    ) {
        self.origin.select_for(recipient_slot, anchor_pos, cursor, out_companions, out_crafts);
        out_companions.retain(|c| c.within_reach(anchor_pos, self.radius_meters));
        out_crafts.retain(|dc| dc.within_reach(anchor_pos, self.radius_meters));
    }
}

/// `PulseAudience` decorator enforcing an MTU-safe actor cap via intimate priority + Round-Robin fairness.
pub struct MtuBudgetAudience<A> {
    origin: A,
    intimate_cap: usize,
    packet_cap: usize,
}

impl<A> MtuBudgetAudience<A> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: A, intimate_cap: usize, packet_cap: usize) -> Self {
        Self { origin, intimate_cap, packet_cap }
    }
}

impl<A: PulseAudience> PulseAudience for MtuBudgetAudience<A> {
    fn select_for(
        &self,
        recipient_slot: u32,
        anchor_pos: Vec3,
        cursor: &mut usize,
        out_companions: &mut Vec<CompanionPresence>,
        out_crafts: &mut Vec<DrivenCraft>,
    ) {
        self.origin.select_for(recipient_slot, anchor_pos, cursor, out_companions, out_crafts);
        out_companions.sort_by(|a, b| a.distance_sq_to(anchor_pos).total_cmp(&b.distance_sq_to(anchor_pos)));
        if out_companions.len() > self.packet_cap {
            let fixed = self.intimate_cap.min(self.packet_cap).min(out_companions.len());
            let rotating_pool = out_companions.len() - fixed;
            let rotating_budget = self.packet_cap - fixed;
            let start = if rotating_pool > 0 { *cursor % rotating_pool } else { 0 };
            *cursor = start.wrapping_add(rotating_budget);
            let mut chosen = Vec::with_capacity(self.packet_cap);
            chosen.extend_from_slice(&out_companions[..fixed]);
            for offset in 0..rotating_budget {
                let idx = fixed + ((start + offset) % rotating_pool);
                chosen.push(out_companions[idx]);
            }
            *out_companions = chosen;
        }
        out_crafts.sort_by(|a, b| {
            let da = (anchor_pos - a.pose().origin()).length_squared();
            let db = (anchor_pos - b.pose().origin()).length_squared();
            da.total_cmp(&db)
        });
        if out_crafts.len() > 8 {
            out_crafts.truncate(8);
        }
    }
}
