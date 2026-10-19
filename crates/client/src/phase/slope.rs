use glam::Vec3;
use world::PlanetaryWorld;

/// Kinematic catapult entity launching avatars off upward ramps with boosted momentum.
#[derive(Clone, Copy, Debug)]
pub struct SkiLaunch {
    min_gradient: f32,
    vertical_boost_ratio: f32,
    momentum_boost_ratio: f32,
}

impl SkiLaunch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(min_gradient: f32, vertical_boost_ratio: f32, momentum_boost_ratio: f32) -> Self {
        Self {
            min_gradient,
            vertical_boost_ratio,
            momentum_boost_ratio,
        }
    }

    /// Secondary constructor calibrated for natural, subtle ramp kick launches.
    pub fn calibrated() -> Self {
        Self::new(0.15, 0.20, 0.15)
    }

    /// Evaluates ramp inclination and yields a natural launch kick and carried forward momentum.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_launch(
        &self,
        jump_strength: f32,
        stride: Vec3,
        surface_up: Vec3,
        origin: Vec3,
        dt: f32,
        eye: f32,
        world: &dyn PlanetaryWorld,
    ) -> (f32, Vec3) {
        if stride.length_squared() <= 1e-6 || dt <= 1e-6 {
            return (jump_strength, Vec3::ZERO);
        }
        let current_ground = world.constrain_to_surface(origin, eye);
        let ahead_ground = world.constrain_to_surface(origin + stride, eye);
        let ahead_rise = (ahead_ground - current_ground).dot(surface_up);

        if ahead_rise > 0.005 && ahead_rise <= 0.35 {
            let horizontal_delta = (ahead_ground - current_ground - surface_up * ahead_rise).length();
            if horizontal_delta > 1e-4 {
                let gradient = ahead_rise / horizontal_delta;
                if gradient > self.min_gradient {
                    let clamped = gradient.min(1.0);
                    let launch_speed = jump_strength * (1.0 + clamped * self.vertical_boost_ratio);
                    let forward_vel = stride / dt;
                    let launch_momentum = forward_vel * (clamped * self.momentum_boost_ratio);
                    return (launch_speed, launch_momentum);
                }
            }
        }
        (jump_strength, Vec3::ZERO)
    }
}

/// Kinematic deflection entity converting downward impact energy into forward downslope slide momentum.
#[derive(Clone, Copy, Debug)]
pub struct SlopeSurfing {
    min_downslope: f32,
    conversion_ratio: f32,
    friction: f32,
}

impl SlopeSurfing {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(min_downslope: f32, conversion_ratio: f32, friction: f32) -> Self {
        Self {
            min_downslope,
            conversion_ratio,
            friction,
        }
    }

    /// Secondary constructor calibrated for downhill slope-surfing and chained bounding.
    pub fn calibrated() -> Self {
        Self::new(0.15, 0.45, 3.5)
    }

    /// Converts downward landing speed into forward tangential slide momentum along a downslope.
    pub fn evaluate_touchdown(
        &self,
        impact_speed: f32,
        stride: Vec3,
        surface_up: Vec3,
        landing_pos: Vec3,
        eye: f32,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        if impact_speed <= 0.5 || stride.length_squared() <= 1e-6 {
            return Vec3::ZERO;
        }
        let current_ground = world.constrain_to_surface(landing_pos, eye);
        let ahead_ground = world.constrain_to_surface(landing_pos + stride, eye);
        let behind_ground = world.constrain_to_surface(landing_pos - stride, eye);

        let ground_drop = (behind_ground - current_ground).dot(surface_up);
        let ahead_drop = (current_ground - ahead_ground).dot(surface_up);

        if ground_drop > 0.03 && ground_drop <= 0.35 && ahead_drop > 0.03 && ahead_drop <= 0.35 {
            let horizontal_delta = (ahead_ground - current_ground + surface_up * ahead_drop).length();
            if horizontal_delta > 1e-4 {
                let gradient = ahead_drop / horizontal_delta;
                if gradient > self.min_downslope {
                    let steepness = gradient.min(1.0);
                    let tangent = (ahead_ground - current_ground).normalize();
                    return tangent * (impact_speed * steepness * self.conversion_ratio);
                }
            }
        }
        Vec3::ZERO
    }

    /// Damps residual slide momentum across elapsed time using surface friction.
    pub fn damp_momentum(&self, momentum: Vec3, dt: f32) -> Vec3 {
        if momentum.length_squared() > 1e-4 {
            momentum * (1.0 - (self.friction * dt).min(1.0))
        } else {
            Vec3::ZERO
        }
    }
}
