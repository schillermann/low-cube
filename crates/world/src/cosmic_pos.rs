use glam::{DVec3, Vec3};

/// Sub-millimeter deterministic cosmic coordinate (`[i64; 3]` in `0.1 mm` steps = `10,000` ticks/m, `1` private field).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CosmicPos {
    ticks: [i64; 3],
}

impl CosmicPos {
    pub const TICKS_PER_METER: i64 = 10_000;
    pub const METERS_PER_TICK: f64 = 1.0e-4;
    pub const SECTOR_SPAN_TICKS: i64 = 4_096 * Self::TICKS_PER_METER;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ticks: [i64; 3]) -> Self {
        Self { ticks }
    }

    pub const fn origin() -> Self {
        Self::new([0, 0, 0])
    }

    pub const fn from_meters_i64(mx: i64, my: i64, mz: i64) -> Self {
        Self::new([
            mx.saturating_mul(Self::TICKS_PER_METER),
            my.saturating_mul(Self::TICKS_PER_METER),
            mz.saturating_mul(Self::TICKS_PER_METER),
        ])
    }

    pub fn from_dvec3(meters: DVec3) -> Self {
        Self::new([
            Self::quantize_axis(meters.x),
            Self::quantize_axis(meters.y),
            Self::quantize_axis(meters.z),
        ])
    }

    pub fn from_vec3(meters: Vec3) -> Self {
        Self::from_dvec3(meters.as_dvec3())
    }

    fn quantize_axis(meters: f64) -> i64 {
        if !meters.is_finite() {
            return 0;
        }
        let scaled = (meters * (Self::TICKS_PER_METER as f64)).clamp(-9.0e18, 9.0e18);
        scaled.round() as i64
    }

    pub const fn ticks(self) -> [i64; 3] {
        self.ticks
    }

    pub fn to_dvec3(self) -> DVec3 {
        DVec3::new(
            (self.ticks[0] as f64) * Self::METERS_PER_TICK,
            (self.ticks[1] as f64) * Self::METERS_PER_TICK,
            (self.ticks[2] as f64) * Self::METERS_PER_TICK,
        )
    }

    pub const fn shifted_ticks(self, dx: i64, dy: i64, dz: i64) -> Self {
        Self::new([
            self.ticks[0].saturating_add(dx),
            self.ticks[1].saturating_add(dy),
            self.ticks[2].saturating_add(dz),
        ])
    }

    pub const fn translated(self, offset: Self) -> Self {
        self.shifted_ticks(offset.ticks[0], offset.ticks[1], offset.ticks[2])
    }

    pub const fn delta_from(self, anchor: Self) -> Self {
        Self::new([
            self.ticks[0].saturating_sub(anchor.ticks[0]),
            self.ticks[1].saturating_sub(anchor.ticks[1]),
            self.ticks[2].saturating_sub(anchor.ticks[2]),
        ])
    }

    /// Subtracts `camera` in exact `i64` (`0.1 mm`) BEFORE converting to `f32` for jitter-free `wgpu` submission.
    pub fn relative_f32(self, camera: Self) -> Vec3 {
        let dx = self.ticks[0].saturating_sub(camera.ticks[0]);
        let dy = self.ticks[1].saturating_sub(camera.ticks[1]);
        let dz = self.ticks[2].saturating_sub(camera.ticks[2]);
        Vec3::new(
            ((dx as f64) * Self::METERS_PER_TICK) as f32,
            ((dy as f64) * Self::METERS_PER_TICK) as f32,
            ((dz as f64) * Self::METERS_PER_TICK) as f32,
        )
    }

    /// Exact `i64`-differenced `f64` offset vector (`self - anchor`) in meters.
    pub fn relative_dvec3(self, anchor: Self) -> DVec3 {
        self.delta_from(anchor).to_dvec3()
    }

    /// Overflow-free `i128` squared Euclidean distance in `0.1 mm²` ticks across the entire Heliosphere.
    pub fn distance_sq_i128(self, other: Self) -> i128 {
        let dx = i128::from(self.ticks[0]) - i128::from(other.ticks[0]);
        let dy = i128::from(self.ticks[1]) - i128::from(other.ticks[1]);
        let dz = i128::from(self.ticks[2]) - i128::from(other.ticks[2]);
        dx * dx + dy * dy + dz * dz
    }

    pub const fn macro_sector(self) -> [i64; 3] {
        [
            self.ticks[0].div_euclid(Self::SECTOR_SPAN_TICKS),
            self.ticks[1].div_euclid(Self::SECTOR_SPAN_TICKS),
            self.ticks[2].div_euclid(Self::SECTOR_SPAN_TICKS),
        ]
    }

    pub const fn nearest_sector(self) -> [i64; 3] {
        let half = Self::SECTOR_SPAN_TICKS / 2;
        [
            self.ticks[0].saturating_add(half).div_euclid(Self::SECTOR_SPAN_TICKS),
            self.ticks[1].saturating_add(half).div_euclid(Self::SECTOR_SPAN_TICKS),
            self.ticks[2].saturating_add(half).div_euclid(Self::SECTOR_SPAN_TICKS),
        ]
    }

    pub fn inscribe(self, out: &mut Vec<u8>) {
        for axis in self.ticks {
            out.extend_from_slice(&axis.to_le_bytes());
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 24 {
            return None;
        }
        let x = i64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let y = i64::from_le_bytes(bytes[8..16].try_into().ok()?);
        let z = i64::from_le_bytes(bytes[16..24].try_into().ok()?);
        Some((Self::new([x, y, z]), 24))
    }
}
