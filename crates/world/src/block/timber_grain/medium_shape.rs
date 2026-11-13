use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, TurfMeshTarget};
use super::super::cabin_seat::CabinSeatMesh;
use super::super::cell::GridCell;
use super::super::earth_clods::EarthClods;
use super::super::field_camera::FieldCameraTripod;
use super::super::mineral_seam::MineralSeam;
use super::super::mold::MoldArchetype;
use super::super::pedal_cluster::{FootwellPlinth, PedalClusterMesh};
use super::super::shape::VoxelShape;
use super::super::vehicle_parts::{VehicleBlockMesh, WheelProfile};
use super::super::wedge::WedgeHeading;
use super::BlockMedium;

/// Decorator binding a material medium to a VoxelShape.
pub struct MediumShape<S = Box<dyn VoxelShape>> {
    inner: S,
    medium: BlockMedium,
    archetype: MoldArchetype,
    heading: WedgeHeading,
}

impl<S: VoxelShape> MediumShape<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_archetype(
        inner: S,
        medium: BlockMedium,
        archetype: MoldArchetype,
        heading: WedgeHeading,
    ) -> Self {
        Self { inner, medium, archetype, heading }
    }

    pub fn with_heading(inner: S, medium: BlockMedium, full_cube: bool, heading: WedgeHeading) -> Self {
        let archetype = if full_cube { MoldArchetype::FullCube } else { MoldArchetype::HalfSlab };
        Self::with_archetype(inner, medium, archetype, heading)
    }

    pub fn new(inner: S, medium: BlockMedium, full_cube: bool, cross_grain: bool) -> Self {
        Self::with_heading(inner, medium, full_cube, if cross_grain { WedgeHeading::East } else { WedgeHeading::South })
    }

    const fn cross_grain(&self) -> bool {
        matches!(self.heading, WedgeHeading::East | WedgeHeading::West)
    }

    fn emit_sculpted_turf(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, cap: Vec3, wall: Vec3, target: &mut dyn MeshTarget) {
        let mut turf = TurfMeshTarget::new(target, frame.normal(), cell.corner_points(frame, floor_elevation), cap, wall, false);
        self.inner.emit_shape(cell, floor_elevation, frame, wall, &mut turf);
        turf.seal_underside();
    }
}

impl<S: VoxelShape> VoxelShape for MediumShape<S> {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        if self.medium.mechanical_part() || matches!(self.medium, BlockMedium::CantileverRig | BlockMedium::FieldCamera) {
            GridCell::SPAN
        } else {
            self.inner.elevation_at(local_u, local_v)
        }
    }
    fn shape_color(&self) -> Vec3 { self.inner.shape_color() }
    fn block_medium(&self) -> BlockMedium { self.medium }
    fn block_heading(&self) -> WedgeHeading { self.heading }
    fn wheel_radius(&self) -> f32 { WheelProfile::from_archetype(self.archetype).radius() }
    fn full_cube_envelope(&self) -> bool {
        self.archetype.full_height_flat() && !self.medium.mechanical_part() && !matches!(self.medium, BlockMedium::FieldCamera)
    }
    fn step_camera_turn(&mut self, delta: i32) -> bool {
        if self.medium == BlockMedium::FieldCamera {
            self.archetype = MoldArchetype::from_code((self.archetype.code() as i32 + delta).rem_euclid(8) as u8);
            true
        } else {
            false
        }
    }
    fn camera_tripod_hub(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        (self.medium == BlockMedium::FieldCamera)
            .then(|| FieldCameraTripod::new(self.heading, self.archetype).tripod_hub(cell, floor_elevation, frame))
    }
    fn rotate_quarter(&mut self) { self.heading = self.heading.rotated_ccw(); }

    fn emit_shape(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, color: Vec3, target: &mut dyn MeshTarget) {
        self.emit_terraced_shape(cell, floor_elevation, frame, color, color, true, target);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_driven_part(&self, cell: GridCell, floor_elevation: f32, spin_angle: f32, steer_angle: f32, frame: &SurfaceFrame, top_color: Vec3, wall_color: Vec3, target: &mut dyn MeshTarget) {
        if self.medium == BlockMedium::CabinSeat {
            CabinSeatMesh::new(self.heading).emit(&self.inner, cell, floor_elevation, frame, target);
        } else if self.medium == BlockMedium::PedalCluster {
            PedalClusterMesh::mounted(self.heading).emit(cell, floor_elevation, frame, target);
        } else if self.medium == BlockMedium::FieldCamera {
            FieldCameraTripod::new(self.heading, self.archetype).emit(cell, floor_elevation, frame, target);
        } else if matches!(self.medium, BlockMedium::SteeringWheel | BlockMedium::AircraftEngine | BlockMedium::RocketEngine | BlockMedium::CarWheel | BlockMedium::CrossAxle) {
            VehicleBlockMesh::with_motion(self.medium, self.heading, self.archetype, (spin_angle, steer_angle)).emit(cell, floor_elevation, frame, target);
        } else {
            self.emit_terraced_shape(cell, floor_elevation, frame, top_color, wall_color, true, target);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_terraced_shape(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, top_color: Vec3, _wall_color: Vec3, bare_top: bool, target: &mut dyn MeshTarget) {
        match (self.medium, self.archetype.full_height_flat()) {
            (BlockMedium::TurfEarth, true) => EarthClods::new(cell, floor_elevation).emit_terraced(top_color, bare_top, frame, target),
            (BlockMedium::CrustRock, true) => BlockMedium::emit_crust_rock_cube(cell, floor_elevation, frame, target),
            (med, true) if med.mineral_ore() => MineralSeam::new(med).emit_ore_cube(cell, floor_elevation, frame, target),
            (BlockMedium::RawTimber, true) => BlockMedium::emit_raw_timber_cube(cell, floor_elevation, frame, target),
            (BlockMedium::PlanedPlanks, true) | (BlockMedium::CantileverRig, _) => {
                BlockMedium::emit_planed_planks_cube(cell, floor_elevation, self.cross_grain(), frame, target);
            }
            (BlockMedium::CabinSeat, _) => CabinSeatMesh::new(self.heading).emit(&self.inner, cell, floor_elevation, frame, target),
            (BlockMedium::PedalCluster, _) => PedalClusterMesh::mounted(self.heading).emit(cell, floor_elevation, frame, target),
            (BlockMedium::FieldCamera, _) => FieldCameraTripod::new(self.heading, self.archetype).emit(cell, floor_elevation, frame, target),
            (BlockMedium::SteeringWheel | BlockMedium::AircraftEngine | BlockMedium::RocketEngine | BlockMedium::CarWheel | BlockMedium::CrossAxle, _) => {
                VehicleBlockMesh::with_archetype(self.medium, self.heading, self.archetype).emit(cell, floor_elevation, frame, target);
            }
            (BlockMedium::TurfEarth, false) => {
                EarthClods::new(cell, floor_elevation).emit_shaped(&self.inner, top_color, bare_top, frame, target);
            }
            _ => {
                let (cap, wall) = self.medium.sculpted_tones();
                self.emit_sculpted_turf(cell, floor_elevation, frame, cap, wall, target);
            }
        }
    }
}
