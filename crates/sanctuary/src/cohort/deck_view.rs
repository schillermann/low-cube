use std::collections::HashMap;
use glam::Vec3;
use world::{
    CompanionPresence, CosmicFrame, CraftIdentity, CraftPose, DeckFrame, DrivenCraft,
    GravityBubble, PilotCraft,
};
use crate::audience::NEARBY_SIGHT_METERS;
use crate::aura::HostAura;
use crate::fleet_ledger::FleetLedger;
use crate::spatial_chronicle::HorizonStream;
use crate::ward::GuestWard;

/// Read-only view projecting guest companions, crafts, and decks into the local `--host` player's frame (`<= 4` fields).
pub(super) struct HostDeckView<'a> {
    wards: &'a HashMap<u32, (GuestWard, HorizonStream)>,
    render_secs: f64,
}

impl<'a> HostDeckView<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(wards: &'a HashMap<u32, (GuestWard, HorizonStream)>, render_secs: f64) -> Self {
        Self { wards, render_secs }
    }

    pub fn active_ship(&self, craft_id: CraftIdentity, aura: &HostAura) -> Option<(CosmicFrame, CraftPose)> {
        if let Some(pose) = aura.host_hull_pose(craft_id) {
            return Some((aura.host_frame(), pose));
        }
        self.wards.values().find_map(|(w, _)| match w.craft_at(self.render_secs) {
            PilotCraft::Driving(dc) if dc.identity() == craft_id => Some((w.cosmic_frame(), dc.pose())),
            _ => None,
        })
    }

    pub fn companions(&self, aura: &HostAura, bubble: GravityBubble, out: &mut Vec<CompanionPresence>) {
        let host_frame = aura.host_frame();
        let host_anchor = aura.occupied_position();
        let reach_sq = f64::from(NEARBY_SIGHT_METERS * NEARBY_SIGHT_METERS);
        out.clear();
        for (ward, _) in self.wards.values() {
            let (wf, p) = if let Some(id) = ward.berth().craft_id() {
                let Some((sf, hull)) = self.active_ship(id, aura) else { continue };
                (sf, ward.presence_at_with_hull(self.render_secs, Some(hull)))
            } else {
                (ward.cosmic_frame(), ward.presence_at(self.render_secs))
            };
            let pos = p.transfer(|_, kin, _, _| kin.transfer(|pt, _, _, _, _| pt));
            if wf == host_frame {
                if host_anchor.is_none_or(|anchor| p.within_reach(anchor, NEARBY_SIGHT_METERS)) {
                    out.push(p);
                }
            } else if let Some(anchor) = host_anchor {
                let mapped = bubble.map_position(wf, pos, host_frame);
                if (mapped - anchor.as_dvec3()).length_squared() <= reach_sq {
                    out.push(p.with_position(mapped.as_vec3()));
                }
            }
        }
    }

    pub fn crafts(
        &self,
        host_frame: CosmicFrame,
        host_anchor: Option<Vec3>,
        bubble: GravityBubble,
        out: &mut Vec<DrivenCraft>,
    ) {
        let reach_sq = 2_400.0_f64 * 2_400.0_f64;
        out.clear();
        for (ward, _) in self.wards.values() {
            let wf = ward.cosmic_frame();
            let PilotCraft::Driving(dc) = ward.craft_at(self.render_secs) else { continue };
            if wf == host_frame {
                if host_anchor.is_none_or(|anchor| dc.within_reach(anchor, 2_400.0)) {
                    out.push(dc);
                }
            } else if let Some(anchor) = host_anchor {
                let mapped = bubble.map_position(wf, dc.pose().origin(), host_frame);
                if (mapped - anchor.as_dvec3()).length_squared() <= reach_sq {
                    out.push(dc.with_origin(mapped.as_vec3()));
                }
            }
        }
    }

    pub fn constrain_crafts(
        &self,
        fleet: &FleetLedger,
        host_frame: CosmicFrame,
        position: Vec3,
        player_height: f32,
        up: Vec3,
    ) -> Option<Vec3> {
        let mut best: Option<Vec3> = None;
        for (ward, _) in self.wards.values().filter(|(w, _)| w.cosmic_frame() == host_frame) {
            if let PilotCraft::Driving(craft) = ward.craft_at(self.render_secs) {
                if craft.within_reach(position, 32.0) {
                    if let Some(top) = fleet.constrain_crafts(std::slice::from_ref(&craft), position, player_height) {
                        if best.is_none_or(|prev| top.dot(up) > prev.dot(up)) {
                            best = Some(top);
                        }
                    }
                }
            }
        }
        best
    }

    pub fn sample_deck_at(
        &self,
        fleet: &FleetLedger,
        host_frame: CosmicFrame,
        position: Vec3,
        player_height: f32,
    ) -> Option<DeckFrame> {
        for (ward, _) in self.wards.values().filter(|(w, _)| w.cosmic_frame() == host_frame) {
            if let PilotCraft::Driving(craft) = ward.craft_at(self.render_secs) {
                if craft.within_reach(position, 32.0) {
                    if let Some(deck) = fleet.sample_deck_at(std::slice::from_ref(&craft), position, player_height) {
                        return Some(deck);
                    }
                }
            }
        }
        None
    }
}
