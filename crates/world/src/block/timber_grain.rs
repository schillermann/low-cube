use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad, TurfMeshTarget};
use super::cabin_seat::CabinSeatMesh;
use super::cell::GridCell;
use super::earth_clods::EarthClods;
use super::field_camera::FieldCameraTripod;
use super::mold::MoldArchetype;
use super::pedal_cluster::{FootwellPlinth, PedalClusterMesh};
use super::shape::VoxelShape;
use super::vehicle_parts::{VehicleBlockMesh, WheelProfile};
use super::wedge::WedgeHeading;

/// Material medium governing the sculpted 3D surface of a molded block in hand and world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockMedium {
    TurfEarth,
    CrustRock,
    QuarriedStone,
    RawTimber,
    PlanedPlanks,
    Sand,
    CantileverRig,
    SteeringWheel,
    AircraftEngine,
    RocketEngine,
    CabinSeat,
    CarWheel,
    FieldCamera,
    PedalCluster,
    CrossAxle,
}

impl BlockMedium {
    /// Indicates whether this block medium settles downward under planetary gravity when unsupported below.
    pub const fn settles_under_gravity(self) -> bool {
        matches!(self, Self::Sand)
    }

    /// Indicates whether this block medium is a vehicle engine or steering control.
    pub const fn vehicle_engine(self) -> bool {
        matches!(self, Self::SteeringWheel | Self::AircraftEngine | Self::RocketEngine)
    }

    /// Indicates whether this block medium is a sculpted mechanical vehicle component.
    pub const fn mechanical_part(self) -> bool {
        self.vehicle_engine() || matches!(self, Self::CabinSeat | Self::CarWheel | Self::PedalCluster | Self::CrossAxle)
    }

    /// Indicates whether this medium is a portable item picked up in a single click without excavation stages.
    pub const fn instant_pickup(self) -> bool {
        matches!(self, Self::FieldCamera)
    }

    const fn sculpted_tones(self) -> (Vec3, Vec3) {
        match self {
            Self::RawTimber => (Vec3::new(0.85, 0.70, 0.46), Vec3::new(0.32, 0.20, 0.11)),
            Self::PlanedPlanks => (Vec3::new(0.84, 0.67, 0.41), Vec3::new(0.74, 0.56, 0.33)),
            Self::QuarriedStone => (Vec3::new(0.62, 0.64, 0.66), Vec3::new(0.50, 0.51, 0.53)),
            Self::Sand => (Vec3::new(0.88, 0.78, 0.52), Vec3::new(0.78, 0.68, 0.44)),
            _ => (Vec3::new(0.35, 0.31, 0.28), Vec3::new(0.31, 0.27, 0.25)),
        }
    }

