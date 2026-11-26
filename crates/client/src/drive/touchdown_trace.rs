use glam::Vec3;
use world::{PlanetaryWorld, VehicleAssembly};
use super::clearance::PadClearance;

pub const TOUCHDOWN_NODES: usize = 13;

/// Visual safety classification for the Belly-Camera predictive touchdown ring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchdownTone {
    SafeEmerald,
    CautionAmber,
    CrashCoral,
}

impl TouchdownTone {
    pub const fn color(self) -> Vec3 {
        match self {
            Self::SafeEmerald => Vec3::new(0.22, 0.98, 0.66),
            Self::CautionAmber => Vec3::new(1.00, 0.76, 0.24),
            Self::CrashCoral => Vec3::new(1.00, 0.34, 0.26),
        }
    }

    pub const fn badge_label(self) -> &'static str {
        match self {
            Self::SafeEmerald => "SAFE PAD",
            Self::CautionAmber => "BRAKE",
            Self::CrashCoral => "BURN NOW",
        }
    }
}

/// Gravity-integrated ventral ballistic arc and terrain impact footprint (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct TouchdownFootprint {
    trail: [Vec3; TOUCHDOWN_NODES],
    surface_up: Vec3,
    velocity: Vec3,
    radius: f32,
}

impl TouchdownFootprint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        trail: [Vec3; TOUCHDOWN_NODES],
        surface_up: Vec3,
        velocity: Vec3,
        radius: f32,
    ) -> Self {
        Self { trail, surface_up, velocity, radius }
    }

    pub fn shifted_by(self, delta: Vec3) -> Self {
        let mut moved = self.trail;
        for pt in &mut moved {
            *pt += delta;
        }
        Self::new(moved, self.surface_up, self.velocity, self.radius)
    }

    /// Transfers `(trail, surface_up, velocity, footprint_radius)` without getters.
    pub fn transfer<R>(
        &self,
        f: impl FnOnce(&[Vec3; TOUCHDOWN_NODES], Vec3, Vec3, f32) -> R,
    ) -> R {
        f(&self.trail, self.surface_up, self.velocity, self.radius)
    }

    pub fn classify_tone(&self) -> TouchdownTone {
        let up = self.surface_up.normalize_or_zero();
        let sink = (-self.velocity.dot(up)).max(0.0);
        let slip = (self.velocity - up * self.velocity.dot(up)).length();
        if sink > 12.5 || slip > 8.5 {
            TouchdownTone::CrashCoral
        } else if sink > 6.0 || slip > 2.8 {
            TouchdownTone::CautionAmber
        } else {
            TouchdownTone::SafeEmerald
        }
    }

    /// Invokes `f(unit_slip_dir, slip_speed)` strictly when horizontal drift exceeds `min_speed`.
    pub fn visit_slip(&self, min_speed: f32, f: impl FnOnce(Vec3, f32)) {
        let up = self.surface_up.normalize_or_zero();
        let raw_slip = self.velocity - up * self.velocity.dot(up);
        let speed = raw_slip.length();
        if speed > min_speed {
            f(raw_slip / speed, speed);
        }
    }
}

/// Polymorphic ventral ballistic prediction state eliminating `Option` in deep space.
#[derive(Clone, Copy, Debug)]
pub enum TouchdownTrace {
    DeepSpace,
    Predicted(TouchdownFootprint),
}

impl TouchdownTrace {
    pub fn sample(
        assembly: &VehicleAssembly,
        velocity: Vec3,
        clearance: PadClearance,
        world: &dyn PlanetaryWorld,
    ) -> Self {
        clearance.transfer(|in_well, altitude, _| {
            if !in_well {
                return Self::DeepSpace;
            }
            let origin = assembly.origin();
            let g_mag = world.apply_pull(origin, 1.0).length();
            if g_mag <= 0.05 {
                return Self::DeepSpace;
            }
            let surf_up = world.surface_up_at(origin).normalize_or_zero();
            let datum = origin.dot(surf_up) - altitude.max(0.0);
            let v_down = -velocity.dot(surf_up);
            let disc = (v_down * v_down + 2.0 * g_mag * altitude.max(0.25)).max(0.0);
            let t_est = ((disc.sqrt() - v_down) / g_mag).clamp(0.25, 24.0);
            let dt = t_est / ((TOUCHDOWN_NODES - 1) as f32);
            let mut trail = [origin; TOUCHDOWN_NODES];
            let (mut p, mut v) = (origin, velocity);
            let mut settled = false;
            for slot in trail.iter_mut().skip(1) {
                if settled {
                    *slot = p;
                    continue;
                }
                let next_v = v + world.apply_pull(p, dt);
                let cand = p + (v + next_v) * (0.5 * dt);
                if cand.dot(surf_up) <= datum {
                    p = world.constrain_to_surface(cand, 0.0);
                    settled = true;
                } else {
                    p = cand;
                    v = next_v;
                }
                *slot = p;
            }
            if !settled {
                let snapped = world.constrain_to_surface(p, 0.0);
                trail[TOUCHDOWN_NODES - 1] = snapped;
            }
            let impact_pt = trail[TOUCHDOWN_NODES - 1];
            let impact_up = world.surface_up_at(impact_pt).normalize_or_zero();
            let (_, hull_r) = assembly.hull_envelope();
            Self::Predicted(TouchdownFootprint::new(
                trail,
                if impact_up.length_squared() > 0.5 { impact_up } else { surf_up },
                velocity,
                hull_r.max(0.85),
            ))
        })
    }

    pub fn shifted_by(self, delta: Vec3) -> Self {
        match self {
            Self::DeepSpace => Self::DeepSpace,
            Self::Predicted(fp) => Self::Predicted(fp.shifted_by(delta)),
        }
    }

    pub fn visit_predicted(&self, f: impl FnOnce(&TouchdownFootprint)) {
        if let Self::Predicted(ref fp) = self {
            f(fp);
        }
    }
}
