use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::super::part::VehiclePart;
use super::{CockpitAxes, PilotOrder, RigidImpulse};

/// Focused 1-method contract for composable stack-allocated spacecraft propulsion decorators.
pub trait ImpulseStage {
    fn resolve(&self) -> RigidImpulse;
}

/// Origin `ImpulseStage` summing linear dry mass and rotational moment of inertia around `center_of_mass`.
pub struct HullBallast<'a> {
    parts: &'a [VehiclePart],
    frame: &'a SurfaceFrame,
    trim: (f32, Vec3),
}

impl<'a> HullBallast<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(parts: &'a [VehiclePart], frame: &'a SurfaceFrame, trim: (f32, Vec3)) -> Self {
        Self { parts, frame, trim }
    }
}

impl ImpulseStage for HullBallast<'_> {
    fn resolve(&self) -> RigidImpulse {
        let (lift, com) = self.trim;
        let (mass, inertia) = self.parts.iter().fold((0.0f32, Vec3::ZERO), |(m_acc, i_acc), part| {
            let m = part.part_mass();
            let rel = part.world_mass_center(lift, self.frame) - com;
            let rx = rel.dot(self.frame.tangent());
            let ry = rel.dot(self.frame.normal());
            let rz = rel.dot(self.frame.bitangent());
            let diag = Vec3::new(ry * ry + rz * rz + 0.08, rx * rx + rz * rz + 0.08, rx * rx + ry * ry + 0.08);
            (m_acc + m, i_acc + diag * m)
        });
        RigidImpulse::with_ballast(mass, inertia, *self.frame)
    }
}

/// `ImpulseStage` decorator folding reaction-wheel gyro authority when `FlightConsole` guidance is engaged.
pub struct ConsoleGyroStage<'a, S> {
    origin: S,
    parts: &'a [VehiclePart],
    command: (PilotOrder, CockpitAxes),
}

impl<'a, S> ConsoleGyroStage<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, parts: &'a [VehiclePart], command: (PilotOrder, CockpitAxes)) -> Self {
        Self { origin, parts, command }
    }
}

impl<S: ImpulseStage> ImpulseStage for ConsoleGyroStage<'_, S> {
    fn resolve(&self) -> RigidImpulse {
        let base = self.origin.resolve();
        let (order, axes) = self.command;
        if !order.guidance_engaged() {
            return base;
        }
        let steer = if self.parts.iter().any(VehiclePart::attitude_pod) {
            order.world_torque(axes)
        } else {
            Vec3::ZERO
        };
        self.parts.iter().fold(base, |imp, part| {
            if part.flight_console() {
                imp.with_gyro(steer)
            } else {
                imp
            }
        })
    }
}

/// `ImpulseStage` decorator firing downward De Laval `RocketNozzle` bells with `r × F` lever-arm torque.
pub struct MainBellStage<'a, S> {
    origin: S,
    hull: (&'a [VehiclePart], &'a SurfaceFrame),
    trim: (f32, Vec3),
    command: (PilotOrder, CockpitAxes),
}

impl<'a, S> MainBellStage<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        hull: (&'a [VehiclePart], &'a SurfaceFrame),
        trim: (f32, Vec3),
        command: (PilotOrder, CockpitAxes),
    ) -> Self {
        Self { origin, hull, trim, command }
    }
}

impl<S: ImpulseStage> ImpulseStage for MainBellStage<'_, S> {
    fn resolve(&self) -> RigidImpulse {
        let (parts, frame) = self.hull;
        let (lift, com) = self.trim;
        let (order, axes) = self.command;
        parts.iter().fold(self.origin.resolve(), |imp, part| {
            part.bell_burn(lift, frame, com, &order, axes, imp)
        })
    }
}

/// `ImpulseStage` decorator firing 5-horn `RcsManifold` pods with physical `r × F` lever moments.
pub struct VernierAttitudeStage<'a, S> {
    origin: S,
    hull: (&'a [VehiclePart], &'a SurfaceFrame),
    trim: (f32, Vec3),
    command: (PilotOrder, CockpitAxes),
}

impl<'a, S> VernierAttitudeStage<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        hull: (&'a [VehiclePart], &'a SurfaceFrame),
        trim: (f32, Vec3),
        command: (PilotOrder, CockpitAxes),
    ) -> Self {
        Self { origin, hull, trim, command }
    }
}

impl<S: ImpulseStage> ImpulseStage for VernierAttitudeStage<'_, S> {
    fn resolve(&self) -> RigidImpulse {
        let (parts, frame) = self.hull;
        let (lift, com) = self.trim;
        let (order, axes) = self.command;
        let inner = self.origin.resolve();
        let lift_dir = frame.normal().normalize_or_zero();
        let main_lifting = inner.lifting_along(lift_dir);
        let raw_lin = order.world_linear(axes);
        let rcs_lin = if main_lifting {
            raw_lin - lift_dir * raw_lin.dot(lift_dir).max(0.0)
        } else {
            raw_lin
        };
        let rcs_torque = order.world_torque(axes);
        parts.iter().fold(inner, |imp, part| {
            part.attitude_burn(lift, frame, com, rcs_lin, rcs_torque, imp)
        })
    }
}
