use glam::Vec3;
use crate::block::{BlockMedium, MoldArchetype, NozzleAxis, VoxelShape};
use crate::frame::SurfaceFrame;
use super::super::impulse::{CockpitAxes, PilotOrder, RigidImpulse};

const CORE_MODULE_MASS: f32 = 1.00;
const OUTBOARD_POD_MASS: f32 = 0.75;
const DENSE_HULL_MASS: f32 = 0.45;
const LIGHT_FRAME_MASS: f32 = 0.20;

/// Spacecraft mass and nozzle reaction contributor for a single `VehiclePart`.
pub(super) struct PartPropulsion<'a> {
    shape: &'a dyn VoxelShape,
    mass_center: Vec3,
}

impl<'a> PartPropulsion<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(shape: &'a dyn VoxelShape, mass_center: Vec3) -> Self {
        Self { shape, mass_center }
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
        let thrust_dir = axis.thrust_vector(frame).normalize_or_zero();
        let throttle = order.throttle_along(thrust_dir, axes);
        impulse.with_bell_burn(self.mass_center - center_of_mass, thrust_dir, throttle)
    }

    pub(super) fn attitude_burn(
        &self,
        frame: &SurfaceFrame,
        order: &PilotOrder,
        axes: CockpitAxes,
        impulse: RigidImpulse,
    ) -> (RigidImpulse, f32) {
        let Some(axis) = self.nozzle_axis().filter(|a| a.attitude_pod()) else {
            return (impulse, 0.0);
        };
        let thrust_dir = axis.thrust_vector(frame).normalize_or_zero();
        let lift_dir = frame.normal().normalize_or_zero();
        let throttle = order.throttle_along(thrust_dir, axes);
        let lift_throttle = order.throttle_along(lift_dir, axes);
        let retro_throttle = order.throttle_along(-lift_dir, axes);
        let next = impulse
            .with_attitude_jet(thrust_dir, throttle)
            .with_attitude_jet(-lift_dir, retro_throttle)
            .with_attitude_steer(order.attitude_rate(axes));
        (next, lift_throttle)
    }
}
