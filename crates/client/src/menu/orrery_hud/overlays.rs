use glam::{Mat4, Vec3};
use world::MeshTarget;
use crate::drive::{BellySnapshot, NavigationCourse, OpticTelemetry, OrrerySnapshot};
use crate::menu::viewfinder::{CameraFeed, ViewfinderHud};
use crate::menu::widget::{ScreenBounds, ScreenPoint};
use super::OrreryConsole;
use super::standoff::BoresightTape;
use super::spine::RangeScale;

/// Focused spacecraft hull-camera HUD layer contract supporting decorator composition.
pub trait OpticOverlay {
    fn emit(&self, target: &mut dyn MeshTarget);
}

/// Null-Object `OpticOverlay` emitted when no spacecraft hull-optic telemetry is active.
#[derive(Clone, Copy, Debug)]
pub struct SilentOverlay;

impl OpticOverlay for SilentOverlay {
    fn emit(&self, _target: &mut dyn MeshTarget) {}
}

/// `OpticOverlay` decorator projecting `[ + ]` surface target-lock brackets onto the active hull camera.
#[derive(Clone, Copy, Debug)]
pub struct TargetLockOverlay<T> {
    origin: T,
    target_world: Vec3,
    view_proj: Mat4,
    inv_aspect: f32,
}

impl<T> TargetLockOverlay<T> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: T, target_world: Vec3, view_proj: Mat4, inv_aspect: f32) -> Self {
        Self { origin, target_world, view_proj, inv_aspect }
    }
}

impl<T: OpticOverlay> OpticOverlay for TargetLockOverlay<T> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        let clip = self.view_proj * self.target_world.extend(1.0);
        if clip.w <= 0.15 {
            return;
        }
        let (nx, ny) = (clip.x / clip.w, clip.y / clip.w);
        if nx.abs() > 1.06 || ny.abs() > 1.06 {
            return;
        }
        let ux = self.inv_aspect;
        let (rx, ry) = (0.056 * ux, 0.056);
        let (ax, ay) = (0.021 * ux, 0.021);
        let (tx, ty) = (0.0034 * ux, 0.0034);
        let bracket_col = Vec3::new(0.28, 0.96, 0.78);
        let cross_col = Vec3::new(0.46, 0.96, 1.00);
        for sx in [-1.0_f32, 1.0_f32] {
            for sy in [-1.0_f32, 1.0_f32] {
                let (cx, cy) = (nx + sx * rx, ny + sy * ry);
                let (hx0, hx1) = if sx < 0.0 { (cx, cx + ax) } else { (cx - ax, cx) };
                let (vy0, vy1) = if sy < 0.0 { (cy, cy + ay) } else { (cy - ay, cy) };
                ScreenBounds::new(hx0, cy - ty, hx1, cy + ty).emit_fill(bracket_col, target);
                ScreenBounds::new(cx - tx, vy0, cx + tx, vy1).emit_fill(bracket_col, target);
            }
        }
        ScreenBounds::new(nx - 0.013 * ux, ny - 0.0022, nx + 0.013 * ux, ny + 0.0022)
            .emit_fill(cross_col, target);
        ScreenBounds::new(nx - 0.0022 * ux, ny - 0.013, nx + 0.0022 * ux, ny + 0.013)
            .emit_fill(cross_col, target);
    }
}

/// `OpticOverlay` decorator uniting an inner HUD overlay with a decorated `RangeScale` spine.
#[derive(Clone, Copy, Debug)]
pub struct RangeTapeOverlay<O, S> {
    origin: O,
    scale: S,
}

impl<O, S> RangeTapeOverlay<O, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: O, scale: S) -> Self {
        Self { origin, scale }
    }
}

impl<O: OpticOverlay, S: RangeScale> OpticOverlay for RangeTapeOverlay<O, S> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        self.scale.emit_scale(target);
    }
}

/// `OpticOverlay` decorator layering the multi-camera intercom & preview strip over an inner HUD overlay.
#[derive(Clone, Copy, Debug)]
pub struct ChannelStripOverlay<O> {
    origin: O,
    feed: CameraFeed,
    aspect: f32,
}

