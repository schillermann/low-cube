use glam::Vec3;
use world::{BlockMedium, BlockMold, CardinalTurn, MoldArchetype, PlanetaryWorld};

/// Encapsulated local forensic trace buffer for `FusionTorch` pad vitrification and crash `BlackBoxPod` deposition (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub(super) struct FlightResidue {
    scorch_ring: [(i32, i32, i32); 4],
    ring_cursor: u8,
    pending_scorch: (Vec3, Vec3, bool),
    pending_crash: (Vec3, Vec3, bool),
}

impl FlightResidue {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        scorch_ring: [(i32, i32, i32); 4],
        ring_cursor: u8,
        pending_scorch: (Vec3, Vec3, bool),
        pending_crash: (Vec3, Vec3, bool),
    ) -> Self {
        Self { scorch_ring, ring_cursor, pending_scorch, pending_crash }
    }

    pub(super) const fn clean() -> Self {
        Self::new([(i32::MIN, i32::MIN, i32::MIN); 4], 0, (Vec3::ZERO, Vec3::Y, false), (Vec3::ZERO, Vec3::Y, false))
    }

    pub(super) fn note_fusion_exhaust(&mut self, fusion_lit: bool, origin: Vec3, world: &dyn PlanetaryWorld) {
        if !fusion_lit {
            return;
        }
        world.surface_contact(origin, 0.0).resolve_crust((), |ground, surf_up| {
            let alt = (origin - ground).dot(surf_up);
            if !(0.45..=3.5).contains(&alt) {
                return;
            }
            let cell = (
                (ground.x * 2.0).round() as i32,
                (ground.y * 2.0).round() as i32,
                (ground.z * 2.0).round() as i32,
            );
            if self.scorch_ring.contains(&cell) {
                return;
            }
            let slot = (self.ring_cursor as usize) & 3;
            self.scorch_ring[slot] = cell;
            self.ring_cursor = self.ring_cursor.wrapping_add(1);
            self.pending_scorch = (ground, surf_up, true);
        });
    }

    pub(super) fn note_touchdown(
        &mut self,
        origin: Vec3,
        pre_vel: Vec3,
        post_vel: Vec3,
        world: &dyn PlanetaryWorld,
    ) {
        world.surface_contact(origin, 0.0).resolve_crust((), |ground, surf_up| {
            let impact_v = pre_vel.dot(surf_up);
            let settled_v = post_vel.dot(surf_up);
            if impact_v < -26.0 && settled_v > -8.0 {
                self.pending_crash = (ground, surf_up, true);
            }
        });
    }

    pub(super) fn drain_into(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let mut altered = false;
        let ref_axis = |up: Vec3| {
            let r = if up.x.abs() < 0.85 { Vec3::X } else { Vec3::Z };
            r.cross(up).normalize_or_zero()
        };
        if self.pending_scorch.2 {
            let (ground, up, _) = self.pending_scorch;
            self.pending_scorch.2 = false;
            let t = ref_axis(up);
            let b = up.cross(t).normalize_or_zero();
            let mold = BlockMold::with_medium(MoldArchetype::HalfSlab, CardinalTurn::Forward, BlockMedium::VitrifiedBasalt);
            for off in [Vec3::ZERO, t * 1.0, -t * 1.0, b * 1.0, -b * 1.0] {
                let p = ground + off;
                if world.place_molded_surface_block(p + up * 0.85, -up, p + t * 2.5, mold) {
                    altered = true;
                    break;
                }
            }
        }
        if !self.pending_crash.2 {
            return altered;
        }
        let (ground, up, _) = self.pending_crash;
        self.pending_crash.2 = false;
        let tangent = ref_axis(up);
        let bitan = up.cross(tangent).normalize_or_zero();
        let pod_mold = BlockMold::cube().in_medium(BlockMedium::BlackBoxPod);
        for off in [tangent * 1.0, -tangent * 1.0, Vec3::ZERO] {
            let probe = ground + off;
            if world.place_molded_surface_block(probe + up * 0.95, -up, probe + bitan * 2.5, pod_mold) {
                return true;
            }
        }
        altered
    }
}
