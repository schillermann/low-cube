use glam::{DVec3, Vec3};
use world::{
    ClimateZone, CosmicFrame, DrivenCraft, GravityBubble, SkySignal, SolarEphemeris,
};
use crate::fleet_ledger::FleetLedger;

const MIN_DRIVE_STAR_METERS: f64 = 2_400.0;
const MIN_BEACON_ORBIT_METERS: f64 = 160.0;
const MAX_BEACON_ORBIT_METERS: f64 = 65_000.0;
const ATMOSPHERE_SHEATH_METERS: f32 = 180.0;
const MAX_DRIVE_STARS: usize = 4;
const MAX_OPTICAL_BEACONS: usize = 4;

/// Horizon- and telescope-culled celestial signal index for distant `FusionTorch` burns and `LaserBeacon`s (`<= 4` fields).
pub struct SkyHorizon<'a> {
    bubble: GravityBubble,
    ephemeris: SolarEphemeris,
    burns: Vec<(u32, DVec3, f32)>,
    beacons: &'a [(CosmicFrame, Vec3, Vec3)],
}

impl<'a> SkyHorizon<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        bubble: GravityBubble,
        ephemeris: SolarEphemeris,
        burns: Vec<(u32, DVec3, f32)>,
        beacons: &'a [(CosmicFrame, Vec3, Vec3)],
    ) -> Self {
        Self { bubble, ephemeris, burns, beacons }
    }

    pub fn indexed(
        crafts: &[(CosmicFrame, DrivenCraft, f32)],
        fleet: &FleetLedger,
        beacons: &'a [(CosmicFrame, Vec3, Vec3)],
        bubble: GravityBubble,
        ephemeris: SolarEphemeris,
    ) -> Self {
        let mut burns = Vec::new();
        for &(frame, dc, _) in crafts {
            let burn = dc.pose().wheel_spin().clamp(0.0, 1.0);
            if burn > 0.25 && fleet.fusion_equipped(dc.identity()) {
                let (origin, _, _) = bubble.heliocentric_state(frame);
                burns.push((dc.pilot_slot(), origin + dc.pose().origin().as_dvec3(), burn));
            }
        }
        Self::new(bubble, ephemeris, burns, beacons)
    }

    pub fn select_for_observer(
        &self,
        observer_slot: u32,
        frame: CosmicFrame,
        local_pos: Vec3,
        sky_watch: bool,
        out: &mut Vec<SkySignal>,
    ) {
        out.clear();
        if self.burns.is_empty() && self.beacons.is_empty() {
            return;
        }
        let (frame_helio, _, _) = self.bubble.heliocentric_state(frame);
        let obs_helio = frame_helio + local_pos.as_dvec3();
        self.select_drive_stars(observer_slot, frame, local_pos, obs_helio, sky_watch, out);
        self.select_optical_beacons(frame, local_pos, obs_helio, out);
    }

    fn select_drive_stars(
        &self,
        observer_slot: u32,
        frame: CosmicFrame,
        local_pos: Vec3,
        obs_helio: DVec3,
        sky_watch: bool,
        out: &mut Vec<SkySignal>,
    ) {
        if self.burns.is_empty() {
            return;
        }
        let body_half = self.bubble.frame_body_half(frame);
        let alt = local_pos.abs().max_element();
        let on_body = frame.anchored_body().is_some() && alt <= body_half + ATMOSPHERE_SHEATH_METERS;
        if on_body && alt < body_half - 1.5 {
            return;
        }
        let local_up = if on_body {
            ClimateZone::dominant(local_pos).normal()
        } else {
            Vec3::ZERO
        };
        if on_body && frame.atmospheric_sound() {
            if !sky_watch {
                return;
            }
            let eph = SolarEphemeris::with_frame(self.ephemeris.proportions(), self.ephemeris.epoch_secs(), frame);
            let sun_dir = eph.sun_direction_from(local_pos.as_dvec3(), 0.0);
            if local_up.dot(sun_dir) >= 0.05 {
                return;
            }
        }
        let min_elev = if frame.atmospheric_sound() { 0.15 } else { 0.02 };
        let mut candidates: Vec<(f32, SkySignal)> = Vec::new();
        for &(pilot_slot, ship_helio, burn) in &self.burns {
            if pilot_slot == observer_slot {
                continue;
            }
            let rel = ship_helio - obs_helio;
            let dist = rel.length();
            if dist <= MIN_DRIVE_STAR_METERS {
                continue;
            }
            let dir = (rel / dist).as_vec3();
            if on_body && dir.dot(local_up) <= min_elev {
                continue;
            }
            let lum = (burn * (50_000.0 / (dist as f32).max(2_400.0)).sqrt()).clamp(0.18, 1.0);
            candidates.push((lum, SkySignal::drive_star(dir, (dist * 0.001) as f32, lum)));
        }
        candidates.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
        out.extend(candidates.into_iter().take(MAX_DRIVE_STARS).map(|(_, s)| s));
    }

    fn select_optical_beacons(
        &self,
        obs_frame: CosmicFrame,
        local_pos: Vec3,
        obs_helio: DVec3,
        out: &mut Vec<SkySignal>,
    ) {
        if self.beacons.is_empty() {
            return;
        }
        let mut candidates: Vec<(f32, SkySignal)> = Vec::new();
        for &(b_frame, b_pos, b_normal) in self.beacons {
            let delta = if b_frame == obs_frame {
                (local_pos - b_pos).as_dvec3()
            } else {
                let (b_origin, _, _) = self.bubble.heliocentric_state(b_frame);
                obs_helio - (b_origin + b_pos.as_dvec3())
            };
            let dist = delta.length();
            if !(MIN_BEACON_ORBIT_METERS..=MAX_BEACON_ORBIT_METERS).contains(&dist) {
                continue;
            }
            let elev_cos = (delta.dot(b_normal.as_dvec3()) / dist) as f32;
            if elev_cos <= 0.05 {
                continue;
            }
            let dir = (-delta / dist).as_vec3();
            let lum = elev_cos.clamp(0.18, 1.0);
            candidates.push((lum, SkySignal::optical_beacon(dir, (dist * 0.001) as f32, lum)));
        }
        candidates.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
        out.extend(candidates.into_iter().take(MAX_OPTICAL_BEACONS).map(|(_, s)| s));
    }
}
