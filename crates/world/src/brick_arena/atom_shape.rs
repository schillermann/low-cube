use glam::Vec3;
use crate::{CrateredMeshTarget, MeshTarget};
use crate::block::{
    BlockMedium, CuboidShape, DiagonalSlope, GentleRidge, GentleValley, GentleWedge, GridCell,
    MediumShape, MoldArchetype, PyramidPeak, RampWedge, RidgeCorner, StairCorner, StairFlight,
    ValleyCorner, VoxelShape, LOAM_CROWN,
};
use crate::frame::SurfaceFrame;
use crate::relief::NaturalCrown;
use super::atom::{AtomKind, RawAtom};
use super::niche::{FlankMeshTarget, LoosenedPartTarget};

const EYE_SPAN_SENTINEL: f32 = 1.0;
const UPRIGHT_STATURE_CLEARANCE: f32 = 1.80;
const HEADROOM_TOLERANCE: f32 = 0.03;
const FOOT_SUPPORT_TOLERANCE: f32 = 0.08;
const RAY_INTERCEPT_MARGIN: f32 = 0.02;
const LOAM_WALL: Vec3 = Vec3::new(0.28, 0.19, 0.12);

/// Zero-allocation stack evaluator binding a `RawAtom` to its sculpted `VoxelShape` (`2` fields).
#[derive(Clone, Copy)]
pub struct AtomSurface {
    atom: RawAtom,
    floor_elev: f32,
}

impl AtomSurface {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(atom: RawAtom, floor_elev: f32) -> Self {
        Self { atom, floor_elev }
    }

