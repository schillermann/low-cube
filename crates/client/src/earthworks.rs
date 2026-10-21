use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use crate::attire::{
    BeltSatchel, BodyPose, FacetedSolid, HeldRelic, LimbFrame, SightMode, StrideRhythm,
    TaperedPrism, YouthAttire,
};

const EXCAVATION_THRESHOLD: u32 = 8;

/// Immutable dual-hand block inventory, satchel inspection state, per-hand block molds, and excavation progress ("Tell, Don't Ask").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalmEarthworks {
    satchel: BeltSatchel,
    dig_progress: u32,
    held_mask: u8,
    dig_site: Vec3,
    splash_active: bool,
}

impl PalmEarthworks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        satchel: BeltSatchel, dig_progress: u32, held_mask: u8, dig_site: Vec3, splash_active: bool,
    ) -> Self {
        Self { satchel, dig_progress, held_mask, dig_site, splash_active }
    }

    /// Secondary constructor initializing empty hands and satchel with forward full-cube molds.
    pub const fn empty() -> Self { Self::new(BeltSatchel::empty(), 0, 0, Vec3::ZERO, false) }

    pub const fn satchel_open(&self) -> bool { self.satchel.open() }

    pub fn mold_next_form(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.map_mold(attire, BlockMold::next_form), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    pub fn mold_prev_form(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.map_mold(attire, BlockMold::prev_form), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    pub fn turn_mold_clockwise(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.map_mold(attire, BlockMold::turn_clockwise), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    pub fn turn_mold_counter_clockwise(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.map_mold(attire, BlockMold::turn_counter_clockwise), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    /// Dispatches mold, pocket stow/draw (`Q`/`E`), satchel toggle (`Tab`/`B`), and `WASD` satchel navigation keys.
    pub fn handle_key(self, code: KeyCode, attire: YouthAttire) -> (Self, YouthAttire, bool) {
        match code {
            KeyCode::KeyR => (self.mold_next_form(attire), attire, true),
            KeyCode::KeyV => (self.turn_mold_clockwise(attire), attire, true),
            KeyCode::KeyC => (self.turn_mold_counter_clockwise(attire), attire, true),
            KeyCode::Tab | KeyCode::KeyB => {
                let (next_satchel, ok) = self.satchel.reconcile_attire(attire).toggle_open();
                (Self::new(next_satchel, 0, 0, Vec3::ZERO, false), attire, ok)
            }
            KeyCode::KeyQ | KeyCode::KeyE => {
                let (next_satchel, next_attire, ok) = self.satchel.toggle_pocket(if code == KeyCode::KeyE { 1.0 } else { -1.0 }, attire);
                (Self::new(next_satchel, 0, 0, Vec3::ZERO, false), next_attire, ok)
            }
            KeyCode::KeyA | KeyCode::ArrowLeft | KeyCode::KeyD | KeyCode::ArrowRight | KeyCode::KeyW | KeyCode::ArrowUp | KeyCode::KeyS | KeyCode::ArrowDown => {
                let dx = if matches!(code, KeyCode::KeyA | KeyCode::ArrowLeft) { -1 } else if matches!(code, KeyCode::KeyD | KeyCode::ArrowRight) { 1 } else { 0 };
                let dy = if matches!(code, KeyCode::KeyW | KeyCode::ArrowUp) { -1 } else if matches!(code, KeyCode::KeyS | KeyCode::ArrowDown) { 1 } else { 0 };
                let (next_satchel, ok) = self.satchel.navigate(dx, dy);
                (Self::new(next_satchel, self.dig_progress, self.held_mask, self.dig_site, self.splash_active), attire, ok)
            }
            _ => (self, attire, false),
        }
    }

    pub fn emit_attire(&self, attire: &YouthAttire, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        attire.emit_satchel(pose, sight, self.satchel, target);
    }

    pub fn reconcile_attire(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.reconcile_attire(attire), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    pub fn merge_site_relics(self, attire: YouthAttire, site_pair: (HeldRelic, HeldRelic)) -> (Self, YouthAttire) {
        let (next_satchel, updated) = self.satchel.merge_site_relics(attire, site_pair);
        (Self::new(next_satchel, self.dig_progress, self.held_mask, self.dig_site, self.splash_active), updated)
    }

    pub fn trigger_hand_swing(attire: YouthAttire, rhythm: StrideRhythm, side: f32) -> StrideRhythm {
        let (right, left) = attire.held_pair();
        if right.free_palm() && left.free_palm() { return rhythm.trigger_right_strike().trigger_left_strike(); }
        let (active, other) = if side > 0.0 { (right, left) } else { (left, right) };
        if active.gripped() || active.free_palm() {
            if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() }
        } else if other.free_palm() {
            if side > 0.0 { rhythm.trigger_left_strike() } else { rhythm.trigger_right_strike() }
        } else {
            rhythm
        }
    }

    pub fn emit_debris(&self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        let t = rhythm.strike_progress();
        if !self.splash_active || t <= 0.01 { return; }
        let right = view_dir.cross(surface_up).normalize_or_zero();
        let frame = LimbFrame::new(self.dig_site, right, surface_up, surface_up.cross(right).normalize_or_zero());
        let spray = [
            (-0.85, 0.42, 0.30, 0.24, 0.019, 0.00), (0.78, 0.55, 0.34, 0.28, 0.022, 0.04),
            (-0.35, -0.88, 0.26, 0.20, 0.017, 0.08), (0.46, -0.82, 0.32, 0.25, 0.020, 0.02),
            (-0.92, -0.22, 0.28, 0.22, 0.016, 0.06), (0.90, -0.18, 0.35, 0.26, 0.018, 0.10),
        ];
        for (i, &(dx, dz, reach, apex, rad, delay)) in spray.iter().enumerate() {
            let tk = ((t - delay) / (1.0 - delay)).clamp(0.0, 1.0);
            if tk <= 0.0 || tk >= 1.0 { continue; }
            let (dist, lift) = (0.02 + reach * tk, 4.0 * apex * tk * (1.0 - tk));
            let col = if i % 2 == 0 { Vec3::new(0.44, 0.31, 0.19) } else { Vec3::new(0.32, 0.22, 0.13) };
            let p = Vec3::new(dx * dist, lift, dz * dist);
            TaperedPrism::new(4, p, p + Vec3::Y * (rad * 1.5), rad, rad, rad * 0.75, rad * 0.75, col).emit(&frame, true, target);
        }
    }

    fn locate_dig_site(eye: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> Vec3 {
        let (dir, up) = (view_dir.normalize_or_zero(), world.surface_up_at(eye));
        for step in 1..=85 {
            let sample = eye + dir * ((step as f32) * 0.04);
            let ground = world.constrain_to_surface(sample, 0.0);
            if (sample - ground).dot(up) <= 0.02 { return ground; }
        }
        world.constrain_to_surface(eye + dir, 0.0)
    }

    pub fn release_hand(self, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld) -> (Self, bool) {
        let cleared = if self.dig_progress > 0 { world.loosen_surface(eye, view_dir, 0) } else { false };
        (Self::new(self.satchel, 0, 0, self.dig_site, false), cleared)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn continue_digging(
        self, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        if self.held_mask == 0 || rhythm.cleave_surge(1.0) > 0.0 || rhythm.cleave_surge(-1.0) > 0.0 {
            return (self, attire, rhythm, false);
        }
        let prev = self.dig_progress;
        let side = if self.held_mask & 1 != 0 { 1.0 } else { -1.0 };
        let (next, next_attire, next_rhythm, excavated) = self.act_with_hand(side, attire, rhythm, eye, view_dir, avatar_pos, world);
        (next, next_attire, next_rhythm, excavated || next.dig_progress != prev)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn act_with_hand(
        self, side: f32, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        let state = self.reconcile_attire(attire);
        let (right, left) = attire.held_pair();
        let (active, other, bit) = if side > 0.0 { (right, left, 1u8) } else { (left, right, 2u8) };

        if active.placeable_block() {
            let next_rhythm = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
            let (next_satchel, next_attire, placed) = state.satchel.place_from_hand(side, attire, eye, view_dir, avatar_pos, world);
            return (Self::new(next_satchel, 0, 0, Vec3::ZERO, false), next_attire, next_rhythm, placed);
        }

        let (effort, next_mask, recipient_side, next_rhythm) = if active.spade_blade() {
            let target_side = if other.free_palm() || other.placeable_block() { -side } else { side };
            let swing = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
            (4, bit, target_side, swing)
        } else if right.free_palm() && left.free_palm() {
            let combined = state.held_mask | bit;
            if combined == 3 {
                (if state.held_mask == 3 { 2 } else { 1 }, 3, side, rhythm.trigger_right_strike().trigger_left_strike())
            } else {
                (1, bit, side, if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() })
            }
        } else if active.free_palm() {
            (1, bit, side, if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() })
        } else {
            return (Self::new(state.satchel, 0, 0, Vec3::ZERO, false), attire, Self::trigger_hand_swing(attire, rhythm, side), false);
        };

        let site = Self::locate_dig_site(eye, view_dir, world);
        let progress = state.dig_progress + effort;
        if progress >= EXCAVATION_THRESHOLD {
            if world.excavate_surface(eye, view_dir) {
                let (next_satchel, next_attire) = state.satchel.collect_excavated_block(recipient_side, attire);
                return (Self::new(next_satchel, 0, 0, site, true), next_attire, next_rhythm, true);
            }
            return (Self::new(state.satchel, 0, 0, Vec3::ZERO, false), attire, next_rhythm, false);
        }

        let stage = ((progress + 1) / 2).clamp(1, 3) as u8;
        if world.loosen_surface(eye, view_dir, stage) {
            return (Self::new(state.satchel, progress, next_mask, site, true), attire, next_rhythm, false);
        }
        (Self::new(state.satchel, 0, 0, Vec3::ZERO, false), attire, next_rhythm, false)
    }
}
