use glam::{DVec3, Vec3};
use world::{BodyId, BodySnapshot};
use super::magnetosphere::Magnetosphere;
use super::{AtmosphericEnvelope, AtmosphericShell, ShellOptics};

/// Focused contract resolving living `AtmosphericShell` envelopes (`Vacuum` Null Object for airless worlds).
pub trait AirCatalog {
    fn shell_for(&self, id: BodyId, rel_cam: Vec3, body_half: f32) -> AtmosphericShell;
    fn nearest_shell(&self, snaps: &[BodySnapshot], viewer: DVec3) -> AtmosphericShell;
}

/// Authoritative solar-system planetary atmosphere and magnetosphere catalog (`Earth`, `Mars`, `Venus`, `Titan`).
#[derive(Clone, Copy, Debug)]
pub struct SolarAirCatalog;

impl SolarAirCatalog {
    /// Primary constructor doing nothing except creating the catalog instance.
    pub const fn new() -> Self {
        Self
    }
}

impl AirCatalog for SolarAirCatalog {
    fn shell_for(&self, id: BodyId, rel_cam: Vec3, body_half: f32) -> AtmosphericShell {
        if id == BodyId::EARTH {
            AtmosphericShell::cubic(
                rel_cam,
                body_half,
                ShellOptics::new(Vec3::new(0.18, 0.56, 1.00), (body_half * 0.30).max(10.0)),
                Magnetosphere::terrestrial_arctic(),
            )
        } else if id == BodyId::MARS {
            AtmosphericShell::cubic(
                rel_cam,
                body_half,
                ShellOptics::new(Vec3::new(0.86, 0.48, 0.28), (body_half * 0.22).max(6.0)),
                Magnetosphere::crustal_wisps(),
            )
        } else if id == BodyId::VENUS {
            AtmosphericShell::cubic(
                rel_cam,
                body_half,
                ShellOptics::new(Vec3::new(0.95, 0.78, 0.42), (body_half * 0.38).max(12.0)),
                Magnetosphere::void(),
            )
        } else if id == BodyId::TITAN {
            AtmosphericShell::cubic(
                rel_cam,
                body_half,
                ShellOptics::new(Vec3::new(0.90, 0.58, 0.20), (body_half * 0.36).max(8.0)),
                Magnetosphere::saturnian_methane(),
            )
        } else {
            AtmosphericShell::vacuum()
        }
    }

    fn nearest_shell(&self, snaps: &[BodySnapshot], viewer: DVec3) -> AtmosphericShell {
        let mut closest_gap = f64::INFINITY;
        let mut closest_airless = false;
        for s in snaps.iter().skip(1) {
            let half = s.half_size();
            let gap = (viewer - s.center()).abs().max_element() - half;
            if gap < closest_gap {
                closest_gap = gap;
                let probe = self.shell_for(s.id(), Vec3::ZERO, half as f32);
                closest_airless = probe.airless() && gap <= half * 4.0;
            }
        }
        if closest_airless {
            return AtmosphericShell::vacuum();
        }
        let mut best_gap = f64::INFINITY;
        let mut best_shell = AtmosphericShell::vacuum();
        for s in snaps {
            let h = s.half_size() as f32;
            let rel_cam = viewer - s.center();
            let candidate = self.shell_for(s.id(), rel_cam.as_vec3(), h);
            let gap = rel_cam.abs().max_element() - s.half_size();
            if candidate.within_reach(gap) && gap < best_gap {
                best_gap = gap;
                best_shell = candidate;
            }
        }
        best_shell
    }
}
