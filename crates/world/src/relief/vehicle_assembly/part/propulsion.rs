use glam::Vec3;
use crate::block::{
    AttitudeNozzleMesh, BlockMedium, FlightConsoleMesh, FusionTorchMesh, GridCell, MoldArchetype,
    NozzleAxis, RcsManifold, SensorPodAxis, VoxelShape,
};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use crate::relief::craft_entity::{
    CellRole, ExhaustJet, JetTone, WireframeCell, MAX_EXHAUST_JETS, MAX_WIRE_CELLS,
};
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
            BlockMedium::CabinSeat | BlockMedium::FlightConsole | BlockMedium::FusionTorch => CORE_MODULE_MASS,
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

    pub(super) fn fusion_burn(
        &self,
        frame: &SurfaceFrame,
        center_of_mass: Vec3,
        order: &PilotOrder,
        axes: CockpitAxes,
        impulse: RigidImpulse,
    ) -> RigidImpulse {
        if self.shape.block_medium() != BlockMedium::FusionTorch {
            return impulse;
        }
        let (uc, vc) = self.cell.center();
        let mass_center = frame.point_at(uc, vc, self.floor_elev + 0.25);
        let thrust_dir = frame.normal().normalize_or_zero();
        let throttle = order.bell_throttle(thrust_dir, axes);
        impulse.with_fusion_burn(mass_center - center_of_mass, thrust_dir, throttle)
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
        if self.shape.block_medium() == BlockMedium::FusionTorch {
            let glow = order.vertical_burn().max(0.0).clamp(0.0, 1.2);
            FusionTorchMesh::new(glow).emit(self.cell, self.floor_elev, frame, out);
            return true;
        }
        if let Some(throttles) = self.attitude_throttles(frame, center_of_mass, linear_demand, torque_demand) {
            AttitudeNozzleMesh::with_throttles(self.shape.block_heading(), throttles)
                .emit(self.cell, self.floor_elev, frame, out);
            return true;
        }
        false
    }

    pub(in crate::relief) fn collect_wireframe(
        &self,
        frame: &SurfaceFrame,
        axes: CockpitAxes,
        hull_metrics: (Vec3, Vec3, f32),
        command: (&PilotOrder, Vec3, Vec3),
        sinks: (
            &mut ([WireframeCell; MAX_WIRE_CELLS], usize),
            &mut ([ExhaustJet; MAX_EXHAUST_JETS], usize),
        ),
    ) {
        let (com, hull_mid, inv_span) = hull_metrics;
        let (order, lin_demand, torque_demand) = command;
        let (cells, jets) = sinks;
        let (uc, vc) = self.cell.center();
        let center = frame.point_at(uc, vc, self.floor_elev + 0.25);
        let local_center = axes.to_local(center) - hull_mid;

        let mut push_cell = |role: CellRole| {
            if cells.1 < MAX_WIRE_CELLS {
                cells.0[cells.1] = WireframeCell::from_local(local_center, inv_span, role);
                cells.1 += 1;
            }
        };
        let mut push_jet = |mouth_w: Vec3, dir_w: Vec3, tone: JetTone, thr: f32| {
            if thr > 0.02 && jets.1 < MAX_EXHAUST_JETS {
                let lm = axes.to_local(mouth_w) - hull_mid;
                let ld = axes.to_local(dir_w);
                jets.0[jets.1] = ExhaustJet::active(lm, inv_span, ld, tone, thr);
                jets.1 += 1;
            }
        };

        let med = self.shape.block_medium();
        if med == BlockMedium::FusionTorch {
            push_cell(CellRole::FusionBell);
            let up = frame.normal().normalize_or_zero();
            let thr = order.bell_throttle(up, axes);
            push_jet(center - up * 0.25, -up, JetTone::FusionBlue, thr);
            return;
        }
        if let Some(axis) = self.nozzle_axis() {
            match axis {
                NozzleAxis::Downward => {
                    push_cell(CellRole::MainBell);
                    let thrust = axis.thrust_vector(frame).normalize_or_zero();
                    let thr = order.bell_throttle(thrust, axes);
                    push_jet(center - thrust * 0.25, -thrust, JetTone::BellAmber, thr);
                }
                NozzleAxis::Attitude(heading) => {
                    let manifold = RcsManifold::new(heading);
                    let horns = manifold.horns(self.cell, self.floor_elev, frame);
                    let throttles = manifold.resolve_throttles(
                        self.cell,
                        self.floor_elev,
                        frame,
                        com,
                        lin_demand,
                        torque_demand,
                    );
                    let any_lit = throttles.iter().any(|&t| t > 0.02);
                    push_cell(if any_lit { CellRole::ActivePod } else { CellRole::AttitudePod });
                    for (horn, &thr) in horns.iter().zip(throttles.iter()) {
                        push_jet(horn.origin(), horn.exhaust_dir(), JetTone::VernierCyan, thr);
                    }
                }
            }
            return;
        }
        if med == BlockMedium::RocketCamera {
            let pod = SensorPodAxis::from_mold(
                MoldArchetype::from_code(self.shape.shape_stamp()[0]),
                self.shape.block_heading(),
            );
            if matches!(pod, SensorPodAxis::NoseCone) {
                push_cell(CellRole::NoseApex);
                return;
            }
        }
        push_cell(CellRole::HullCube);
    }
}
