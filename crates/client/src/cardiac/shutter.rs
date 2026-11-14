use std::f32::consts::PI;
#[cfg(not(test))]
use std::sync::OnceLock;
use super::AcousticWave;

/// Mechanical double-transient camera shutter click-clack acoustic waveform.
#[derive(Clone, Copy, Debug)]
pub struct ShutterClick {
    sample_rate: u32,
}

impl ShutterClick {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 24000 Hz mono for broadband mechanical shutter transients.
    pub const fn mechanical() -> Self {
        Self::new(24000)
    }

    fn broadband_noise(t: f32, rate: u32) -> f32 {
        let idx = ((t.max(0.0) * (rate as f32)) as u32).wrapping_add(1);
        let h = idx.wrapping_mul(747796405).wrapping_add(2891336453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277803737);
        let centered = ((w ^ (w >> 16)) as i32) as f32;
        centered / (i32::MAX as f32)
    }
}

impl AcousticWave for ShutterClick {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        let noise = Self::broadband_noise(t, self.sample_rate);
        let prev_noise = Self::broadband_noise((t - 1.0 / (self.sample_rate as f32)).max(0.0), self.sample_rate);
        let crisp = 0.65 * (noise - 0.45 * prev_noise);
        let open_transient = if (0.0..0.025).contains(&t) {
            let mirror_slap = (-125.0 * t).exp() * (2.0 * PI * (210.0 * t + 16.25 * (1.0 - (-130.0 * t).exp()))).cos() * 0.62;
            let blade_snap = (-240.0 * t).exp() * crisp * 0.78;
            let spring_ping = (-190.0 * t).exp() * (2.0 * PI * 1120.0 * t).cos() * 0.22;
            blade_snap + mirror_slap + spring_ping
        } else {
            0.0
        };
        let curtain_slide = if (0.025..0.062).contains(&t) {
            let dt = t - 0.025;
            (-65.0 * dt).exp() * crisp * 0.12
        } else {
            0.0
        };
        let close_transient = if (0.062..0.112).contains(&t) {
            let dt = t - 0.062;
            let cushion_thud = (-95.0 * dt).exp() * (2.0 * PI * (135.0 * dt + 11.25 * (1.0 - (-95.0 * dt).exp()))).cos() * 0.64;
            let arrest_snap = (-180.0 * dt).exp() * crisp * 0.68;
            arrest_snap + cushion_thud
        } else {
            0.0
        };
        (open_transient + curtain_slide + close_transient).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            static SHUTTER_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = SHUTTER_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_shutter_mech_v2.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.01);
        }
    }
}
