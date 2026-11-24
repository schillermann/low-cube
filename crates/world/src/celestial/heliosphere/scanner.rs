use glam::Vec3;
use super::super::belt::AsteroidBelt;
use super::super::catalog::BodyId;
use super::super::ephemeris::SolarEphemeris;

/// Zero-allocation stack scanner locating nearby landable planets, moons, or belt rocks (`<= 4` fields).
pub(super) struct BodyProximity {
    probe: Vec3,
    ephemeris: SolarEphemeris,
}

impl BodyProximity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(probe: Vec3, ephemeris: SolarEphemeris) -> Self {
        Self { probe, ephemeris }
    }

    pub(super) fn top_two(&self) -> [Option<(BodyId, Vec3)>; 2] {
        let probe_d = self.probe.as_dvec3();
        let snaps = self.ephemeris.snapshot_all(0.0);
        let mut first: Option<(f64, BodyId, Vec3)> = None;
        let mut second: Option<(f64, BodyId, Vec3)> = None;
        for snap in snaps {
            if snap.id() == BodyId::EARTH || !snap.nature().landable_crust() {
                continue;
            }
            let awaken_d = (snap.half_size() * 22.0).clamp(96.0, 900.0);
            let surf_d = (probe_d - snap.center()).abs().max_element() - snap.half_size();
            if surf_d <= awaken_d {
                let entry = (surf_d, snap.id(), snap.center().as_vec3());
                if first.map_or(true, |f| surf_d < f.0) {
                    second = first;
                    first = Some(entry);
                } else if second.map_or(true, |s| surf_d < s.0) {
                    second = Some(entry);
                }
            }
        }
        if first.is_none() {
            let sun = snaps[0];
            let belt = AsteroidBelt::new(self.ephemeris.proportions(), self.ephemeris.epoch_secs());
            if let Some(rock) = belt.nearest_rock(probe_d, sun.center(), sun.gm(), 64.0) {
                return [Some((rock.id(), rock.center().as_vec3())), None];
            }
            return [None, None];
        }
        [first.map(|(_, id, c)| (id, c)), second.map(|(_, id, c)| (id, c))]
    }
}