    /// Primary and secondary debris particle colors emitted when striking this block medium.
    pub const fn debris_tones(self) -> (Vec3, Vec3) {
        match self {
            Self::TurfEarth => (Vec3::new(0.44, 0.31, 0.19), Vec3::new(0.32, 0.22, 0.13)),
            Self::CrustRock => (Vec3::new(0.35, 0.31, 0.28), Vec3::new(0.25, 0.21, 0.20)),
            Self::QuarriedStone => (Vec3::new(0.62, 0.64, 0.66), Vec3::new(0.46, 0.48, 0.50)),
            Self::RawTimber | Self::PlanedPlanks | Self::CantileverRig => (Vec3::new(0.82, 0.66, 0.42), Vec3::new(0.58, 0.42, 0.24)),
            Self::Sand => (Vec3::new(0.88, 0.78, 0.52), Vec3::new(0.74, 0.64, 0.40)),
            Self::SteeringWheel | Self::FieldCamera => (Vec3::new(0.94, 0.76, 0.24), Vec3::new(0.20, 0.22, 0.27)),
            Self::AircraftEngine => (Vec3::new(0.22, 0.88, 0.98), Vec3::new(0.54, 0.62, 0.72)),
            Self::RocketEngine => (Vec3::new(0.99, 0.32, 0.14), Vec3::new(0.24, 0.22, 0.28)),
            Self::CabinSeat => (Vec3::new(0.72, 0.24, 0.16), Vec3::new(0.36, 0.20, 0.13)),
            Self::CarWheel | Self::CrossAxle => (Vec3::new(0.18, 0.20, 0.22), Vec3::new(0.68, 0.72, 0.78)),
            Self::PedalCluster => (Vec3::new(0.88, 0.22, 0.18), Vec3::new(0.24, 0.84, 0.36)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_box(
        u0: f32, u1: f32, v0: f32, v1: f32, h0: f32, h1: f32,
        frame: &SurfaceFrame, wall_col: Vec3, top_col: Vec3, bottom_cap: bool, target: &mut dyn MeshTarget,
    ) {
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        let [b0, b1, b2, b3] = [
            frame.point_at(u0, v0, h0), frame.point_at(u1, v0, h0),
            frame.point_at(u1, v1, h0), frame.point_at(u0, v1, h0),
        ];
        let [p0, p1, p2, p3] = [
            frame.point_at(u0, v0, h1), frame.point_at(u1, v0, h1),
            frame.point_at(u1, v1, h1), frame.point_at(u0, v1, h1),
        ];
        let top_glow = if crate::MeadowCrown::new(frame.normal(), 0).verdant(top_col) { 0.02 } else { 0.0 };
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, top_col, top_glow));
        if bottom_cap {
            target.push_quad(Quad::new(b0, b3, b2, b1, -n, wall_col));
        }
        target.push_quad(Quad::new(b0, b1, p1, p0, -b, wall_col));
        target.push_quad(Quad::new(p3, p2, b2, b3, b, wall_col));
        target.push_quad(Quad::new(b3, b0, p0, p3, -t, wall_col));
        target.push_quad(Quad::new(b1, b2, p2, p1, t, wall_col));
    }

    /// Emits Variant 1A Timber Block: Bark-ridged timber block with bright sapwood and concentric annual rings.
    fn emit_raw_timber_cube(cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (cell.u_bounds(), cell.v_bounds(), floor_elev + 0.5);
        let (bark_dark, bark_ridge, sapwood, ring_dark, heartwood) = (
            Vec3::new(0.32, 0.20, 0.11), Vec3::new(0.44, 0.28, 0.16),
            Vec3::new(0.85, 0.70, 0.46), Vec3::new(0.64, 0.47, 0.27), Vec3::new(0.48, 0.32, 0.17),
        );
        Self::emit_box(u0, u1, v0, v1, floor_elev, top_elev, frame, bark_dark, sapwood, true, target);
        let (r_h0, r_h1, d, w) = (floor_elev + 0.01, top_elev - 0.01, 0.014, 0.040);
        for step in 0..3 {
            let center_u = u0 + (0.20 + (step as f32) * 0.30) * (u1 - u0);
            let center_v = v0 + (0.20 + (step as f32) * 0.30) * (v1 - v0);
            Self::emit_box(center_u - w, center_u + w, v1, v1 + d, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            Self::emit_box(center_u - w, center_u + w, v0 - d, v0, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            Self::emit_box(u1, u1 + d, center_v - w, center_v + w, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            Self::emit_box(u0 - d, u0, center_v - w, center_v + w, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
        }
        for (inset, h_off, col) in [(0.035, 0.000, sapwood), (0.105, 0.006, ring_dark), (0.165, 0.012, sapwood), (0.212, 0.018, heartwood)] {
            Self::emit_box(u0 + inset, u1 - inset, v0 + inset, v1 - inset, top_elev + h_off, top_elev + h_off + 0.006, frame, col, col, false, target);
        }
    }

    /// Emits a full-cube planetary crust rock block with dark craggy relief and glowing mineral seams.
    fn emit_crust_rock_cube(cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let vault = crate::CrustVault::beneath(64);
        let geom = vault.geometry(frame);
        let ((u0, u1), (v0, v1), y0, y1) = (cell.u_bounds(), cell.v_bounds(), floor_elev, floor_elev + 0.5);
        let layer = (floor_elev * 2.0).round() as i32;
        geom.emit_crust_floor(cell, y1, false, target);
        geom.emit_shaft_ceiling(cell, y0, 1.0, target);
        for (edge, norm) in [
            (((u0, v1), (u1, v1)), frame.bitangent()),
            (((u1, v0), (u0, v0)), -frame.bitangent()),
            (((u1, v0), (u1, v1)), frame.tangent()),
            (((u0, v1), (u0, v0)), -frame.tangent()),
        ] {
            geom.emit_crust_flank(edge, layer, layer - 2, norm, target);
        }
    }
    fn emit_planed_planks_cube(cell: GridCell, floor_elev: f32, cross_grain: bool, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (cell.u_bounds(), cell.v_bounds(), floor_elev + 0.5);
        let groove_core = Vec3::new(0.22, 0.14, 0.08);
        let board_tones = [
            Vec3::new(0.86, 0.69, 0.43), Vec3::new(0.76, 0.58, 0.34),
            Vec3::new(0.82, 0.65, 0.39), Vec3::new(0.73, 0.55, 0.32),
        ];
        Self::emit_box(u0 + 0.016, u1 - 0.016, v0 + 0.016, v1 - 0.016, floor_elev + 0.008, top_elev - 0.002, frame, groove_core, groove_core, true, target);
        let (bh, v_pad, bw, h_pad) = (0.125, 0.009, 0.125, 0.009);
        for (i, &tone) in board_tones.iter().enumerate() {
            let (y0, y1) = (floor_elev + (i as f32) * bh + v_pad, floor_elev + ((i + 1) as f32) * bh - v_pad);
            Self::emit_box(u0, u1, v0, v1, y0, y1, frame, tone, tone, true, target);
        }
        for i in 0..4 {
            let tone = board_tones[(i + 1) % 4];
            if cross_grain {
                let (pv0, pv1) = (v0 + (i as f32) * bw + h_pad, v0 + ((i + 1) as f32) * bw - h_pad);
                Self::emit_box(u0, u1, pv0, pv1, top_elev - 0.002, top_elev + 0.010, frame, tone, tone, false, target);
            } else {
                let (pu0, pu1) = (u0 + (i as f32) * bw + h_pad, u0 + ((i + 1) as f32) * bw - h_pad);
                Self::emit_box(pu0, pu1, v0, v1, top_elev - 0.002, top_elev + 0.010, frame, tone, tone, false, target);
            }
        }
    }
}

/// Decorator binding a material medium to a VoxelShape.
pub struct MediumShape {
    inner: Box<dyn VoxelShape>,
    medium: BlockMedium,
    archetype: MoldArchetype,
    heading: WedgeHeading,
}

impl MediumShape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_archetype(
        inner: Box<dyn VoxelShape>,
        medium: BlockMedium,
        archetype: MoldArchetype,
        heading: WedgeHeading,
    ) -> Self {
        Self { inner, medium, archetype, heading }
    }

    pub fn with_heading(inner: Box<dyn VoxelShape>, medium: BlockMedium, full_cube: bool, heading: WedgeHeading) -> Self {
        let archetype = if full_cube { MoldArchetype::FullCube } else { MoldArchetype::HalfSlab };
        Self::with_archetype(inner, medium, archetype, heading)
    }

    pub fn new(inner: Box<dyn VoxelShape>, medium: BlockMedium, full_cube: bool, cross_grain: bool) -> Self {
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

impl VoxelShape for MediumShape {
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
            CabinSeatMesh::new(self.heading).emit(self.inner.as_ref(), cell, floor_elevation, frame, target);
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
            (BlockMedium::RawTimber, true) => BlockMedium::emit_raw_timber_cube(cell, floor_elevation, frame, target),
            (BlockMedium::PlanedPlanks, true) | (BlockMedium::CantileverRig, _) => {
                BlockMedium::emit_planed_planks_cube(cell, floor_elevation, self.cross_grain(), frame, target);
            }
            (BlockMedium::CabinSeat, _) => CabinSeatMesh::new(self.heading).emit(self.inner.as_ref(), cell, floor_elevation, frame, target),
            (BlockMedium::PedalCluster, _) => PedalClusterMesh::mounted(self.heading).emit(cell, floor_elevation, frame, target),
            (BlockMedium::FieldCamera, _) => FieldCameraTripod::new(self.heading, self.archetype).emit(cell, floor_elevation, frame, target),
            (BlockMedium::SteeringWheel | BlockMedium::AircraftEngine | BlockMedium::RocketEngine | BlockMedium::CarWheel | BlockMedium::CrossAxle, _) => {
                VehicleBlockMesh::with_archetype(self.medium, self.heading, self.archetype).emit(cell, floor_elevation, frame, target);
            }
            (BlockMedium::TurfEarth, false) => {
                EarthClods::new(cell, floor_elevation).emit_shaped(self.inner.as_ref(), top_color, bare_top, frame, target);
            }
            _ => {
                let (cap, wall) = self.medium.sculpted_tones();
                self.emit_sculpted_turf(cell, floor_elevation, frame, cap, wall, target);
            }
        }
    }
}
