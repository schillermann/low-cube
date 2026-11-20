use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::super::part::VehiclePart;
use super::{CockpitAxes, PilotOrder, RigidImpulse, NOZZLE_THRUST};

/// Focused 1-method contract for composable stack-allocated spacecraft propulsion decorators.
pub trait ImpulseStage {
    fn resolve(&self) -> RigidImpulse;
}

/// Origin `ImpulseStage` summing linear dry mass across structural blocks and cockpit fixtures.
pub struct HullBallast<'a> {
    parts: &'a [VehiclePart],
}

impl<'a> HullBallast<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(parts: &'a [VehiclePart]) -> Self {
        Self { parts }
    }
}

impl ImpulseStage for HullBallast<'_> {
    fn resolve(&self) -> RigidImpulse {
        let mass = self.parts.iter().fold(0.0f32, |acc, part| acc + part.part_mass());
        RigidImpulse::with_mass(mass)
    }
}

/// `ImpulseStage` decorator folding reaction-wheel gyro authority from installed `FlightConsole` units.
pub struct ConsoleGyroStage<'a, S> {
    origin: S,
    parts: &'a [VehiclePart],
}

impl<'a, S> ConsoleGyroStage<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, parts: &'a [VehiclePart]) -> Self {
        Self { origin, parts }
    }
}

impl<S: ImpulseStage> ImpulseStage for ConsoleGyroStage<'_, S> {
    fn resolve(&self) -> RigidImpulse {
        self.parts.iter().fold(self.origin.resolve(), |imp, part| {
            if part.flight_console() {
                imp.with_gyro()
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

/// `ImpulseStage` decorator firing side-mounted `Steuerdüse` RCS pods with vernier ascent fallback.
pub struct VernierAttitudeStage<'a, S> {
    origin: S,
    hull: (&'a [VehiclePart], &'a SurfaceFrame),
    command: (PilotOrder, CockpitAxes),
}

impl<'a, S> VernierAttitudeStage<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        hull: (&'a [VehiclePart], &'a SurfaceFrame),
        command: (PilotOrder, CockpitAxes),
    ) -> Self {
        Self { origin, hull, command }
    }
}

impl<S: ImpulseStage> ImpulseStage for VernierAttitudeStage<'_, S> {
    fn resolve(&self) -> RigidImpulse {
        let (parts, frame) = self.hull;
        let (order, axes) = self.command;
        let inner = self.origin.resolve();
        let lift_dir = frame.normal().normalize_or_zero();
        let main_lifting = inner.lifting_along(lift_dir);
        let (mut next, mut vernier_sum) = (inner, 0.0f32);
        for part in parts {
            let (updated, lift_share) = part.attitude_burn(frame, &order, axes, next);
            next = updated;
            vernier_sum += lift_share;
        }
        if !main_lifting && vernier_sum > 0.01 {
            next.with_linear_boost(lift_dir * (NOZZLE_THRUST * vernier_sum.min(1.0)))
        } else {
            next
        }
    }
}
