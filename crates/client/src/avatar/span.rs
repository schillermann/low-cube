use glam::Vec3;

/// Immutable two-tick position span enabling sub-tick visual interpolation with zero causal drift.
#[derive(Clone, Copy, Debug)]
pub struct BodySpan {
    prior: Vec3,
    current: Vec3,
    blend: f32,
}

impl BodySpan {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(prior: Vec3, current: Vec3, blend: f32) -> Self {
        Self { prior, current, blend }
    }

    /// Secondary constructor anchoring both endpoints at `point` without interpolation offset.
    pub const fn anchored(point: Vec3) -> Self {
        Self::new(point, point, 1.0)
    }

    pub const fn current(self) -> Vec3 {
        self.current
    }

    pub const fn settled(self) -> Self {
        Self::anchored(self.current)
    }

    pub const fn advanced(self, next: Vec3) -> Self {
        Self::new(self.current, next, 1.0)
    }

    pub const fn blended(self, alpha: f32) -> Self {
        Self::new(self.prior, self.current, alpha.clamp(0.0, 1.0))
    }

    pub fn visual(self) -> Vec3 {
        self.prior.lerp(self.current, self.blend)
    }

    pub fn landed_on(world: &dyn world::PlanetaryWorld) -> Self {
        if let Some(rec) = world.assigned_survivor().and_then(|w| w.restored_record()) {
            Self::anchored(world.constrain_to_surface(rec.position(), 1.0))
        } else {
            Self::spawned_on(world)
        }
    }

    pub fn spawned_on(world: &dyn world::PlanetaryWorld) -> Self {
        let spawn = if let Some(beacon) = world.assigned_spawn_point() {
            world.constrain_to_surface(beacon, 1.0)
        } else {
            let probe_y = world
                .celestial_atlas()
                .ephemeris()
                .map_or(1_024.0, |e| (e.proportions().earth_half() * 2.0).max(1_024.0))
                + 64.0;
            [(0.25, 0.25), (2.25, 0.25), (-2.25, 0.25), (0.25, 2.25), (0.25, -2.25)]
                .into_iter()
                .map(|(x, z)| world.constrain_to_surface(Vec3::new(x, probe_y, z), 1.0))
                .fold(world.constrain_to_surface(Vec3::new(0.25, probe_y, 0.25), 1.0), |acc, c| {
                    if c.y > acc.y + 1.5 { c } else { acc }
                })
        };
        Self::anchored(spawn)
    }
}
