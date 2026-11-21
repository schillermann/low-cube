use glam::Vec3;
use crate::block::{BlockMold, GridCell, VoxelShape, WedgeHeading};
use crate::frame::SurfaceFrame;
use super::super::craft_entity::{CraftBlueprint, CraftIdentity};
use super::super::molded_structure::VoxelCells;
use super::super::sight_ray::SurfaceSightRay;
use super::super::vehicle_assembly::{VehicleAssembly, VehicleAxes, VehiclePart};

/// Collaborator extracting blueprint parts and surface-cleared launchpad origin from a crane `CraftCluster`.
pub(super) struct PadCommission<'a> {
    grid: &'a VoxelCells,
}

impl<'a> PadCommission<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(grid: &'a VoxelCells) -> Self {
        Self { grid }
    }

    pub(super) fn extract_parts(&self, anchor: (i32, i32), frame: &SurfaceFrame) -> Vec<VehiclePart> {
        let floor_min = self.min_floor();
        let flip_v = frame.tangent().cross(frame.bitangent()).dot(frame.normal()) > 0.0;
        let v_sign = if flip_v { -1 } else { 1 };
        let mut parts = Vec::new();
        self.grid.visit_cells(|(cu, cv), list| {
            for b in list {
                let floor_off = (b.floor() - floor_min).max(0.0);
                let mut shape = BlockMold::shape_from_stamp(b.shape_stamp());
                if flip_v && matches!(shape.block_heading(), WedgeHeading::North | WedgeHeading::South) {
                    shape.rotate_quarter();
                    shape.rotate_quarter();
                }
                parts.push(VehiclePart::new((cu - anchor.0, (cv - anchor.1) * v_sign), floor_off, shape));
            }
        });
        parts.sort_by_key(VehiclePart::canonical_key);
        parts
    }

    pub(super) fn assemble_vehicle(
        &self,
        identity: CraftIdentity,
        seat: (i32, i32),
        ground_fn: impl Fn(i32, i32) -> f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> VehicleAssembly {
        let flip_v = frame.tangent().cross(frame.bitangent()).dot(frame.normal()) > 0.0;
        let v_sign = if flip_v { -1 } else { 1 };
        let axes = VehicleAxes::new(frame.tangent(), frame.bitangent() * (v_sign as f32), frame.normal());
        let canonical_frame = SurfaceFrame::new(frame.tangent(), frame.bitangent() * (v_sign as f32), frame.normal(), 0.0);
        let parts = self.extract_parts(seat, frame);
        let spacefaring = parts.iter().any(VehiclePart::flight_console);
        let pad_top = self.cleared_pad_top(seat, spacefaring, ground_fn);
        let heading = parts
            .iter()
            .find_map(|p| p.console_heading(&canonical_frame))
            .or_else(|| parts.iter().find_map(|p| p.steering_heading(&canonical_frame)))
            .unwrap_or_else(|| ray.align_heading(frame));
        let (cx, cy) = GridCell::new(seat.0, seat.1).center();
        VehicleAssembly::with_identity(identity, frame.point_at(cx, cy, pad_top), axes, heading, parts)
    }

    pub(super) fn draft_blueprint(&self, anchor: (i32, i32), frame: &SurfaceFrame) -> Option<CraftBlueprint> {
        let parts = self.extract_parts(anchor, frame);
        (!parts.is_empty()).then(|| CraftBlueprint::commission(Vec3::ZERO, parts))
    }

    fn min_floor(&self) -> f32 {
        let mut floor_min = f32::MAX;
        self.grid.visit_cells(|_, list| {
            for b in list {
                floor_min = floor_min.min(b.floor());
            }
        });
        floor_min
    }

    fn cleared_pad_top(&self, seat: (i32, i32), spacefaring: bool, ground_fn: impl Fn(i32, i32) -> f32) -> f32 {
        let mut pad_top = ground_fn(seat.0, seat.1);
        if !spacefaring {
            return pad_top;
        }
        let floor_min = self.min_floor();
        self.grid.visit_cells(|(cu, cv), list| {
            let ground = ground_fn(cu, cv);
            for b in list {
                let floor_off = (b.floor() - floor_min).max(0.0);
                pad_top = pad_top.max(ground - floor_off);
            }
        });
        pad_top
    }
}
