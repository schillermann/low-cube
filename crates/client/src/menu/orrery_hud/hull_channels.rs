use glam::{Mat4, Vec3};
use world::MeshTarget;
use crate::drive::{BellySnapshot, NavigationCourse, OpticTelemetry, OrrerySnapshot};
use crate::menu::widget::ScreenPoint;
use super::burn_magnet::BurnMagnetOverlay;
use super::compass_ring::CompassRingOverlay;
use super::course_alignment::CourseAlignmentOverlay;
use super::lateral_slip::LateralSlipOverlay;
use super::overlays::{OpticOverlay, RangeTapeOverlay, SilentOverlay, TargetLockOverlay};
use super::prograde_drift::ProgradeDriftOverlay;
use super::sight_stage::HudSightStage;
use super::standoff::BoresightTape;
use super::touchdown_ghost::TouchdownGhostOverlay;
use super::wire_ship::WireframeShipOverlay;

/// Nose-Camera HUD composing `CompassRingOverlay`, `ProgradeDriftOverlay`, `BurnMagnetOverlay`, `CourseAlignmentOverlay`, `TargetLockOverlay`, and `RangeTapeOverlay`.
#[derive(Clone, Copy, Debug)]
pub struct NoseFlightHud {
    snapshot: OrrerySnapshot,
    view_proj: Mat4,
    reticle: ScreenPoint,
    inv_aspect: f32,
}

impl NoseFlightHud {
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

impl OpticOverlay for NoseFlightHud {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let stage = HudSightStage::new(self.view_proj, self.reticle, ux);
        self.snapshot.transfer_flight(|atlas, kinematics, course| {
            kinematics.transfer(|ship_pos, _, velocity| {
                let compass = CompassRingOverlay::new(SilentOverlay, stage, atlas, ship_pos);
                let drift = ProgradeDriftOverlay::new(compass, stage, kinematics);
                match course {
                    NavigationCourse::Uncharted => drift.emit(target),
                    NavigationCourse::Charted(arc) => {
                        let magnet = BurnMagnetOverlay::new(drift, stage, kinematics, arc);
                        let aligned = CourseAlignmentOverlay::new(magnet, stage, kinematics, arc);
                        arc.sample_range(ship_pos, velocity, |tgt_world, dist_m, rate_mps| {
                            let locked = TargetLockOverlay::new(aligned, tgt_world, self.view_proj, ux);
                            let tape = BoresightTape::nose(self.reticle, dist_m, rate_mps, ux);
                            RangeTapeOverlay::new(locked, tape).emit(target);
                        });
                    }
                }
            });
        });
    }
}

/// Belly-Camera HUD composing `TouchdownGhostOverlay`, `LateralSlipOverlay`, `TargetLockOverlay`, and `RangeTapeOverlay`.
#[derive(Clone, Copy, Debug)]
pub struct BellyFlightHud {
    snapshot: BellySnapshot,
    view_proj: Mat4,
    reticle: ScreenPoint,
    inv_aspect: f32,
}

impl BellyFlightHud {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        snapshot: BellySnapshot,
        view_proj: Mat4,
        reticle: ScreenPoint,
        inv_aspect: f32,
    ) -> Self {
        Self { snapshot, view_proj, reticle, inv_aspect }
    }
}

impl OpticOverlay for BellyFlightHud {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let vp = self.view_proj;
        let stage = HudSightStage::new(vp, self.reticle, ux);
        self.snapshot.transfer(|clearance, trace, course, ship_pos| {
            let ghost = TouchdownGhostOverlay::new(SilentOverlay, stage, trace);
            let slip = LateralSlipOverlay::new(ghost, stage, trace);
            clearance.transfer(|in_well, altitude, range_rate| {
                let tape = BoresightTape::belly(self.reticle, altitude, range_rate, ux);
                match course {
                    NavigationCourse::Uncharted if in_well => {
                        RangeTapeOverlay::new(slip, tape).emit(target);
                    }
                    NavigationCourse::Uncharted => slip.emit(target),
                    NavigationCourse::Charted(ref arc) => {
                        arc.sample_range(ship_pos, Vec3::ZERO, |tgt_world, _, _| {
                            let locked = TargetLockOverlay::new(slip, tgt_world, vp, ux);
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
}

impl OpticOverlay for ShipOpticHud {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let vp = self.view_proj;
        match self.telemetry {
            OpticTelemetry::Silent => SilentOverlay.emit(target),
            OpticTelemetry::Nose(nose, wireframe) => {
                let hud = NoseFlightHud::new(nose, vp, self.reticle, ux);
                WireframeShipOverlay::new(hud, wireframe, vp, ux).emit(target);
            }
            OpticTelemetry::Belly(belly, wireframe) => {
                let hud = BellyFlightHud::new(belly, vp, self.reticle, ux);
                WireframeShipOverlay::new(hud, wireframe, vp, ux).emit(target);
            }
            OpticTelemetry::Flank(wireframe) => {
                WireframeShipOverlay::new(SilentOverlay, wireframe, vp, ux).emit(target);
            }
        }
    }
}
