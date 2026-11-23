use world::{HullWireframe, JetTone};
use acoustics::{AcousticWave, AttitudeHiss, FusionHum, HullChorus, HullDriveDemand, NozzleRoar};

/// Sampled peak throttles and lateral hull balance across active spacecraft nozzles.
#[derive(Clone, Copy, Debug)]
struct JetCensus {
    fusion_peak: f32,
    bell_peak: f32,
    rcs_peak: f32,
    rcs_pan: f32,
}

impl JetCensus {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(fusion_peak: f32, bell_peak: f32, rcs_peak: f32, rcs_pan: f32) -> Self {
        Self { fusion_peak, bell_peak, rcs_peak, rcs_pan }
    }

    fn from_wireframe(wireframe: &HullWireframe, flipping: bool, attitude_capable: bool) -> Self {
        let mut fusion_peak = 0.0_f32;
        let mut bell_peak = 0.0_f32;
        let mut rcs_peak = 0.0_f32;
        let mut pan_sum = 0.0_f32;
        let mut pan_weight = 0.0_f32;

        wireframe.visit_jets(|tone, mouth, _dir, thr| match tone {
            JetTone::FusionBlue => fusion_peak = fusion_peak.max(thr),
            JetTone::BellAmber => bell_peak = bell_peak.max(thr),
            JetTone::VernierCyan => {
                rcs_peak = rcs_peak.max(thr);
                pan_sum += mouth.x.clamp(-1.0, 1.0) * thr;
                pan_weight += thr;
            }
        });
        if flipping && attitude_capable {
            rcs_peak = rcs_peak.max(0.88);
        }
        let rcs_pan = if pan_weight > 1e-4 {
            (pan_sum / pan_weight).clamp(-0.68, 0.68)
        } else {
            0.0
        };
        Self::new(fusion_peak, bell_peak, rcs_peak, rcs_pan)
    }
}

/// Hull-conducted Expanse propulsion cadence driving continuous stereo `SoundDeck` loops.
#[derive(Clone, Copy, Debug)]
pub(super) struct HullBurnPulse {
    drive_clock: f32,
    rcs_timers: (f32, f32),
    prior_lit: (bool, bool, bool),
    valve_cadence: u32,
}

impl HullBurnPulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        drive_clock: f32,
        rcs_timers: (f32, f32),
        prior_lit: (bool, bool, bool),
        valve_cadence: u32,
    ) -> Self {
        Self { drive_clock, rcs_timers, prior_lit, valve_cadence }
    }

    pub(super) const fn silent() -> Self {
        Self::new(0.0, (0.0, 0.0), (false, false, false), 0)
    }

    fn thump_chorus(&self, fusion_lit: bool, ignite: bool, hiss: AttitudeHiss, pan: f32) {
        let _ = self.valve_cadence;
        if fusion_lit {
            let drive = if ignite { FusionHum::torch() } else { FusionHum::sustained() };
            if ignite {
                drive.thump();
            }
            HullChorus::cockpit(drive, hiss, pan).thump();
        } else {
            let drive = if ignite { NozzleRoar::ignition() } else { NozzleRoar::sustained() };
            if ignite {
                drive.thump();
            }
            HullChorus::cockpit(drive, hiss, pan).thump();
        }
    }

    fn thump_main_only(&self, fusion_lit: bool, ignite: bool) {
        let _ = self.drive_clock;
        if fusion_lit {
            let drive = if ignite { FusionHum::torch() } else { FusionHum::sustained() };
            drive.thump();
        } else {
            let drive = if ignite { NozzleRoar::ignition() } else { NozzleRoar::sustained() };
            drive.thump();
        }
    }

    pub(super) fn advance(
        &mut self,
        wireframe: &HullWireframe,
        flipping: bool,
        attitude_capable: bool,
        dt: f32,
    ) {
        let census = JetCensus::from_wireframe(wireframe, flipping, attitude_capable);
        let (was_main, was_rcs, was_flip) = self.prior_lit;
        let fusion_lit = census.fusion_peak > 0.03;
        let main_lit = fusion_lit || census.bell_peak > 0.03;
        let rcs_threshold = if was_rcs { 0.025 } else { 0.065 };
        let rcs_lit = census.rcs_peak > rcs_threshold;
        let flip_arrest = was_flip && !flipping && attitude_capable;

        if (!was_rcs && rcs_lit) || flip_arrest {
            self.rcs_timers.1 = 0.14;
        } else {
            self.rcs_timers.1 = (self.rcs_timers.1 - dt).max(0.0);
        }
        let stream_rcs = if rcs_lit {
            census.rcs_peak
        } else if self.rcs_timers.1 > 0.0 {
            0.72
        } else {
            0.0
        };
        if main_lit || stream_rcs > 0.0 || was_main || was_rcs {
            HullDriveDemand::new(census.fusion_peak, census.bell_peak, stream_rcs, census.rcs_pan)
                .publish();
        }

        if main_lit {
            let fresh_main = !was_main;
            let fresh_rcs = !was_rcs && rcs_lit;
            self.drive_clock -= dt;
            self.rcs_timers.0 = (self.rcs_timers.0 - dt).max(0.0);

            if fresh_main {
                if rcs_lit || flip_arrest {
                    let hiss = AttitudeHiss::valve_burst(
                        census.rcs_peak.max(0.65),
                        census.rcs_pan,
                        self.valve_cadence,
                    );
                    self.thump_chorus(fusion_lit, true, hiss, census.rcs_pan);
                    self.valve_cadence = self.valve_cadence.wrapping_add(1);
                    self.rcs_timers.0 = 0.20;
                } else {
                    self.thump_main_only(fusion_lit, true);
                }
                self.drive_clock = 0.36;
            } else if flip_arrest || (fresh_rcs && self.rcs_timers.0 <= 0.0) {
                let hiss = AttitudeHiss::valve_burst(
                    census.rcs_peak.max(0.65),
                    census.rcs_pan,
                    self.valve_cadence,
                );
                self.thump_chorus(fusion_lit, false, hiss, census.rcs_pan);
                self.valve_cadence = self.valve_cadence.wrapping_add(1);
                self.drive_clock = 0.34;
                self.rcs_timers.0 = 0.20;
            } else if self.drive_clock <= 0.0 {
                self.thump_main_only(fusion_lit, false);
                self.drive_clock = 0.36;
            }
        } else {
            self.drive_clock = 0.0;
            if rcs_lit || flip_arrest {
                let fresh_rcs = !was_rcs && rcs_lit;
                self.rcs_timers.0 -= dt;
                if flip_arrest || (fresh_rcs && self.rcs_timers.0 <= 0.0) {
                    AttitudeHiss::valve_burst(
                        census.rcs_peak.max(0.65),
                        census.rcs_pan,
                        self.valve_cadence,
                    )
                    .thump();
                    self.valve_cadence = self.valve_cadence.wrapping_add(1);
                    self.rcs_timers.0 = 0.20;
                } else if rcs_lit && self.rcs_timers.0 <= 0.0 {
                    AttitudeHiss::sustained_vent(census.rcs_peak, census.rcs_pan).thump();
                    self.rcs_timers.0 = 0.28;
                }
            } else {
                self.rcs_timers.0 = (self.rcs_timers.0 - dt).clamp(0.0, 0.04);
            }
        }
        self.prior_lit = (main_lit, rcs_lit || self.rcs_timers.1 > 0.0, flipping);
    }
}
