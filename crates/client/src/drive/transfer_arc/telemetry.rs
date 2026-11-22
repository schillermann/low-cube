use glam::Vec3;

/// Selectable trajectory solver mode in `SolarChartModal`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CourseMode {
    Ballistic,
    Brachistochrone,
}

impl CourseMode {
    pub const fn toggled(self) -> Self {
        match self {
            Self::Ballistic => Self::Brachistochrone,
            Self::Brachistochrone => Self::Ballistic,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ballistic => "MODE: BALLISTIC KEPLER",
            Self::Brachistochrone => "MODE: TORCH FLIP+BURN",
        }
    }
}

/// Polymorphic propulsion profile attached to a plotted `TransferArc`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CourseProfile {
    Ballistic {
        arrival_secs: f32,
    },
    MovingBody {
        body: world::BodyId,
        local_offset: Vec3,
        arrival_secs: f32,
    },
    Brachistochrone {
        flip_point: Vec3,
        total_secs: f32,
        burn_accel: f32,
    },
}

impl CourseProfile {
    pub fn anchored_to_ephemeris(self, target: Vec3, eph: world::SolarEphemeris) -> Self {
        let td = target.as_dvec3();
        for snap in eph.snapshot_all(0.0) {
            if snap.id() == world::BodyId::SOL || snap.id() == world::BodyId::EARTH {
                continue;
            }
            let rel = td - snap.center();
            if rel.abs().max_element() <= snap.half_size() + 12.0 {
                let secs = match self {
                    Self::Ballistic { arrival_secs }
                    | Self::MovingBody { arrival_secs, .. } => arrival_secs,
                    Self::Brachistochrone { total_secs, .. } => total_secs,
                };
                return Self::MovingBody {
                    body: snap.id(),
                    local_offset: rel.as_vec3(),
                    arrival_secs: secs,
                };
            }
        }
        self
    }

    pub fn live_destination(self, atlas: world::CelestialAtlas) -> Option<Vec3> {
        let Self::MovingBody { body, local_offset, .. } = self else {
            return None;
        };
        let snap = atlas.ephemeris()?.snapshot_all(0.0).into_iter().find(|s| s.id() == body)?;
        Some(snap.center().as_vec3() + local_offset)
    }
}

/// Polymorphic *The Expanse* Flip-and-Burn HUD status without `Option` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlipBurnReadout {
    Inactive,
    TimeToFlip(f32),
    FlipNow,
    BrakeBurn(f32),
}

/// Live course-alignment telemetry sampled from a spacecraft against a `TransferArc`.
#[derive(Clone, Copy, Debug)]
pub struct RouteTelemetry {
    anchor: Vec3,
    cross_track: f32,
    angle_deg: f32,
    aligned: bool,
}

impl RouteTelemetry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(anchor: Vec3, cross_track: f32, angle_deg: f32, aligned: bool) -> Self {
        Self { anchor, cross_track, angle_deg, aligned }
    }

    /// Transfers `(anchor_world, cross_track_meters, angle_deg, aligned)` without getters.
    pub fn transfer<R>(self, f: impl FnOnce(Vec3, f32, f32, bool) -> R) -> R {
        f(self.anchor, self.cross_track, self.angle_deg, self.aligned)
    }
}
