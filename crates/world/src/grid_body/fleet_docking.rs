use glam::Vec3;
use crate::relief::CraftIdentity;
use super::docking::DockingClamp;
use super::GridBody;

/// Collaborator coupling, propagating, and separating `2×2×1` `DockingRing`-linked `GridBody`s (`1` field).
pub struct FleetDockyard<'a> {
    bodies: &'a mut Vec<GridBody>,
}

impl<'a> FleetDockyard<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(bodies: &'a mut Vec<GridBody>) -> Self {
        Self { bodies }
    }

    pub fn couple_nearby(&mut self) -> bool {
        if self.bodies.len() < 2 {
            return false;
        }
        let clamp = DockingClamp::orbital();
        let mut coupled_any = false;
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                if !self.bodies[i].mooring().free_moving() || !self.bodies[j].mooring().free_moving() {
                    continue;
                }
                let (mut ports_i, mut ports_j) = (Vec::new(), Vec::new());
                self.bodies[i].collect_docking_ports(&mut ports_i);
                self.bodies[j].collect_docking_ports(&mut ports_j);
                let mut match_found = None;
                'port_scan: for &pi in &ports_i {
                    for &pj in &ports_j {
                        if let Some(res) = clamp.try_couple(
                            pi,
                            self.bodies[i].motion(),
                            self.bodies[i].total_mass_kg(),
                            pj,
                            self.bodies[j].motion(),
                            self.bodies[j].total_mass_kg(),
                        ) {
                            match_found = Some(res);
                            break 'port_scan;
                        }
                    }
                }
                if let Some((host_m, link, guest_m)) = match_found {
                    self.bodies[i].apply_motion(host_m);
                    self.bodies[j].lock_to_port(link, guest_m);
                    coupled_any = true;
                }
            }
        }
        coupled_any
    }

    pub fn propagate_docked(&mut self) {
        for _pass in 0..2 {
            for idx in 0..self.bodies.len() {
                let Some(link) = self.bodies[idx].mooring().docked_link() else { continue };
                let Some(host) = self.bodies.iter().find(|b| b.identity() == link.host()) else { continue };
                let host_motion = host.motion();
                let cur_pose = self.bodies[idx].pose();
                let synced = link.synced_motion(host_motion, cur_pose);
                self.bodies[idx].apply_motion(synced);
            }
        }
    }

    pub fn undock(&mut self, guest_id: CraftIdentity) -> bool {
        let Some(guest_idx) = self.bodies.iter().position(|b| b.identity() == guest_id) else {
            return false;
        };
        let Some(link) = self.bodies[guest_idx].mooring().docked_link() else {
            return false;
        };
        let sep_dir = self
            .bodies
            .iter()
            .find(|b| b.identity() == link.host())
            .map_or(Vec3::Y, |h| {
                (self.bodies[guest_idx].pose().origin() - h.pose().origin()).normalize_or_zero()
            });
        self.bodies[guest_idx].undock(sep_dir * 0.40)
    }
}
