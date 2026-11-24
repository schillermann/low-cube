use glam::{Quat, Vec3};
use world::{CelestialAtlas, PlanetaryWorld, VehicleAssembly};
use super::track::FaceContact;

const RADAR_CEILING: f32 = 105.0;

/// Polymorphic ventral ground-clearance state avoiding `Option` in deep space vs. planetary wells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PadClearance {
    DeepSpace,
    GravityWell { altitude: f32, range_rate: f32 },
}

impl PadClearance {
    /// Transfers `(in_well, altitude_meters, signed_range_rate_mps)` to a HUD collaborator without getters.
    pub fn transfer<R>(self, f: impl FnOnce(bool, f32, f32) -> R) -> R {
        match self {
            Self::DeepSpace => f(false, 0.0, 0.0),
            Self::GravityWell { altitude, range_rate } => f(true, altitude, range_rate),
        }
    }
}

/// Hull-sole and nadir terrain clearance probe shared between `TouchdownStep` physics and Belly-Camera radar.
pub struct HullClearance<'a> {
    assembly: &'a VehicleAssembly,
    origin: Vec3,
    velocity: Vec3,
    rotation: Quat,
}

impl<'a> HullClearance<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        assembly: &'a VehicleAssembly,
        origin: Vec3,
        velocity: Vec3,
        rotation: Quat,
    ) -> Self {
        Self { assembly, origin, velocity, rotation }
    }

    pub fn at_pose(assembly: &'a VehicleAssembly, velocity: Vec3) -> Self {
        Self::new(assembly, assembly.origin(), velocity, Quat::IDENTITY)
    }

    pub(super) fn span_projection(&self, surf_up: Vec3, rot_up: Vec3) -> f32 {
        let (hull_top, hull_radius) = self.assembly.hull_envelope();
        let c = rot_up.dot(surf_up).clamp(-1.0, 1.0);
        let sin_perp = (1.0 - c * c).max(0.0).sqrt();
        -((c * hull_top).min(0.0) - sin_perp * hull_radius)
    }

    pub(super) fn local_contact(&self, world: &dyn PlanetaryWorld) -> (Vec3, Vec3, f32, f32) {
        let surf_up = world.surface_up_at(self.origin);
        let (hull_top, hull_radius) = self.assembly.hull_envelope();
        let rotated_up = (self.rotation * self.assembly.normal()).normalize_or_zero();
        let ground_base = FaceContact::new(self.origin).ground(0.0, world);
        let nose_pt = self.origin + rotated_up * hull_top;
        let ground_nose = FaceContact::new(nose_pt).ground(0.0, world);
        let mut ground = if rotated_up.dot(surf_up) < 0.75
            && world.surface_up_at(nose_pt).dot(surf_up) > 0.5
            && (ground_nose - ground_base).dot(surf_up) > 0.0
        {
            self.origin + surf_up * (ground_nose - self.origin).dot(surf_up)
        } else {
            ground_base
        };
        if rotated_up.dot(surf_up) >= 0.75
            && (self.origin - ground_base).dot(surf_up) <= hull_top + hull_radius + 4.0
        {
            let posed = self.assembly.pose().with_origin(self.origin).rotated(self.rotation);
            self.assembly.visit_hull_soles(posed, |sole_pt| {
                let sole_ground = FaceContact::new(sole_pt).ground(0.0, world);
                let deficit = (sole_ground - (sole_pt - self.origin) - ground).dot(surf_up);
                if deficit > 0.0 && deficit <= 3.0 {
                    ground += surf_up * deficit;
                }
            });
        }
        let clearance = self.span_projection(surf_up, rotated_up);
        let alt = (self.origin - ground).dot(surf_up) - clearance;
        let v_norm = self.velocity.dot(surf_up);
        (ground, surf_up, alt, v_norm)
    }

    pub(super) fn crust_contact(&self, world: &dyn PlanetaryWorld) -> Option<(Vec3, Vec3, f32, f32)> {
        world
            .surface_contact(self.origin, 0.0)
            .resolve_crust(None, |_, _| Some(self.local_contact(world)))
    }

    fn sample_body(
        &self,
        body_center: Vec3,
        crust_half: f32,
        world: &dyn PlanetaryWorld,
    ) -> PadClearance {
        let rel = self.origin - body_center;
        let cheb = rel.abs().max_element();
        let seed_bound = crust_half + 6.0;
        let radial_seed = body_center + if cheb > seed_bound { rel * (seed_bound / cheb) } else { rel };
        if world.apply_pull(radial_seed, 1.0).length_squared() <= 0.04 {
            return PadClearance::DeepSpace;
        }
        if cheb <= crust_half + 32.0 {
            let (_, _, alt, v_norm) = self.local_contact(world);
            return PadClearance::GravityWell { altitude: alt.max(0.0), range_rate: v_norm };
        }
        let surf_up = world.surface_up_at(radial_seed).normalize_or_zero();
        let raw_tan = rel - surf_up * rel.dot(surf_up);
        let tangent = raw_tan.clamp(Vec3::splat(-crust_half), Vec3::splat(crust_half));
        let nadir_probe = body_center + tangent + surf_up * (crust_half + 12.0);
        let ground = world.constrain_to_surface(nadir_probe, 0.0);
        let rotated_up = (self.rotation * self.assembly.normal()).normalize_or_zero();
        let clearance = self.span_projection(surf_up, rotated_up);
        let delta = self.origin - ground;
        let span_dist = if (raw_tan - tangent).length_squared() <= 1e-3 {
            delta.dot(surf_up)
        } else {
            delta.length()
        };
        let alt = (span_dist - clearance).max(0.0);
        PadClearance::GravityWell { altitude: alt, range_rate: self.velocity.dot(surf_up) }
    }

    pub fn sample_pad(&self, atlas: CelestialAtlas, world: &dyn PlanetaryWorld) -> PadClearance {
        atlas.transfer(|home_half, moon_center, moon_half| {
            let h_half = home_half.max(4.0);
            let home_gap = (self.origin.abs() - Vec3::splat(h_half)).max(Vec3::ZERO).length();
            let moon_gap = if moon_half > 0.0 {
                ((self.origin - moon_center).abs() - Vec3::splat(moon_half))
                    .max(Vec3::ZERO)
                    .length()
            } else {
                f32::MAX
            };
            if moon_gap < home_gap && moon_gap <= RADAR_CEILING {
                self.sample_body(moon_center, moon_half, world)
            } else if home_gap <= RADAR_CEILING {
                self.sample_body(Vec3::ZERO, h_half, world)
            } else {
                PadClearance::DeepSpace
            }
        })
    }
}
