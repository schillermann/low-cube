use glam::Vec3;
use crate::block::{
    BlockMedium, BlockMold, CuboidShape, DiagonalSlope, GentleRidge, GentleValley, GentleWedge,
    GridCell, MoldArchetype, PyramidPeak, RampWedge, RidgeCorner, SensorPodAxis, StairCorner,
    StairFlight, ValleyCorner, VoxelShape, WedgeHeading,
};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::hull_facet::{FacetSink, HullFacet};

/// Closed 3D structural or fairing volume contributing to a spacecraft's unified outer hull contour.
#[derive(Clone, Copy, Debug)]
pub(in crate::relief) enum HullSolid {
    Molded {
        cell: GridCell,
        floor: f32,
        arch: MoldArchetype,
        wedge: WedgeHeading,
    },
    PodCone {
        mount_hub: Vec3,
        axis: Vec3,
    },
}

impl HullSolid {
    pub(in crate::relief) fn from_stamp(cell: GridCell, floor: f32, stamp: [u8; 3]) -> Option<Self> {
        let arch = MoldArchetype::from_code(stamp[0]);
        let wedge = WedgeHeading::from_code(stamp[1]);
        let medium = BlockMedium::from_code(stamp[2]);
        match medium {
            BlockMedium::RocketNozzle | BlockMedium::FusionTorch | BlockMedium::CarWheel => None,
            BlockMedium::RocketCamera => {
                let unit_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
                let (uc, vc) = cell.center();
                let (mount_hub, axis) = match SensorPodAxis::from_mold(arch, wedge) {
                    SensorPodAxis::NoseCone => (Vec3::new(uc, floor, vc), Vec3::Y),
                    SensorPodAxis::BellyPod => (Vec3::new(uc, floor + 0.50, vc), Vec3::NEG_Y),
                    SensorPodAxis::SideFairing(heading) => {
                        let out = (-heading.forward_vector(&unit_frame)).normalize_or_zero();
                        (Vec3::new(uc, floor + 0.25, vc) - out * 0.25, out)
                    }
                };
                Some(Self::PodCone { mount_hub, axis })
            }
            BlockMedium::CabinSeat => {
                let seat_arch = if arch == MoldArchetype::HalfSlab {
                    MoldArchetype::HalfSlab
                } else {
                    MoldArchetype::FullCube
                };
                Some(Self::Molded { cell, floor, arch: seat_arch, wedge })
            }
            BlockMedium::FlightConsole
            | BlockMedium::SteeringWheel
            | BlockMedium::AircraftEngine
            | BlockMedium::PedalCluster
            | BlockMedium::FieldCamera
            | BlockMedium::VehicleBlueprint
            | BlockMedium::CantileverRig => {
                Some(Self::Molded { cell, floor, arch: MoldArchetype::FullCube, wedge })
            }
            _ => Some(Self::Molded { cell, floor, arch, wedge }),
        }
    }

    fn with_voxel_shape<R>(
        arch: MoldArchetype,
        wedge: WedgeHeading,
        f: impl FnOnce(&dyn VoxelShape) -> R,
    ) -> R {
        let corner = BlockMold::corner_for(wedge);
        match arch {
            MoldArchetype::FullCube => f(&CuboidShape::full_cube()),
            MoldArchetype::HalfSlab => f(&CuboidShape::half_slab()),
            MoldArchetype::RampWedge => f(&RampWedge::new(wedge)),
            MoldArchetype::SteppedStair => f(&StairFlight::new(wedge)),
            MoldArchetype::PyramidApex => f(&PyramidPeak::slope_aligned()),
            MoldArchetype::ConvexRidge => f(&RidgeCorner::new(corner)),
            MoldArchetype::ConcaveValley => f(&ValleyCorner::new(corner)),
            MoldArchetype::DiagonalRamp => f(&DiagonalSlope::new(corner)),
            MoldArchetype::GentleLowerWedge => f(&GentleWedge::lower(wedge)),
            MoldArchetype::GentleUpperWedge => f(&GentleWedge::upper(wedge)),
            MoldArchetype::GentleLowerRidge => f(&GentleRidge::lower(corner)),
            MoldArchetype::GentleUpperRidge => f(&GentleRidge::upper(corner)),
            MoldArchetype::GentleLowerValley => f(&GentleValley::lower(corner)),
            MoldArchetype::GentleUpperValley => f(&GentleValley::upper(corner)),
            MoldArchetype::ConvexStairCorner => f(&StairCorner::convex(corner)),
            MoldArchetype::ConcaveStairCorner => f(&StairCorner::concave(corner)),
        }
    }

    pub(super) fn contains(&self, p: Vec3) -> bool {
        match *self {
            Self::Molded { cell, floor, arch, wedge } => {
                let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
                if p.x <= u0 + 1e-5
                    || p.x >= u1 - 1e-5
                    || p.z <= v0 + 1e-5
                    || p.z >= v1 - 1e-5
                    || p.y <= floor + 1e-5
                    || p.y >= floor + 0.5 - 1e-5
                {
                    return false;
                }
                let (lu, lv) = ((p.x - u0) * 2.0, (p.z - v0) * 2.0);
                let stepped = matches!(
                    arch,
                    MoldArchetype::SteppedStair
                        | MoldArchetype::ConvexStairCorner
                        | MoldArchetype::ConcaveStairCorner
                );
                if stepped
                    && ((lu - 0.5).abs() <= 2e-5 || (lv - 0.5).abs() <= 2e-5)
                    && p.y >= floor + 0.25 - 2e-5
                {
                    return false;
                }
                let h = Self::with_voxel_shape(arch, wedge, |s| s.elevation_at(lu, lv));
                h > 1e-5 && p.y < floor + h - 1e-5
            }
            Self::PodCone { mount_hub, axis } => {
                let rel = p - mount_hub;
                let z = rel.dot(axis);
                if z <= 1e-5 || z >= 0.25 - 1e-5 {
                    return false;
                }
                let perp = (rel - axis * z).abs();
                perp.max_element() < (0.25 - z) - 1e-5
            }
        }
    }

    pub(super) fn emit_facets(
        &self,
        facets: &mut Vec<HullFacet>,
        edges: &mut Vec<(Vec3, Vec3)>,
    ) {
        let unit_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let mut sink = FacetSink::new(facets, edges, false);
        match *self {
            Self::Molded { cell, floor, arch, wedge } => {
                Self::with_voxel_shape(arch, wedge, |s| {
                    s.emit_shape(cell, floor, &unit_frame, Vec3::ONE, &mut sink);
                });
                sink.seal_underside(cell.corner_points(&unit_frame, floor));
            }
            Self::PodCone { mount_hub, axis } => {
                let ref_vec = if axis.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
                let side_a = axis.cross(ref_vec).normalize_or_zero() * 0.25;
                let side_b = axis.cross(side_a).normalize_or_zero() * 0.25;
                let b = [
                    mount_hub - side_a - side_b,
                    mount_hub + side_a - side_b,
                    mount_hub + side_a + side_b,
                    mount_hub - side_a + side_b,
                ];
                let apex = mount_hub + axis * 0.25;
                sink.push_quad(Quad::new(b[0], b[1], b[2], b[3], -axis, Vec3::ONE));
                for i in 0..4 {
                    let (p0, p1) = (b[i], b[(i + 1) % 4]);
                    let raw = (p1 - p0).cross(apex - p0).normalize_or_zero();
                    let n = if raw.dot(axis) >= 0.0 { raw } else { -raw };
                    sink.push_quad(Quad::new(p0, p1, apex, apex, n, Vec3::ONE));
                }
            }
        }
    }
}
