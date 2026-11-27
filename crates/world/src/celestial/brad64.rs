use std::f64::consts::{FRAC_PI_2, PI, TAU};

const ONE_Q60: i128 = 1_i128 << 60;
const PI_HALF_Q60: u128 = 0x1921_FB54_442D_1847;
const INV_ONE_Q60: f64 = 1.0 / ((1_u64 << 60) as f64);
const INV_TAU: f64 = 1.0 / TAU;
const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;

/// Platform-independent 64-bit binary angle (`[0, 2π)` mapped to `0..2^64`, `0` OS `libm` calls, `1` private field).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Brad64 {
    turns_q64: u64,
}

impl Brad64 {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(turns_q64: u64) -> Self {
        Self { turns_q64 }
    }

    pub const fn zero() -> Self {
        Self::new(0)
    }

    /// Converts IEEE-754 `turns` (`1.0 == 2π`) into `Brad64` using hardware `floor` without `libm::fmod`.
    pub fn from_turns(turns: f64) -> Self {
        if !turns.is_finite() {
            return Self::zero();
        }
        let frac = (turns - turns.floor()).clamp(0.0, 0.999_999_999_999_999_9);
        Self::new((frac * TWO_POW_64) as u64)
    }

    /// Converts radians into `Brad64` without `libm::fmod`.
    pub fn from_radians(radians: f64) -> Self {
        Self::from_turns(radians * INV_TAU)
    }

    /// Wraps `radians` deterministically into `(-π, π]` using IEEE-754 `floor` (`0` `libm` calls).
    pub fn wrap_signed_radians(radians: f64) -> f64 {
        if !radians.is_finite() {
            return 0.0;
        }
        let turns = radians * INV_TAU;
        let mut wrapped = (turns - turns.floor()) * TAU;
        if wrapped > PI {
            wrapped -= TAU;
        }
        wrapped
    }

    /// Pure-Rust `i128` `Q2.60` polynomial `(sin, cos)` bit-identical across Linux, Windows, x86_64, and ARM64.
    pub fn sin_cos_f64(self) -> (f64, f64) {
        let quadrant = (self.turns_q64 >> 62) & 3;
        let u_q62 = self.turns_q64 & ((1_u64 << 62) - 1);
        let half_quad = 1_u64 << 61;
        let (v_q62, swapped) = if u_q62 <= half_quad {
            (u_q62, false)
        } else {
            ((1_u64 << 62) - u_q62, true)
        };

        let x = (((v_q62 as u128) * PI_HALF_Q60) >> 62) as i128;
        let x2 = Self::mul_q60(x, x);

        let sin_poly = Self::horner_sin(x, x2);
        let cos_poly = Self::horner_cos(x2);
        let (s_q, c_q) = if swapped { (cos_poly, sin_poly) } else { (sin_poly, cos_poly) };

        let (sin_signed, cos_signed) = match quadrant {
            0 => (s_q, c_q),
            1 => (c_q, -s_q),
            2 => (-s_q, -c_q),
            _ => (-c_q, s_q),
        };
        ((sin_signed as f64) * INV_ONE_Q60, (cos_signed as f64) * INV_ONE_Q60)
    }

    const fn mul_q60(a: i128, b: i128) -> i128 {
        (a * b) >> 60
    }

    const fn horner_sin(x: i128, x2: i128) -> i128 {
        let r7 = ONE_Q60 - Self::mul_q60(x2, ONE_Q60 / 210);
        let r6 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r7, ONE_Q60 / 156));
        let r5 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r6, ONE_Q60 / 110));
        let r4 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r5, ONE_Q60 / 72));
        let r3 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r4, ONE_Q60 / 42));
        let r2 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r3, ONE_Q60 / 20));
        let r1 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r2, ONE_Q60 / 6));
        Self::mul_q60(x, r1)
    }

    const fn horner_cos(x2: i128) -> i128 {
        let r7 = ONE_Q60 - Self::mul_q60(x2, ONE_Q60 / 182);
        let r6 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r7, ONE_Q60 / 132));
        let r5 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r6, ONE_Q60 / 90));
        let r4 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r5, ONE_Q60 / 56));
        let r3 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r4, ONE_Q60 / 30));
        let r2 = ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r3, ONE_Q60 / 12));
        ONE_Q60 - Self::mul_q60(x2, Self::mul_q60(r2, ONE_Q60 / 2))
    }

    /// Pure-Rust IEEE-754 `atan2(y, x)` in `[0, 2π)` radians using double half-angle `sqrt` reduction (`0` `libm` calls).
    pub fn atan2_radians(y: f64, x: f64) -> f64 {
        if !x.is_finite() || !y.is_finite() || (x == 0.0 && y == 0.0) {
            return 0.0;
        }
        let (ax, ay) = (x.abs(), y.abs());
        let (num, den, swapped) = if ay <= ax { (ay, ax, false) } else { (ax, ay, true) };
        let t = num / den;
        let t1 = t / (1.0 + (1.0 + t * t).sqrt());
        let u = t1 / (1.0 + (1.0 + t1 * t1).sqrt());
        let u2 = u * u;
        let series = 1.0
            + u2 * (-1.0 / 3.0
                + u2 * (1.0 / 5.0
                    + u2 * (-1.0 / 7.0 + u2 * (1.0 / 9.0 + u2 * (-1.0 / 11.0 + u2 * (1.0 / 13.0))))));
        let mut base = 4.0 * u * series;
        if swapped {
            base = FRAC_PI_2 - base;
        }
        let signed = match (x >= 0.0, y >= 0.0) {
            (true, true) => base,
            (false, true) => PI - base,
            (false, false) => PI + base,
            (true, false) => TAU - base,
        };
        if signed >= TAU { signed - TAU } else { signed }
    }
}
