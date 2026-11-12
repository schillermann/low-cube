use super::mesh::RigGeometry;
use super::{BLOCK_STEP, MAX_HOIST_STEPS};

const MIN_HOIST_STEPS: i32 = 0;
const INITIAL_HOIST_STEPS: i32 = 3;
const INITIAL_REACH_STEPS: i32 = 2;
const LAYER_MATCH_TOLERANCE: f32 = 0.30;

/// Geometric posture of a cantilever crane's mast, hoist carriage, and telescoping boom.
#[derive(Clone, Copy, Debug)]
pub struct RigArm {
    ground_top: f32,
    hoist_steps: i32,
    reach_steps: i32,
    turn: u8,
}

impl RigArm {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ground_top: f32, hoist_steps: i32, reach_steps: i32, turn: u8) -> Self {
        Self { ground_top, hoist_steps, reach_steps, turn }
    }

    pub const fn deployed(ground_top: f32, turn: u8) -> Self {
        Self::new(ground_top, INITIAL_HOIST_STEPS, INITIAL_REACH_STEPS, turn & 3)
    }

    pub fn above_base(&self, floor_elev: f32) -> bool {
        floor_elev > self.ground_top + 0.05
    }

    pub const fn hoisted(&self) -> bool {
        self.hoist_steps > 0
    }

    pub fn reset(&mut self) {
        self.hoist_steps = INITIAL_HOIST_STEPS;
        self.reach_steps = INITIAL_REACH_STEPS;
    }

    pub fn reset_reach(&mut self) {
        self.reach_steps = INITIAL_REACH_STEPS;
    }

    pub fn advance_toward_craft(&mut self, mast_u: i32, mast_v: i32, occupied_at: impl Fn(i32, i32) -> bool) {
        let (gu, gv) = self.gripped_cell(mast_u, mast_v);
        if occupied_at(gu, gv) {
            return;
        }
        let (du, dv) = self.step_vectors();
        for step in (self.reach_steps + 1)..=(self.reach_steps + 8) {
            if occupied_at(mast_u + du * step, mast_v + dv * step) {
                self.reach_steps = step;
                return;
            }
        }
    }

    pub const fn step_vectors(&self) -> (i32, i32) {
        match self.turn & 3 {
            0 => (1, 0),
            1 => (0, 1),
            2 => (-1, 0),
            _ => (0, -1),
        }
    }

    pub const fn hoist_elevation(&self) -> f32 {
        self.ground_top + (self.hoist_steps as f32) * BLOCK_STEP
    }

    pub const fn mast_ceiling(&self) -> f32 {
        self.hoist_elevation() + 0.65
    }

    pub const fn geometry(&self) -> RigGeometry {
        RigGeometry::new(self.ground_top, self.hoist_elevation(), self.reach_steps, self.turn)
    }

    pub const fn gripped_cell(&self, mast_u: i32, mast_v: i32) -> (i32, i32) {
        let (du, dv) = self.step_vectors();
        (mast_u + du * self.reach_steps, mast_v + dv * self.reach_steps)
    }

    pub const fn claw_slot(&self, mast_u: i32, mast_v: i32) -> (i32, i32) {
        let (du, dv) = self.step_vectors();
        let step = self.reach_steps - 1;
        (mast_u + du * step, mast_v + dv * step)
    }

    pub const fn occupies_boom_cell(&self, mast_u: i32, mast_v: i32, u: i32, v: i32) -> bool {
        let (du, dv) = self.step_vectors();
        let mut step = 0;
        while step <= self.reach_steps {
            if u == mast_u + du * step && v == mast_v + dv * step {
                return true;
            }
            step += 1;
        }
        false
    }

    pub const fn near_boom(&self, mast_u: i32, mast_v: i32, u: i32, v: i32) -> bool {
        let (du, dv) = self.step_vectors();
        let mut step = 0;
        while step <= self.reach_steps {
            let (bu, bv) = (mast_u + du * step, mast_v + dv * step);
            if (u - bu).abs() <= 1 && (v - bv).abs() <= 1 {
                return true;
            }
            step += 1;
        }
        false
    }

    pub fn occupies_mast_layer(&self, layer_elev: f32) -> bool {
        layer_elev >= self.ground_top - 0.05 && layer_elev <= self.mast_ceiling()
    }

    pub fn constrain_mast_support(&self, support_h: f32, foot_elev: f32) -> f32 {
        if foot_elev + 0.08 <= self.ground_top {
            support_h
        } else {
            support_h.max(self.hoist_elevation() + BLOCK_STEP)
        }
    }

    pub fn clamped_step_delta(&self, delta_steps: i32) -> f32 {
        let target = (self.hoist_steps + delta_steps).clamp(MIN_HOIST_STEPS, MAX_HOIST_STEPS);
        ((target - self.hoist_steps) as f32) * BLOCK_STEP
    }

    pub fn apply_vertical_shift(&mut self, dz: f32) {
        let delta_steps = (dz / BLOCK_STEP).round() as i32;
        self.hoist_steps = (self.hoist_steps + delta_steps).clamp(MIN_HOIST_STEPS, MAX_HOIST_STEPS);
    }

    pub fn retract_for_placed_block(
        &mut self,
        mast_u: i32,
        mast_v: i32,
        placed_u: i32,
        placed_v: i32,
        placed_floor: f32,
    ) -> Option<(i32, i32)> {
        if (placed_floor - self.hoist_elevation()).abs() > BLOCK_STEP + LAYER_MATCH_TOLERANCE
            || (placed_u, placed_v) != self.claw_slot(mast_u, mast_v)
        {
            return None;
        }
        if self.reach_steps > 1 {
            self.reach_steps -= 1;
            None
        } else {
            let (du, dv) = self.step_vectors();
            Some((mast_u - du, mast_v - dv))
        }
    }
}
