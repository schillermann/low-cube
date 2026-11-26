use glam::Vec3;
use super::transfer_arc::{ARC_NODES, FlipBurnReadout, TransferArc};

/// Polymorphic Newtonian thrust-lead guidance vector eliminating boolean mode flags.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BurnVector {
    ForwardLead(Vec3),
    RetroBrake(Vec3),
}

impl BurnVector {
    /// Presents `(unit_burn_dir, hud_color, cue_caption)` against `nose_axis` without getters.
    pub fn present<R>(
        self,
        nose_axis: Vec3,
        f: impl FnOnce(Vec3, Vec3, &'static str) -> R,
    ) -> R {
        let (raw_dir, mode_col, mode_cue) = match self {
            Self::ForwardLead(d) => (d, Vec3::new(1.00, 0.84, 0.24), "BURN"),
            Self::RetroBrake(d) => (d, Vec3::new(1.00, 0.56, 0.22), "RETRO"),
        };
        let burn_unit = raw_dir.normalize_or_zero();
        let nose_unit = nose_axis.normalize_or_zero();
        if nose_unit.dot(burn_unit) >= 0.9985 {
            f(burn_unit, Vec3::new(0.36, 1.00, 0.76), "LOCKED")
        } else {
            f(burn_unit, mode_col, mode_cue)
        }
    }
}

/// Immutable spacecraft position, nose axis, and inertial velocity packet (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct FlightKinematics {
    ship_pos: Vec3,
    nose_axis: Vec3,
    velocity: Vec3,
}

impl FlightKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ship_pos: Vec3, nose_axis: Vec3, velocity: Vec3) -> Self {
        Self { ship_pos, nose_axis, velocity }
    }

    /// Transfers `(ship_pos, nose_axis, velocity)` to a HUD collaborator without getters.
    pub fn transfer<R>(self, f: impl FnOnce(Vec3, Vec3, Vec3) -> R) -> R {
        f(self.ship_pos, self.nose_axis, self.velocity)
    }

    /// Resolves Newtonian drift-cancelling `BurnVector` lead guidance along `arc`.
    pub fn burn_guidance(self, arc: &TransferArc) -> BurnVector {
        let fb = arc.sample_flip_burn(self.ship_pos, self.velocity, self.nose_axis);
        let mut aim_pt = self.ship_pos;
        arc.feed_nodes(|nodes, _| {
            let mut closest_idx = 0usize;
            let mut best_sq = f32::MAX;
            for (idx, pt) in nodes.iter().enumerate() {
                let d_sq = (*pt - self.ship_pos).length_squared();
                if d_sq < best_sq {
                    best_sq = d_sq;
                    closest_idx = idx;
                }
            }
            aim_pt = nodes[(closest_idx + 5).min(ARC_NODES - 1)];
        });
        let raw_des = aim_pt - self.ship_pos;
        let u_des = if raw_des.length_squared() > 1e-4 {
            raw_des.normalize()
        } else if self.nose_axis.length_squared() > 1e-4 {
            self.nose_axis.normalize()
        } else {
            Vec3::Y
        };
        let v_par = self.velocity.dot(u_des);
        let v_lat = self.velocity - u_des * v_par;
        let retro = matches!(fb, FlipBurnReadout::FlipNow | FlipBurnReadout::BrakeBurn(_));
        if retro && self.velocity.length_squared() > 1.0 {
            let brake_vec = -u_des * v_par.abs().max(12.0) - v_lat * 1.85;
            let dir = brake_vec.normalize_or_zero();
            let fallback = (-self.velocity).normalize_or_zero();
            return BurnVector::RetroBrake(if dir.length_squared() > 0.5 { dir } else { fallback });
        }
        let lead_vec = u_des * v_par.abs().max(26.0) - v_lat * 1.85;
        let dir = lead_vec.normalize_or_zero();
        BurnVector::ForwardLead(if dir.length_squared() > 0.5 { dir } else { u_des })
    }
}