impl<O> ChannelStripOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: O, feed: CameraFeed, aspect: f32) -> Self {
        Self { origin, feed, aspect }
    }
}

impl<O: OpticOverlay> OpticOverlay for ChannelStripOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        ViewfinderHud::new(self.feed, self.aspect).emit(target);
    }
}

/// Nose-Camera HUD composing `OrreryConsole`, `TargetLockOverlay`, and `RangeTapeOverlay` decorators.
#[derive(Clone, Copy, Debug)]
pub struct NoseOrreryHud {
    snapshot: OrrerySnapshot,
    view_proj: Mat4,
    reticle: ScreenPoint,
    inv_aspect: f32,
}

impl NoseOrreryHud {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        snapshot: OrrerySnapshot,
        view_proj: Mat4,
        reticle: ScreenPoint,
        inv_aspect: f32,
    ) -> Self {
        Self { snapshot, view_proj, reticle, inv_aspect }
    }
}

impl OpticOverlay for NoseOrreryHud {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let console = OrreryConsole::new(self.snapshot, ux);
        self.snapshot.transfer(|_, ship_pos, (_, velocity), course| match course {
            NavigationCourse::Uncharted => console.emit(target),
            NavigationCourse::Charted(ref arc) => {
                arc.sample_range(ship_pos, velocity, |tgt_world, dist_m, rate_mps| {
                    let locked = TargetLockOverlay::new(console, tgt_world, self.view_proj, ux);
                    let tape = BoresightTape::nose(self.reticle, dist_m, rate_mps, ux);
                    RangeTapeOverlay::new(locked, tape).emit(target);
                });
            }
        });
    }
}

/// Polymorphic spacecraft hull-optic HUD dispatching Nose and Belly decorator chains without `Option`.
#[derive(Clone, Copy, Debug)]
pub struct ShipOpticHud {
    telemetry: OpticTelemetry,
    view_proj: Mat4,
    reticle: ScreenPoint,
    inv_aspect: f32,
}

impl ShipOpticHud {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_parts(
        telemetry: OpticTelemetry,
        view_proj: Mat4,
        reticle: ScreenPoint,
        inv_aspect: f32,
    ) -> Self {
        Self { telemetry, view_proj, reticle, inv_aspect }
    }

    pub fn new(telemetry: OpticTelemetry, view_proj: Mat4, reticle: ScreenPoint, aspect: f32) -> Self {
        Self::from_parts(
            telemetry,
            view_proj,
            reticle,
            (1.0 / aspect.max(0.5)).clamp(0.25, 1.5),
        )
    }

    fn emit_belly(&self, belly: BellySnapshot, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        belly.transfer(|clearance, course, ship_pos| {
            clearance.transfer(|in_well, altitude, range_rate| {
                let tape = BoresightTape::belly(self.reticle, altitude, range_rate, ux);
                match course {
                    NavigationCourse::Uncharted if in_well => {
                        RangeTapeOverlay::new(SilentOverlay, tape).emit(target);
                    }
                    NavigationCourse::Uncharted => {
                        SilentOverlay.emit(target);
                    }
                    NavigationCourse::Charted(ref arc) => {
                        arc.sample_range(ship_pos, Vec3::ZERO, |tgt_world, _, _| {
                            let locked = TargetLockOverlay::new(SilentOverlay, tgt_world, self.view_proj, ux);
                            if in_well {
                                RangeTapeOverlay::new(locked, tape).emit(target);
                            } else {
                                locked.emit(target);
                            }
                        });
                    }
                }
            });
        });
    }
}

impl OpticOverlay for ShipOpticHud {
    fn emit(&self, target: &mut dyn MeshTarget) {
        match self.telemetry {
            OpticTelemetry::Silent => SilentOverlay.emit(target),
            OpticTelemetry::Nose(orrery) => {
                NoseOrreryHud::new(orrery, self.view_proj, self.reticle, self.inv_aspect).emit(target);
            }
            OpticTelemetry::Belly(belly) => self.emit_belly(belly, target),
        }
    }
}
