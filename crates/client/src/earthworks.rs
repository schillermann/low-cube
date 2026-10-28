use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld, SurfaceFind};
use crate::attire::{
    ArtisanTome, BeltSatchel, BodyPose, FacetedSolid, HeldRelic, LimbFrame, SatchelWardrobe,
    SightMode, StrideRhythm, TaperedPrism, Wardrobe, YouthAttire,
};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};

const EXCAVATION_THRESHOLD: u32 = 8;

/// Encapsulates dual-hand button press, deferred block placement, and active digging states without exposing bitmasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DigGrip {
    mask: u8,
}

impl DigGrip {
    pub const fn idle() -> Self { Self { mask: 0 } }
    const fn hand_bit(side: f32) -> u8 { if side > 0.0 { 1 } else { 2 } }
    pub const fn any_held(self) -> bool { (self.mask & 3) != 0 }
    pub const fn both_held(self) -> bool { (self.mask & 3) == 3 }
    pub const fn held_on(self, side: f32) -> bool { (self.mask & Self::hand_bit(side)) != 0 }
    pub const fn active_side(self) -> f32 { if (self.mask & 1) != 0 { 1.0 } else { -1.0 } }
    pub const fn pending_place_any(self) -> bool { (self.mask & 12) != 0 }
    pub const fn pending_place_side(self) -> f32 { if (self.mask & 4) != 0 { 1.0 } else { -1.0 } }
    pub const fn pending_place_on(self, side: f32) -> bool { (self.mask & (Self::hand_bit(side) << 2)) != 0 }
    pub const fn combined_both_held(self, side: f32) -> bool { ((self.mask & 3) | Self::hand_bit(side)) == 3 }

    pub const fn with_pending_place(side: f32) -> Self {
        let b = Self::hand_bit(side);
        Self { mask: b | (b << 2) }
    }

    pub const fn promoted_to_dig(self, side: f32) -> Self {
        Self { mask: (self.mask & !12) | (Self::hand_bit(side) << 4) }
    }

    pub const fn with_dig_on(side: f32, weightless_place: bool) -> Self {
        let b = Self::hand_bit(side);
        let place = if weightless_place { b << 2 } else { 0 };
        Self { mask: b | (b << 4) | place }
    }

    pub const fn with_both_digging() -> Self {
        Self { mask: 3 | 16 | 32 }
    }

    pub const fn without_pending_place(self) -> Self {
        Self { mask: self.mask & !12 }
    }

    pub const fn bare_hand_digging(self, r_block: bool, l_block: bool, progress: u32) -> (bool, bool) {
        let both = self.both_held();
        let dig_r = r_block && (self.mask & 1) != 0 && (progress > 0 || (self.mask & 28) != 0 || (both && l_block));
        let dig_l = l_block && (self.mask & 2) != 0 && (progress > 0 || (self.mask & 12) != 0 || (both && r_block));
        (dig_r, dig_l)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DigOperation { dig_progress: u32, grip: DigGrip, dig_site: Vec3, splash_active: bool }

/// Immutable dual-hand block inventory, satchel inspection state, crafting tome, per-hand block molds, and excavation progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalmEarthworks {
    satchel: BeltSatchel,
    tome: ArtisanTome,
    dig: DigOperation,
}

impl PalmEarthworks {
    pub const fn with_tome(
        satchel: BeltSatchel, tome: ArtisanTome, dig_progress: u32, grip: DigGrip, dig_site: Vec3, splash_active: bool,
    ) -> Self {
        Self { satchel, tome, dig: DigOperation { dig_progress, grip, dig_site, splash_active } }
    }

    pub const fn new(
        satchel: BeltSatchel, dig_progress: u32, grip: DigGrip, dig_site: Vec3, splash_active: bool,
    ) -> Self {
        Self::with_tome(satchel, ArtisanTome::closed(), dig_progress, grip, dig_site, splash_active)
    }

