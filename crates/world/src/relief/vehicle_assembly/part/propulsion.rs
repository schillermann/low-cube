use glam::Vec3;
use crate::block::{
    AttitudeNozzleMesh, BlockMedium, FlightConsoleMesh, GridCell, MoldArchetype, NozzleAxis,
    RcsManifold, VoxelShape,
};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::super::impulse::{CockpitAxes, PilotOrder, RigidImpulse};

const CORE_MODULE_MASS: f32 = 1.00;
const OUTBOARD_POD_MASS: f32 = 0.75;
const DENSE_HULL_MASS: f32 = 0.45;
const LIGHT_FRAME_MASS: f32 = 0.20;

/// Spacecraft mass and nozzle reaction contributor for a single `VehiclePart`.
pub(super) struct PartPropulsion<'a> {
    shape: &'a dyn VoxelShape,
    cell: GridCell,
    floor_elev: f32,
}

impl<'a> PartPropulsion<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(shape: &'a dyn VoxelShape, cell: GridCell, floor_elev: f32) -> Self {
        Self { shape, cell, floor_elev }
    }

    fn nozzle_axis(&self) -> Option<NozzleAxis> {
        (self.shape.block_medium() == BlockMedium::RocketNozzle).then(|| {
            NozzleAxis::from_mold(
                MoldArchetype::from_code(self.shape.shape_stamp()[0]),
                self.shape.block_heading(),
            )
        })
    }

    pub(super) fn part_mass(&self) -> f32 {
        match self.shape.block_medium() {
            BlockMedium::CabinSeat | BlockMedium::FlightConsole => CORE_MODULE_MASS,
            BlockMedium::RocketCamera => OUTBOARD_POD_MASS,
            BlockMedium::RocketNozzle => match self.nozzle_axis() {
                Some(axis) if axis.attitude_pod() => OUTBOARD_POD_MASS,
                _ => CORE_MODULE_MASS,
            },
            BlockMedium::QuarriedStone
            | BlockMedium::CrustRock
            | BlockMedium::CopperOre
            | BlockMedium::IronOre
            | BlockMedium::GoldOre => DENSE_HULL_MASS,
            _ => LIGHT_FRAME_MASS,
        }
    }

    pub(super) fn bell_burn(
        &self,
        frame: &SurfaceFrame,
        center_of_mass: Vec3,
        order: &PilotOrder,
        axes: CockpitAxes,
        impulse: RigidImpulse,
    ) -> RigidImpulse {
        let Some(axis) = self.nozzle_axis().filter(|a| !a.attitude_pod()) else {
            return impulse;
        };
        let (uc, vc) = self.cell.center();
        let mass_center = frame.point_at(uc, vc, self.floor_elev + 0.25);
        let thrust_dir = axis.thrust_vector(frame).normalize_or_zero();
        let throttle = order.bell_throttle(thrust_dir, axes);
        impulse.with_bell_burn(mass_center - center_of_mass, thrust_dir, throttle)
    }

    pub(super) fn attitude_throttles(
        &self,
        frame: &SurfaceFrame,
        center_of_mass: Vec3,
        linear_demand: Vec3,
        torque_demand: Vec3,
    ) -> Option<[f32; 5]> {
        let NozzleAxis::Attitude(heading) = self.nozzle_axis()? else {
            return None;
        };
        Some(RcsManifold::new(heading).resolve_throttles(
            self.cell,
            self.floor_elev,
            frame,
            center_of_mass,
            linear_demand,
            torque_demand,
        ))
    }

    pub(super) fn attitude_burn(
        &self,
        frame: &SurfaceFrame,
        center_of_mass: Vec3,
        linear_demand: Vec3,
        torque_demand: Vec3,
        impulse: RigidImpulse,
    ) -> RigidImpulse {
        let Some(NozzleAxis::Attitude(heading)) = self.nozzle_axis() else {
            return impulse;
        };
        let manifold = RcsManifold::new(heading);
        let horns = manifold.horns(self.cell, self.floor_elev, frame);
        let throttles = manifold.resolve_throttles(
            self.cell,
            self.floor_elev,
            frame,
            center_of_mass,
            linear_demand,
            torque_demand,
        );
        horns.iter().zip(throttles.iter()).fold(impulse, |imp, (horn, &throttle)| {
            imp.with_rcs_horn(horn.origin() - center_of_mass, horn.thrust_dir(), throttle)
        })
    }

    pub(super) fn emit_space_part(
        &self,
        frame: &SurfaceFrame,
        center_of_mass: Vec3,
        order: &PilotOrder,
        linear_demand: Vec3,
        torque_demand: Vec3,
        out: &mut dyn MeshTarget,
    ) -> bool {
        if self.shape.block_medium() == BlockMedium::FlightConsole {
            FlightConsoleMesh::with_guidance(
                self.shape.block_heading(),
                (order.vertical_burn(), order.console_stick()),
                order.guidance_engaged(),
            )
            .emit(self.cell, self.floor_elev, frame, out);
            return true;
        }
        if let Some(throttles) = self.attitude_throttles(frame, center_of_mass, linear_demand, torque_demand) {
            AttitudeNozzleMesh::with_throttles(self.shape.block_heading(), throttles)
                .emit(self.cell, self.floor_elev, frame, out);
            return true;
        }
        false
    }
}
