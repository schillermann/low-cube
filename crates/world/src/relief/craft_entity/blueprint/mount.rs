use glam::Vec3;

/// Mounting station of a hull-mounted camera pod on a spacecraft or wheeled vehicle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpticMount {
    RocketNose,
    RocketBelly,
    RocketSideLeft,
    RocketSideRight,
    RocketSideFore,
    RocketSideAft,
    VehicleFront,
    VehicleRear,
    VehicleLeft,
    VehicleRight,
}

impl OpticMount {
    pub fn rocket_side(out_dir: Vec3, ship_fwd: Vec3, ship_up: Vec3) -> Self {
        let ship_right = ship_fwd.cross(ship_up).normalize_or_zero();
        let f = out_dir.dot(ship_fwd);
        let r = out_dir.dot(ship_right);
        if f.abs() >= r.abs() {
            if f >= 0.0 { Self::RocketSideFore } else { Self::RocketSideAft }
        } else if r < 0.0 {
            Self::RocketSideLeft
        } else {
            Self::RocketSideRight
        }
    }

    pub fn vehicle_direction(lens_fwd: Vec3, ship_fwd: Vec3, ship_up: Vec3) -> Self {
        let ship_right = ship_fwd.cross(ship_up).normalize_or_zero();
        let f = lens_fwd.dot(ship_fwd);
        let r = lens_fwd.dot(ship_right);
        if f.abs() >= r.abs() {
            if f >= 0.0 { Self::VehicleFront } else { Self::VehicleRear }
        } else if r < 0.0 {
            Self::VehicleLeft
        } else {
            Self::VehicleRight
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::RocketNose => "NOSE",
            Self::RocketBelly => "BELLY",
            Self::RocketSideLeft => "SIDE L",
            Self::RocketSideRight => "SIDE R",
            Self::RocketSideFore => "SIDE F",
            Self::RocketSideAft => "SIDE B",
            Self::VehicleFront => "FRONT",
            Self::VehicleRear => "REAR",
            Self::VehicleLeft => "LEFT",
            Self::VehicleRight => "RIGHT",
        }
    }
}

/// Hull-mounted optical camera channel uniting 3D `(eye, forward, up)` ray and `OpticMount` station.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HullOptic {
    eye: Vec3,
    forward: Vec3,
    up: Vec3,
    mount: OpticMount,
}

impl HullOptic {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eye: Vec3, forward: Vec3, up: Vec3, mount: OpticMount) -> Self {
        Self { eye, forward, up, mount }
    }

    pub fn shifted(self, delta: Vec3) -> Self {
        Self::new(self.eye + delta, self.forward, self.up, self.mount)
    }

    pub fn transfer<R>(self, f: impl FnOnce(Vec3, Vec3, Vec3, OpticMount) -> R) -> R {
        f(self.eye, self.forward, self.up, self.mount)
    }
}