    pub const fn empty() -> Self { Self::new(BeltSatchel::empty(), 0, DigGrip::idle(), Vec3::ZERO, false) }
    pub const fn satchel_open(&self) -> bool { self.satchel.open() || self.tome.open() }
    pub const fn tome_open(&self) -> bool { self.tome.open() }
    fn with_satchel(self, satchel: BeltSatchel) -> Self { Self { satchel, ..self } }
    pub fn mold_next_form(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::next_form)) }
    pub fn mold_prev_form(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::prev_form)) }
    pub fn turn_mold_clockwise(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::turn_clockwise)) }
    pub fn reconcile_attire(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.reconcile_attire(attire)) }

    pub fn handle_trigger(
        self, trigger: InputTrigger, bindings: &ActionBindings, attire: YouthAttire,
    ) -> (Self, YouthAttire, bool) {
        if bindings.matches(ControlAction::CraftingBook, trigger)
            || (self.tome.open() && trigger == InputTrigger::Key(winit::keyboard::KeyCode::Escape))
        {
            let next_satchel = self.satchel.reconcile_attire(attire).with_open(false);
            return (Self::with_tome(next_satchel, self.tome.toggle(), 0, DigGrip::idle(), Vec3::ZERO, false), attire, true);
        }
        if self.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = self.tome.dispatch_trigger(trigger, bindings, self.satchel, attire);
            return (Self::with_tome(next_satchel, next_tome, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, ok);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldForm, trigger) {
            return (self.turn_mold_clockwise(attire), attire, true);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldRight, trigger) {
            return (self.mold_next_form(attire), attire, true);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldLeft, trigger) {
            return (self.mold_prev_form(attire), attire, true);
        }
        if bindings.matches(ControlAction::Satchel, trigger)
            || (self.satchel.open() && trigger == InputTrigger::Key(winit::keyboard::KeyCode::Escape))
        {
            let (next_satchel, ok) = self.satchel.reconcile_attire(attire).toggle_open();
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), attire, ok);
        }
        if bindings.matches(ControlAction::SatchelRight, trigger) || bindings.matches(ControlAction::SatchelLeft, trigger) {
            let side = if bindings.matches(ControlAction::SatchelRight, trigger) { 1.0 } else { -1.0 };
            let (next_satchel, next_attire, ok) = self.satchel.transfer_hand(side, attire);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, ok);
        }
        let dx = if bindings.matches(ControlAction::MoveLeft, trigger) { -1 } else if bindings.matches(ControlAction::MoveRight, trigger) { 1 } else { 0 };
        let dy = if bindings.matches(ControlAction::MoveForward, trigger) { -1 } else if bindings.matches(ControlAction::MoveBackward, trigger) { 1 } else { 0 };
        if dx != 0 || dy != 0 {
            let (next_satchel, ok) = self.satchel.navigate(dx, dy);
            return (self.with_satchel(next_satchel), attire, ok);
        }
        (self, attire, false)
    }

    pub fn emit_attire(&self, attire: &YouthAttire, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let (r, l) = attire.held_pair();
        let (dig_r, dig_l) = self.dig.grip.bare_hand_digging(r.placeable_block(), l.placeable_block(), self.dig.dig_progress);
        SatchelWardrobe::new(*attire, self.satchel, dig_r, dig_l).emit(pose, sight, target);
        self.tome.emit_book(&self.satchel, *attire, pose, target);
    }

    pub fn trigger_hand_swing(attire: YouthAttire, rhythm: StrideRhythm, side: f32) -> StrideRhythm {
        let (right, left) = attire.held_pair();
        if right.free_palm() && left.free_palm() {
            return rhythm.trigger_right_strike().trigger_left_strike();
        }
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
        if !self.dig.splash_active || t <= 0.01 { return; }
        let right = view_dir.cross(surface_up).normalize_or_zero();
        let frame = LimbFrame::new(self.dig.dig_site, right, surface_up, surface_up.cross(right).normalize_or_zero());
        let spray = [
            (-0.85, 0.42, 0.30, 0.24, 0.019, 0.00), (0.78, 0.55, 0.34, 0.28, 0.022, 0.04),
            (-0.35, -0.88, 0.26, 0.20, 0.017, 0.08), (0.46, -0.82, 0.32, 0.25, 0.020, 0.02),
            (-0.92, -0.22, 0.28, 0.22, 0.016, 0.06), (0.90, -0.18, 0.35, 0.26, 0.018, 0.10),
        ];
        for (i, &(dx, dz, reach, apex, rad, delay)) in spray.iter().enumerate() {
            let tk = ((t - delay) / (1.0 - delay)).clamp(0.0, 1.0);
            if tk > 0.0 && tk < 1.0 {
                let (dist, lift) = (0.02 + reach * tk, 4.0 * apex * tk * (1.0 - tk));
                let col = if i % 2 == 0 { Vec3::new(0.44, 0.31, 0.19) } else { Vec3::new(0.32, 0.22, 0.13) };
                let p = Vec3::new(dx * dist, lift, dz * dist);
                TaperedPrism::new(4, p, p + Vec3::Y * (rad * 1.5), rad, rad, rad * 0.75, rad * 0.75, col).emit(&frame, true, target);
            }
        }
    }

    fn locate_dig_site(eye: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> Vec3 {
        let (dir, up) = (view_dir.normalize_or_zero(), world.surface_up_at(eye));
        if world.weightless_at(eye) || (world.constrain_to_surface(eye, 0.0) - eye).dot(up) > 0.5 {
            return world.confine_beneath_crust(eye + dir * 1.1);
        }
        for step in 1..=85 {
            let sample = eye + dir * ((step as f32) * 0.04);
            let ground = world.constrain_to_surface(sample, 0.0);
            if (ground - sample).dot(up) >= -0.02 {
                return ground - up * (ground - sample).dot(up).max(0.0);
            }
        }
        world.constrain_to_surface(eye + dir, 0.0)
    }

    pub fn release_hand(
        self, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool) {
        let (state, up) = (self.reconcile_attire(attire), world.surface_up_at(eye));
        let d = state.dig;
        let overhead = world.weightless_at(eye)
            || (world.constrain_to_surface(eye, 0.0) - eye).dot(up) > 0.5
            || view_dir.normalize_or_zero().dot(up) > 0.7;
        if d.grip.pending_place_any() && (d.dig_progress == 0 || (overhead && d.dig_progress <= 4)) {
            if d.dig_progress > 0 { world.loosen_surface(eye, view_dir, 0); }
            let side = d.grip.pending_place_side();
            let (next_satchel, next_attire, placed) = state.satchel.place_from_hand(side, attire, eye, view_dir, avatar_pos, world);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, placed);
        }
        let cleared = if d.dig_progress > 0 { world.loosen_surface(eye, view_dir, 0) } else { false };
        (Self::new(state.satchel, 0, DigGrip::idle(), d.dig_site, false), attire, cleared)
    }

    pub fn continue_digging(
        self, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        let d = self.dig;
        if !d.grip.any_held() { return (self, attire, rhythm, false); }
        let side = d.grip.active_side();
        if d.grip.pending_place_on(side) && d.dig_progress == 0 {
            if rhythm.cleave_surge(side) > 0.0 && rhythm.cleave_arc(side) < 0.0 {
                return (self, attire, rhythm, false);
            }
            let state = Self::new(self.satchel, d.dig_progress, d.grip.promoted_to_dig(side), d.dig_site, d.splash_active).reconcile_attire(attire);
            if rhythm.cleave_surge(side) > 0.0 {
                let (site, ok) = (Self::locate_dig_site(eye, view_dir, world), world.loosen_surface(eye, view_dir, 1));
                return (Self::new(state.satchel, u32::from(ok), state.dig.grip, if ok { site } else { Vec3::ZERO }, ok), attire, rhythm, ok);
            }
            let (next, next_attire, next_rhythm, excavated) = state.act_with_hand(side, attire, rhythm, eye, view_dir, world);
            return (next, next_attire, next_rhythm, excavated || next.dig.dig_progress > 0);
        }
        if rhythm.cleave_surge(1.0) > 0.0 || rhythm.cleave_surge(-1.0) > 0.0 {
            return (self, attire, rhythm, false);
        }
        let prev = d.dig_progress;
        let (next, next_attire, next_rhythm, excavated) = self.act_with_hand(side, attire, rhythm, eye, view_dir, world);
        (next, next_attire, next_rhythm, excavated || next.dig.dig_progress != prev)
    }

    pub fn act_with_hand(
        self, side: f32, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        if self.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = self.tome.craft_selected(side, self.satchel, attire);
            let next_rhythm = if ok { Self::trigger_hand_swing(next_attire, rhythm, side) } else { rhythm };
            return (Self::with_tome(next_satchel, next_tome, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, next_rhythm, false);
        }
        let (state, find) = (self.reconcile_attire(attire), world.forage_surface(eye, view_dir));
        let d = state.dig;
        if find.gathered() {
            let gathered = if matches!(find, SurfaceFind::DeadwoodBough) { HeldRelic::DeadwoodBough } else { HeldRelic::FlintStone };
            let (next_satchel, next_attire) = state.satchel.collect_foraged(side, gathered, attire);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, Self::trigger_hand_swing(next_attire, rhythm, side), true);
        }
        let (right, left) = attire.held_pair();
        let (active, other) = if side > 0.0 { (right, left) } else { (left, right) };
        let up = world.surface_up_at(eye);
        let weightless = world.weightless_at(eye) || (world.constrain_to_surface(eye, 0.0) - eye).dot(up) > 0.5 || view_dir.normalize_or_zero().dot(up) > 0.7;
        if active.placeable_block() && !weightless && !d.grip.held_on(side) && !d.grip.pending_place_any() && d.dig_progress == 0 {
            let next_rhythm = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
            return (Self::new(state.satchel, 0, DigGrip::with_pending_place(side), Vec3::ZERO, false), attire, next_rhythm, false);
        }
        let deferred = u32::from(d.grip.pending_place_on(side));
        let w_place = weightless && active.placeable_block() && d.dig_progress == 0;
        let both_dig = (right.free_palm() || right.placeable_block()) && (left.free_palm() || left.placeable_block());
        let (effort, next_grip, recipient_side, next_rhythm) = if active.spade_blade() {
            let target_side = if other.free_palm() || other.placeable_block() { -side } else { side };
            (4, DigGrip::with_dig_on(side, false), target_side, if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() })
        } else if both_dig && d.grip.combined_both_held(side) {
            let step = if d.grip.both_held() { 2 } else { 1 + u32::from(d.grip.pending_place_any()) };
            let target_side = if active.free_palm() || !other.free_palm() { side } else { -side };
            let grip = if weightless { DigGrip::with_dig_on(side, w_place) } else { DigGrip::with_both_digging() };
            (if weightless { 4 } else { step }, grip, target_side, rhythm.trigger_right_strike().trigger_left_strike())
        } else if active.free_palm() || active.placeable_block() {
            let outcome = if other.axe_blade() { world.strike_timber(eye, view_dir) } else { world::CleaveOutcome::Missed };
            if outcome.struck() {
                let (ns, na) = if outcome.felled() { state.satchel.collect_felled_timber(side, attire) } else { (state.satchel, attire) };
                return (Self::new(ns, 0, DigGrip::with_dig_on(side, false), Vec3::ZERO, false), na, Self::trigger_hand_swing(attire, rhythm, -side), true);
            }
            (if weightless { 4 } else { 1 + deferred }, DigGrip::with_dig_on(side, w_place), side, if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() })
        } else {
            let outcome = if active.axe_blade() { world.strike_timber(eye, view_dir) } else { world::CleaveOutcome::Missed };
            let (ns, na) = if outcome.felled() { state.satchel.collect_felled_timber(-side, attire) } else { (state.satchel, attire) };
            let grip = if active.axe_blade() { DigGrip::with_dig_on(side, false) } else { DigGrip::idle() };
            return (Self::new(ns, 0, grip, Vec3::ZERO, false), na, Self::trigger_hand_swing(attire, rhythm, side), outcome.struck());
        };
        let (site, progress) = (Self::locate_dig_site(eye, view_dir, world), d.dig_progress + effort);
        if progress >= EXCAVATION_THRESHOLD {
            let harvested = HeldRelic::from_medium(world.surface_medium(eye, view_dir));
            if world.excavate_surface(eye, view_dir) {
                let (next_satchel, next_attire) = state.satchel.collect_excavated_block(recipient_side, harvested, attire);
                return (Self::new(next_satchel, 0, next_grip.without_pending_place(), site, true), next_attire, next_rhythm, true);
            }
            return (Self::new(state.satchel, 0, next_grip.without_pending_place(), Vec3::ZERO, false), attire, next_rhythm, false);
        }
        if world.loosen_surface(eye, view_dir, progress.div_ceil(2).clamp(1, 3) as u8) {
            return (Self::new(state.satchel, progress, next_grip, site, true), attire, next_rhythm, false);
        }
        let fallback_grip = if w_place { DigGrip::with_pending_place(side) } else { next_grip.without_pending_place() };
        (Self::new(state.satchel, 0, fallback_grip, Vec3::ZERO, false), attire, next_rhythm, false)
    }
}
