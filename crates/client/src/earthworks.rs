use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use crate::attire::{BeltSatchel, BodyPose, FacetedSolid, LimbFrame, SatchelWardrobe, SightMode, StrideRhythm, TaperedPrism, Wardrobe, YouthAttire};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};

const EXCAVATION_THRESHOLD: u32 = 8;

/// Immutable dual-hand block inventory, satchel inspection state, per-hand block molds, and excavation progress ("Tell, Don't Ask").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalmEarthworks {
    satchel: BeltSatchel, dig_progress: u32, held_mask: u8, dig_site: Vec3, splash_active: bool,
}

impl PalmEarthworks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(satchel: BeltSatchel, dig_progress: u32, held_mask: u8, dig_site: Vec3, splash_active: bool) -> Self {
        Self { satchel, dig_progress, held_mask, dig_site, splash_active }
    }

    /// Secondary constructor initializing empty hands and satchel with forward full-cube molds.
    pub const fn empty() -> Self { Self::new(BeltSatchel::empty(), 0, 0, Vec3::ZERO, false) }
    pub const fn satchel_open(&self) -> bool { self.satchel.open() }

    pub fn mold_next_form(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.map_mold(attire, BlockMold::next_form), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    pub fn turn_mold_clockwise(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.map_mold(attire, BlockMold::turn_clockwise), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    pub fn turn_mold_counter_clockwise(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.map_mold(attire, BlockMold::turn_counter_clockwise), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    /// Dispatches mold, satchel transfer, satchel toggle, and movement-bound satchel navigation triggers using single-trigger bindings.
    pub fn handle_trigger(self, trigger: InputTrigger, bindings: &ActionBindings, attire: YouthAttire) -> (Self, YouthAttire, bool) {
        if !self.satchel.open() && bindings.matches(ControlAction::MoldForm, trigger) {
            return (self.mold_next_form(attire), attire, true);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldRight, trigger) {
            return (self.turn_mold_clockwise(attire), attire, true);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldLeft, trigger) {
            return (self.turn_mold_counter_clockwise(attire), attire, true);
        }
        if bindings.matches(ControlAction::Satchel, trigger)
            || (self.satchel.open() && trigger == InputTrigger::Key(winit::keyboard::KeyCode::Escape))
        {
            let (next_satchel, ok) = self.satchel.reconcile_attire(attire).toggle_open();
            return (Self::new(next_satchel, 0, 0, Vec3::ZERO, false), attire, ok);
        }
        if bindings.matches(ControlAction::SatchelRight, trigger) || bindings.matches(ControlAction::SatchelLeft, trigger) {
            let side = if bindings.matches(ControlAction::SatchelRight, trigger) { 1.0 } else { -1.0 };
            let (next_satchel, next_attire, ok) = self.satchel.transfer_hand(side, attire);
            return (Self::new(next_satchel, 0, 0, Vec3::ZERO, false), next_attire, ok);
        }
        let dx = if bindings.matches(ControlAction::MoveLeft, trigger) { -1 } else if bindings.matches(ControlAction::MoveRight, trigger) { 1 } else { 0 };
        let dy = if bindings.matches(ControlAction::MoveForward, trigger) { -1 } else if bindings.matches(ControlAction::MoveBackward, trigger) { 1 } else { 0 };
        if dx != 0 || dy != 0 {
            let (next_satchel, ok) = self.satchel.navigate(dx, dy);
            return (Self::new(next_satchel, self.dig_progress, self.held_mask, self.dig_site, self.splash_active), attire, ok);
        }
        (self, attire, false)
    }

    pub fn emit_attire(&self, attire: &YouthAttire, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let (r, l) = attire.held_pair();
        let dig_r = r.placeable_block() && (self.held_mask & 1) != 0 && (self.dig_progress > 0 || (self.held_mask & 12) != 0);
        let dig_l = l.placeable_block() && (self.held_mask & 2) != 0 && (self.dig_progress > 0 || (self.held_mask & 12) != 0);
        SatchelWardrobe::new(*attire, self.satchel, dig_r, dig_l).emit(pose, sight, target);
    }

    pub fn reconcile_attire(self, attire: YouthAttire) -> Self {
        Self::new(self.satchel.reconcile_attire(attire), self.dig_progress, self.held_mask, self.dig_site, self.splash_active)
    }

    pub fn trigger_hand_swing(attire: YouthAttire, rhythm: StrideRhythm, side: f32) -> StrideRhythm {
        let (right, left) = attire.held_pair();
        if right.free_palm() && left.free_palm() { return rhythm.trigger_right_strike().trigger_left_strike(); }
        let (active, other) = if side > 0.0 { (right, left) } else { (left, right) };
        if active.gripped() || active.free_palm() { if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() } }
        else if other.free_palm() { if side > 0.0 { rhythm.trigger_left_strike() } else { rhythm.trigger_right_strike() } }
        else { rhythm }
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
            if (-0.02..=0.52).contains(&(ground - sample).dot(up)) { return ground - up * (ground - sample).dot(up).max(0.0); }
        }
        world.constrain_to_surface(eye + dir, 0.0)
    }

    pub fn release_hand(
        self, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool) {
        let state = self.reconcile_attire(attire);
        if state.dig_progress == 0 && (state.held_mask & 12) != 0 {
            let side = if state.held_mask & 4 != 0 { 1.0 } else { -1.0 };
            let (next_satchel, next_attire, placed) = state.satchel.place_from_hand(side, attire, eye, view_dir, avatar_pos, world);
            return (Self::new(next_satchel, 0, 0, Vec3::ZERO, false), next_attire, placed);
        }
        let cleared = if state.dig_progress > 0 { world.loosen_surface(eye, view_dir, 0) } else { false };
        (Self::new(state.satchel, 0, 0, state.dig_site, false), attire, cleared)
    }

    pub fn continue_digging(
        self, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        if (self.held_mask & 3) == 0 { return (self, attire, rhythm, false); }
        let side = if self.held_mask & 1 != 0 { 1.0 } else { -1.0 };
        let (place_bit, dig_bit) = if side > 0.0 { (4u8, 16u8) } else { (8u8, 32u8) };
        if (self.held_mask & place_bit) != 0 {
            if rhythm.cleave_surge(side) > 0.0 && rhythm.cleave_arc(side) < 0.0 { return (self, attire, rhythm, false); }
            let state = Self::new(self.satchel, self.dig_progress, (self.held_mask & !12) | dig_bit, self.dig_site, self.splash_active).reconcile_attire(attire);
            if rhythm.cleave_surge(side) > 0.0 {
                let site = Self::locate_dig_site(eye, view_dir, world);
                let ok = world.loosen_surface(eye, view_dir, 1);
                return (Self::new(state.satchel, u32::from(ok), state.held_mask, if ok { site } else { Vec3::ZERO }, ok), attire, rhythm, ok);
            }
            let (next, next_attire, next_rhythm, excavated) = state.act_with_hand(side, attire, rhythm, eye, view_dir, world);
            return (next, next_attire, next_rhythm, excavated || next.dig_progress > 0);
        }
        if rhythm.cleave_surge(1.0) > 0.0 || rhythm.cleave_surge(-1.0) > 0.0 { return (self, attire, rhythm, false); }
        let prev = self.dig_progress;
        let (next, next_attire, next_rhythm, excavated) = self.act_with_hand(side, attire, rhythm, eye, view_dir, world);
        (next, next_attire, next_rhythm, excavated || next.dig_progress != prev)
    }

    pub fn act_with_hand(
        self, side: f32, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        let state = self.reconcile_attire(attire);
        let (right, left) = attire.held_pair();
        let (active, other, bit) = if side > 0.0 { (right, left, 1u8) } else { (left, right, 2u8) };
        let (place_bit, dig_bit) = (bit << 2, bit << 4);

        if active.placeable_block() && (state.held_mask & bit) == 0 && (state.held_mask & 12) == 0 && state.dig_progress == 0 {
            let next_rhythm = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
            return (Self::new(state.satchel, 0, bit | place_bit, Vec3::ZERO, false), attire, next_rhythm, false);
        }

        let deferred = u32::from((state.held_mask & place_bit) != 0);
        let both_dig = (right.free_palm() || right.placeable_block()) && (left.free_palm() || left.placeable_block());
        let (effort, next_mask, recipient_side, next_rhythm) = if active.spade_blade() {
            let target_side = if other.free_palm() || other.placeable_block() { -side } else { side };
            (4, bit | dig_bit, target_side, if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() })
        } else if both_dig && ((state.held_mask & 3) | bit) == 3 {
            let step = if (state.held_mask & 3) == 3 { 2 } else { 1 + u32::from((state.held_mask & 12) != 0) };
            let target_side = if active.free_palm() || !other.free_palm() { side } else { -side };
            (step, 3 | 16 | 32, target_side, rhythm.trigger_right_strike().trigger_left_strike())
        } else if active.free_palm() || active.placeable_block() {
            (1 + deferred, bit | dig_bit, side, if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() })
        } else {
            return (Self::new(state.satchel, 0, 0, Vec3::ZERO, false), attire, Self::trigger_hand_swing(attire, rhythm, side), false);
        };

        let site = Self::locate_dig_site(eye, view_dir, world);
        let progress = state.dig_progress + effort;
        if progress >= EXCAVATION_THRESHOLD {
            if world.excavate_surface(eye, view_dir) {
                let (next_satchel, next_attire) = state.satchel.collect_excavated_block(recipient_side, attire);
                return (Self::new(next_satchel, 0, next_mask, site, true), next_attire, next_rhythm, true);
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
