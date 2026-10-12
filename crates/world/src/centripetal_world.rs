use glam::Vec3;
use crate::{GravityField, MeshTarget, PlanetaryWorld};

/// Decorator blending surface normal gravity with spherical radial core pull at elevated altitudes ("Tell, Don't Ask").
pub struct CentripetalWorld {
    world: Box<dyn PlanetaryWorld>,
    half_size: f32,
    blend_margin: f32,
}

impl CentripetalWorld {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(world: Box<dyn PlanetaryWorld>, half_size: f32, blend_margin: f32) -> Self {
        Self {
            world,
            half_size,
            blend_margin,
        }
    }

    /// Evaluates blending weight based on elevation above the cubic planetary half-size.
    fn blend_weight(&self, position: Vec3) -> f32 {
        let radial_dist = position.length();
        let surface_radius = self.half_size;
        let altitude = radial_dist - surface_radius;
        if altitude <= 0.0 {
            0.0
        } else {
            let t = (altitude / self.blend_margin).clamp(0.0, 1.0);
            t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
        }
    }
}

impl GravityField for CentripetalWorld {
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        let base_up = self.world.surface_up_at(position);
        if position.length_squared() < 1e-6 {
            return base_up;
        }
        let radial_up = position.normalize();
        let weight = self.blend_weight(position);
        base_up.lerp(radial_up, weight).normalize()
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}

impl PlanetaryWorld for CentripetalWorld {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.world.constrain_to_surface(position, player_height)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.world.emit_surface(target);
    }
}
