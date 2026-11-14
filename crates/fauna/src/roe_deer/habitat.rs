use std::f32::consts::TAU;
use glam::Vec3;
use world::{ClimateZone, PlanetaryWorld};

/// Temperate meadow and oak-glade habitat locator keeping woodland ungulates strictly inside `ClimateZone::Temperate`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TemperateGlade {
    hub: Vec3,
}

impl TemperateGlade {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(crate) const fn new(hub: Vec3) -> Self {
        Self { hub }
    }

    /// Secondary constructor resolving the nearest Temperate ground hub around `listener` or planetary `+Y` zenith.
    pub(crate) fn around(listener: Vec3, world: &dyn PlanetaryWorld) -> Self {
        let hub = if Self::temperate_surface(listener, world) {
            world.constrain_to_surface(Vec3::new(listener.x, listener.y + 6.0, listener.z), 0.0)
        } else {
            world.constrain_to_surface(Vec3::new(0.0, 2048.0, 0.0), 0.0)
        };
        Self::new(hub)
    }

    const fn next_hash(seed: u32) -> u32 {
        seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453)
    }

    pub(crate) fn temperate_surface(spot: Vec3, world: &dyn PlanetaryWorld) -> bool {
        ClimateZone::dominant(spot) == ClimateZone::Temperate && world.surface_up_at(spot).y > 0.85
    }

    pub(crate) fn oak_rooting_spot(&self, hash: u32, min_r: f32, span_r: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        for attempt in 0..8_u32 {
            let h = Self::next_hash(hash.wrapping_add(attempt.wrapping_mul(1153)));
            let (sa, ca) = (((h % 360) as f32) * (TAU / 360.0)).sin_cos();
            let radius = min_r + (((h >> 9) % 100) as f32) * 0.01 * span_r;
            let probe = Vec3::new(self.hub.x + ca * radius, self.hub.y + 8.0, self.hub.z + sa * radius);
            let ground = world.constrain_to_surface(probe, 0.0);
            if Self::temperate_surface(ground, world)
                && world.canopy_drag(ground + Vec3::Y * 2.4) > 0.01
                && world.canopy_drag(ground + Vec3::Y * 0.6) < 0.05
            {
                return ground;
            }
        }
        self.meadow_spot(hash, min_r, span_r, world)
    }

    pub(crate) fn meadow_spot(&self, hash: u32, min_r: f32, span_r: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        for attempt in 0..8_u32 {
            let h = Self::next_hash(hash.wrapping_add(attempt.wrapping_mul(977)));
            let (sa, ca) = (((h % 360) as f32) * (TAU / 360.0)).sin_cos();
            let radius = min_r + (((h >> 9) % 100) as f32) * 0.01 * span_r;
            let probe = Vec3::new(self.hub.x + ca * radius, self.hub.y + 8.0, self.hub.z + sa * radius);
            let ground = world.constrain_to_surface(probe, 0.0);
            if Self::temperate_surface(ground, world) && world.canopy_drag(ground + Vec3::Y * 1.1) < 0.05 {
                return ground;
            }
        }
        let fallback = world.constrain_to_surface(
            Vec3::new(self.hub.x * 0.35 + 3.5, self.hub.y + 8.0, self.hub.z * 0.35 - 3.5),
            0.0,
        );
        if Self::temperate_surface(fallback, world) {
            fallback
        } else {
            world.constrain_to_surface(Vec3::new(2.5, 2048.0, -2.5), 0.0)
        }
    }

    pub(crate) fn escape_from(&self, human_pos: Vec3, hash: u32, world: &dyn PlanetaryWorld) -> Vec3 {
        let raw = Vec3::new(self.hub.x - human_pos.x, 0.0, self.hub.z - human_pos.z);
        let away = if raw.length_squared() > 0.04 { raw.normalize() } else { Vec3::X };
        let side = Vec3::new(-away.z, 0.0, away.x);
        for attempt in 0..8_u32 {
            let h = Self::next_hash(hash.wrapping_add(attempt.wrapping_mul(1319)));
            let spread = (((h % 61) as i32 - 30) as f32) * 0.026 + (attempt as f32) * 0.42;
            let (ss, cs) = spread.sin_cos();
            let dir = (away * cs + side * ss).normalize_or_zero();
            let dist = 12.5 + (((h >> 8) % 50) as f32) * 0.11;
            let probe = Vec3::new(self.hub.x + dir.x * dist, self.hub.y + 8.0, self.hub.z + dir.z * dist);
            let ground = world.constrain_to_surface(probe, 0.0);
            if Self::temperate_surface(ground, world) && (ground - human_pos).length_squared() > 9.5 * 9.5 {
                return ground;
            }
        }
        Self::around(self.hub, world).meadow_spot(hash, 9.0, 6.0, world)
    }
}
