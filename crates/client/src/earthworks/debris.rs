use glam::Vec3;
use world::{BlockMedium, MeshTarget, PlanetaryWorld};
use crate::attire::{FacetedSolid, HeldRelic, LimbFrame, StrideRhythm, TaperedPrism, YouthAttire};
use crate::cardiac::{AcousticWave, CadencedWave};
use crate::grip::{DigGrip, DigStroke};

const SUBTERRANEAN_ROOF_OFFSET: f32 = 0.5;
const OVERHEAD_AIM_DOT: f32 = 0.7;

/// Encapsulated excavation debris spray at a world site with material tones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DebrisSplash {
    site: Vec3,
    medium: BlockMedium,
    active: bool,
}

impl DebrisSplash {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(site: Vec3, medium: BlockMedium, active: bool) -> Self {
        Self { site, medium, active }
    }

    pub const fn halted(self) -> Self {
        Self::new(self.site, self.medium, false)
    }

    pub fn sub_eye(eye: Vec3, up: Vec3, world: &dyn PlanetaryWorld) -> bool {
        (world.constrain_to_surface(eye, 0.0) - eye).dot(up) > SUBTERRANEAN_ROOF_OFFSET
    }

    pub fn overhead_aim(eye: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> bool {
        let up = world.surface_up_at(eye);
        Self::sub_eye(eye, up, world) || view_dir.normalize_or_zero().dot(up) > OVERHEAD_AIM_DOT
    }

    pub fn locate_site(eye: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> Vec3 {
        let (dir, up) = (view_dir.normalize_or_zero(), world.surface_up_at(eye));
        if Self::sub_eye(eye, up, world) {
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

    pub fn resolve_site(self, eye: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> Vec3 {
        if self.active {
            self.site
        } else {
            Self::locate_site(eye, view_dir, world)
        }
    }

    pub fn emit(self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        let t = rhythm.strike_progress();
        if !self.active || t <= 0.01 {
            return;
        }
        let right = view_dir.cross(surface_up).normalize_or_zero();
        let frame = LimbFrame::new(self.site, right, surface_up, surface_up.cross(right).normalize_or_zero());
        let (col_a, col_b) = self.medium.debris_tones();
        let spray = [
            (-0.85, 0.42, 0.30, 0.24, 0.019, 0.00),
            (0.78, 0.55, 0.34, 0.28, 0.022, 0.04),
            (-0.35, -0.88, 0.26, 0.20, 0.017, 0.08),
            (0.46, -0.82, 0.32, 0.25, 0.020, 0.02),
            (-0.92, -0.22, 0.28, 0.22, 0.016, 0.06),
            (0.90, -0.18, 0.35, 0.26, 0.018, 0.10),
        ];
        for (i, &(dx, dz, reach, apex, rad, delay)) in spray.iter().enumerate() {
            let tk = ((t - delay) / (1.0 - delay)).clamp(0.0, 1.0);
            if tk > 0.0 && tk < 1.0 {
                let (dist, lift) = (0.02 + reach * tk, 4.0 * apex * tk * (1.0 - tk));
                let col = if i % 2 == 0 { col_a } else { col_b };
                let p = Vec3::new(dx * dist, lift, dz * dist);
                TaperedPrism::new(4, p, p + Vec3::Y * (rad * 1.5), rad, rad, rad * 0.75, rad * 0.75, col).emit(&frame, true, target);
            }
        }
    }
}

/// Polymorphic digging instrument resonance triggered at a progressive stroke cadence.
pub trait StrokeResonance {
    fn resonate(&self, cadence: u32);
}

impl StrokeResonance for HeldRelic {
    fn resonate(&self, cadence: u32) {
        if self.spade_blade() {
            CadencedWave::spade(cadence).thump();
        } else {
            CadencedWave::palm(cadence).thump();
        }
    }
}

/// Encapsulated excavation progress, stroke cadence, dual-hand grip, and debris splash state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DigOperation {
    dig_progress: u32,
    cadence: u32,
    grip: DigGrip,
    splash: DebrisSplash,
}

impl DigOperation {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(dig_progress: u32, cadence: u32, grip: DigGrip, splash: DebrisSplash) -> Self {
        Self { dig_progress, cadence, grip, splash }
    }

    pub const fn idle() -> Self {
        Self::new(0, 0, DigGrip::idle(), DebrisSplash::new(Vec3::ZERO, BlockMedium::TurfEarth, false))
    }

    pub const fn halted(self) -> Self {
        Self::new(0, self.cadence, DigGrip::idle(), self.splash.halted())
    }

    pub const fn with_grip(self, grip: DigGrip) -> Self {
        Self::new(0, self.cadence, grip, DebrisSplash::new(Vec3::ZERO, BlockMedium::TurfEarth, false))
    }

    pub const fn promoted(self, grip: DigGrip) -> Self {
        Self::new(self.dig_progress, self.cadence, grip, self.splash)
    }

    pub fn advanced_stroke(
        self, progress: u32, grip: DigGrip, site: Vec3, medium: BlockMedium, instrument: &dyn StrokeResonance,
    ) -> Self {
        instrument.resonate(self.cadence);
        Self::new(progress, self.cadence.wrapping_add(1), grip, DebrisSplash::new(site, medium, true))
    }

    pub fn loosen_stroke(
        self, progress: u32, grip: DigGrip, site: Vec3, medium: BlockMedium, instrument: &dyn StrokeResonance,
    ) -> Self {
        if progress > self.dig_progress {
            self.advanced_stroke(progress, grip, site, medium, instrument)
        } else {
            Self::new(progress, self.cadence, grip, DebrisSplash::new(site, medium, true))
        }
    }

    pub const fn untouched(self) -> bool {
        self.dig_progress == 0
    }

    pub const fn scarred(self) -> bool {
        self.dig_progress > 0
    }

    pub const fn progressed_from(self, prior: Self) -> bool {
        self.dig_progress != prior.dig_progress
    }

    pub const fn step_progress(self, stroke: DigStroke) -> u32 {
        stroke.advance_progress(self.dig_progress)
    }

    pub const fn grip(self) -> DigGrip {
        self.grip
    }

    pub fn bare_hand_digging(self, attire: &YouthAttire) -> (bool, bool) {
        let (r, l) = attire.held_pair();
        self.grip.bare_hand_digging(r.placeable_block(), l.placeable_block(), self.dig_progress)
    }

    pub const fn ready_to_perch(self) -> bool {
        self.grip.pending_perch_any() && self.dig_progress == 0
    }

    pub const fn ready_to_place(self, overhead: bool) -> bool {
        self.grip.pending_place_any() && (self.dig_progress == 0 || (overhead && self.dig_progress <= 4))
    }

    pub fn clear_cracks(self, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld) -> bool {
        if self.dig_progress > 0 {
            world.loosen_surface(eye, view_dir, 0).is_some()
        } else {
            false
        }
    }

    pub fn resolve_site(self, eye: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> Vec3 {
        self.splash.resolve_site(eye, view_dir, world)
    }

    pub fn emit_debris(self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.splash.emit(view_dir, surface_up, rhythm, target);
    }
}