    pub fn with_shape<R>(self, f: impl FnOnce(&dyn VoxelShape) -> R) -> R {
        struct RefShape<'a>(&'a dyn VoxelShape);
        impl VoxelShape for RefShape<'_> {
            fn elevation_at(&self, u: f32, v: f32) -> f32 { self.0.elevation_at(u, v) }
            fn shape_color(&self) -> Vec3 { self.0.shape_color() }
            fn full_cube_envelope(&self) -> bool { self.0.full_cube_envelope() }
            fn emit_shape(&self, c: GridCell, b: f32, fr: &SurfaceFrame, col: Vec3, t: &mut dyn MeshTarget) {
                self.0.emit_shape(c, b, fr, col, t);
            }
            fn emit_terraced_shape(&self, c: GridCell, b: f32, fr: &SurfaceFrame, tc: Vec3, wc: Vec3, bt: bool, t: &mut dyn MeshTarget) {
                self.0.emit_terraced_shape(c, b, fr, tc, wc, bt, t);
            }
        }
        let medium = self.atom.medium();
        match self.atom.kind() {
            AtomKind::Molded { archetype, heading, corner } => {
                let invoke = |raw: &dyn VoxelShape| {
                    let shaped = MediumShape::with_archetype(RefShape(raw), medium, archetype, heading);
                    f(&shaped)
                };
                match archetype {
                    MoldArchetype::FullCube => invoke(&CuboidShape::full_cube()),
                    MoldArchetype::HalfSlab => invoke(&CuboidShape::half_slab()),
                    MoldArchetype::RampWedge => invoke(&RampWedge::new(heading)),
                    MoldArchetype::SteppedStair => invoke(&StairFlight::new(heading)),
                    MoldArchetype::PyramidApex => invoke(&PyramidPeak::slope_aligned()),
                    MoldArchetype::ConvexRidge => invoke(&RidgeCorner::new(corner)),
                    MoldArchetype::ConcaveValley => invoke(&ValleyCorner::new(corner)),
                    MoldArchetype::DiagonalRamp => invoke(&DiagonalSlope::new(corner)),
                    MoldArchetype::GentleLowerWedge => invoke(&GentleWedge::lower(heading)),
                    MoldArchetype::GentleUpperWedge => invoke(&GentleWedge::upper(heading)),
                    MoldArchetype::GentleLowerRidge => invoke(&GentleRidge::lower(corner)),
                    MoldArchetype::GentleUpperRidge => invoke(&GentleRidge::upper(corner)),
                    MoldArchetype::GentleLowerValley => invoke(&GentleValley::lower(corner)),
                    MoldArchetype::GentleUpperValley => invoke(&GentleValley::upper(corner)),
                    MoldArchetype::ConvexStairCorner => invoke(&StairCorner::convex(corner)),
                    MoldArchetype::ConcaveStairCorner => invoke(&StairCorner::concave(corner)),
                }
            }
            AtomKind::Crown(crown) => match crown {
                NaturalCrown::Ramp(_, h) => f(&RampWedge::new(h)),
                NaturalCrown::Ridge(_, c) => f(&RidgeCorner::new(c)),
                NaturalCrown::Valley(_, c) => f(&ValleyCorner::new(c)),
                NaturalCrown::Flat => f(&CuboidShape::full_cube()),
            },
        }
    }

    pub fn elevation_at(self, local_u: f32, local_v: f32) -> f32 {
        self.with_shape(|s| s.elevation_at(local_u, local_v))
    }

    pub fn crown_elevation(self) -> f32 {
        self.floor_elev + self.elevation_at(0.5, 0.5)
    }

    pub fn full_cube_envelope(self) -> bool {
        self.with_shape(|s| s.full_cube_envelope())
    }

    pub fn smothers_turf(self) -> bool {
        self.with_shape(|s| s.smothers_turf())
    }

    pub fn wheel_ground_lift(self) -> f32 {
        self.with_shape(|s| s.wheel_tread_drop().unwrap_or(0.0))
    }

    pub fn camera_tripod_hub(self, cell: GridCell, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.with_shape(|s| s.camera_tripod_hub(cell, self.floor_elev, frame))
    }

    pub fn constrain_support(self, local_u: f32, local_v: f32, support_h: f32, foot_elev: f32, body_span: f32) -> f32 {
        let dh = self.elevation_at(local_u, local_v);
        let req_clearance = if (body_span - EYE_SPAN_SENTINEL).abs() < 1e-3 {
            UPRIGHT_STATURE_CLEARANCE
        } else {
            body_span
        };
        if dh <= 0.0
            || (body_span > 1e-4
                && self.floor_elev >= support_h + req_clearance - HEADROOM_TOLERANCE
                && foot_elev + FOOT_SUPPORT_TOLERANCE <= self.floor_elev)
        {
            support_h
        } else {
            support_h.max(self.floor_elev + dh)
        }
    }

    pub fn intercepts(self, local_u: f32, local_v: f32, ray_elev: f32) -> bool {
        let dh = self.elevation_at(local_u, local_v);
        dh > 0.0
            && ray_elev >= self.floor_elev - RAY_INTERCEPT_MARGIN
            && ray_elev <= self.floor_elev + dh + RAY_INTERCEPT_MARGIN
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_terraced(
        self,
        cell: GridCell,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        covered_above: bool,
        flank_only: bool,
        stage: u8,
        target: &mut dyn MeshTarget,
    ) {
        let bare_top = covered_above || !self.atom.verdant_turf();
        if stage > 0 && self.atom.medium().mechanical_part() {
            let (uc, vc) = cell.center();
            let pivot = frame.point_at(uc, vc, self.floor_elev + GridCell::SPAN * 0.5);
            let mut loosened = LoosenedPartTarget::new(target, pivot, stage);
            self.with_shape(|s| s.emit_terraced_shape(cell, self.floor_elev, frame, top_color, wall_color, bare_top, &mut loosened));
            return;
        }
        let mut cratered = CrateredMeshTarget::new(target, stage);
        if stage > 0 && self.atom.fertile_soil() && self.full_cube_envelope() {
            let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
            let cap = if bare_top { LOAM_CROWN } else { top_color };
            BlockMedium::emit_box(u0, u1, v0, v1, self.floor_elev, self.floor_elev + GridCell::SPAN, frame, LOAM_WALL, cap, true, &mut cratered);
        } else if flank_only && stage == 0 {
            let mut flank = FlankMeshTarget::new(&mut cratered, frame.normal());
            self.with_shape(|s| s.emit_terraced_shape(cell, self.floor_elev, frame, top_color, wall_color, bare_top, &mut flank));
        } else {
            self.with_shape(|s| s.emit_terraced_shape(cell, self.floor_elev, frame, top_color, wall_color, bare_top, &mut cratered));
        }
    }
}
