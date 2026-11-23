use glam::Vec3;
use super::medium::{
    CubicAirVolume, HorizonHazeMedium, OpticalMedium, SolarFluxMedium, VacuumMedium,
};
#[cfg(test)]
use super::strata::SmoothBand;
#[cfg(test)]
use super::strata::BandCurve;
use super::strata::{BarometricStratum, CrimsonDusk, TropopauseRim};

/// Composed 3D cubic Rayleigh atmospheric shell (`[-outer, +outer]^3`, `<= 4` fields) implementing `OpticalMedium`.
#[derive(Clone, Copy, Debug)]
pub struct CubicRayleighShell {
    body_half: f32,
    shell_thickness: f32,
    day_tint: Vec3,
    flux: f32,
}

impl CubicRayleighShell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(body_half: f32, shell_thickness: f32, day_tint: Vec3, flux: f32) -> Self {
        Self { body_half, shell_thickness, day_tint, flux }
    }
}

impl OpticalMedium for CubicRayleighShell {
    fn scatter_along(&self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> (Vec3, f32) {
        if self.shell_thickness <= 0.01 || self.body_half <= 0.5 {
            return VacuumMedium::new().scatter_along(cam_pos, ray_dir, sun_dir);
        }
        let stratum = TropopauseRim::cubic_ceiling(CrimsonDusk::terrestrial(
            BarometricStratum::new(self.day_tint),
        ));
        let medium = SolarFluxMedium::new(
            HorizonHazeMedium::pale(CubicAirVolume::new(
                self.body_half,
                self.shell_thickness,
                stratum,
            )),
            self.flux,
        );
        medium.scatter_along(cam_pos, ray_dir, sun_dir)
    }
}

/// Focused contract for solar body ray intersection and eclipse occultation.
#[cfg(test)]
pub trait SolarDisc: Copy {
    fn strikes_cube(&self, ray_dir: Vec3) -> bool;
    fn eclipse_transmittance(&self, ray_dir: Vec3, occluder: [f32; 4]) -> f32;
}

/// Analytical 3D cubic Sun ray intersection and square solar-eclipse occultation (`<= 4` fields).
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub struct CubicSolarBody {
    sun_dir: Vec3,
    pole: Vec3,
    angular_half: f32,
}

#[cfg(test)]
impl CubicSolarBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sun_dir: Vec3, pole: Vec3, angular_half: f32) -> Self {
        Self { sun_dir, pole, angular_half }
    }
}

#[cfg(test)]
impl SolarDisc for CubicSolarBody {
    fn strikes_cube(&self, ray_dir: Vec3) -> bool {
        let dir = ray_dir.normalize_or_zero();
        let fwd = self.sun_dir.normalize_or_zero();
        if dir.dot(fwd) <= 0.85 {
            return false;
        }
        let right = fwd.cross(self.pole).normalize_or_zero();
        let up = right.cross(fwd).normalize_or_zero();
        (dir - fwd).dot(right).abs().max((dir - fwd).dot(up).abs()) <= self.angular_half
    }

    fn eclipse_transmittance(&self, ray_dir: Vec3, occluder: [f32; 4]) -> f32 {
        let occ_half = occluder[3];
        if occ_half <= 0.0005 {
            return 1.0;
        }
        let dir = ray_dir.normalize_or_zero();
        let occ_dir = Vec3::new(occluder[0], occluder[1], occluder[2]).normalize_or_zero();
        if dir.dot(occ_dir) <= 0.92 {
            return 1.0;
        }
        let right = occ_dir.cross(self.pole).normalize_or_zero();
        let up = right.cross(occ_dir).normalize_or_zero();
        let box_d = (dir - occ_dir).dot(right).abs().max((dir - occ_dir).dot(up).abs());
        let smooth = SmoothBand::new(occ_half * 0.92, occ_half.max(occ_half * 0.92 + 1e-5)).sample(box_d);
        1.0 - (1.0 - smooth) * 0.985
    }
}
