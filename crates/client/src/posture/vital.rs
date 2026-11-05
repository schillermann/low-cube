use glam::Vec3;

const BLACKOUT_DURATION: f32 = 2.6;
const HEARTBEAT_PERIOD: f32 = 0.85;

/// Zero-gravity horizontal hover, core-proximity cardiac strain, and dying blackout state.
#[derive(Clone, Copy, Debug)]
pub struct VitalStrain {
    hover: f32,
    strain: f32,
    cardiac_timer: f32,
    perished: bool,
}

impl VitalStrain {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(hover: f32, strain: f32, cardiac_timer: f32, perished: bool) -> Self {
        Self { hover, strain, cardiac_timer, perished }
    }

    pub const fn calm() -> Self {
        Self::new(0.0, 0.0, 0.0, false)
    }

    pub const fn perished(self) -> bool {
        self.perished
    }

    pub fn hover_factor(self) -> f32 {
        self.hover.clamp(0.0, 1.0)
    }

    pub fn perish(self) -> Self {
        if self.perished {
            self
        } else {
            Self::new(1.0, self.strain.max(0.45), 0.0, true)
        }
    }

    pub fn blackout_complete(self) -> bool {
        self.perished && self.cardiac_timer >= BLACKOUT_DURATION
    }

    fn perish_cycles(timer: f32) -> f32 {
        ((1.0 + 3.2 * timer.max(0.0)).sqrt() - 1.0) / 0.56
    }

    pub fn advance_perish(self, dt: f32) -> (Self, bool) {
        if !self.perished {
            return (self, false);
        }
        let next_timer = self.cardiac_timer.max(0.0) + dt.max(0.0);
        let beat = self.cardiac_timer <= 0.0
            || Self::perish_cycles(self.cardiac_timer).floor() < Self::perish_cycles(next_timer).floor();
        (Self::new(self.hover, self.strain, next_timer, true), beat)
    }

    pub fn advance_core_hover(self, weightless: bool, core_closeness: f32, dt: f32) -> (Self, bool) {
        if self.perished {
            return self.advance_perish(dt);
        }
        let target = if weightless { 1.0 } else { 0.0 };
        let next_hover = self.hover + (target - self.hover) * (14.0 * dt).clamp(0.0, 1.0);
        let closeness = core_closeness.clamp(0.0, 1.0);
        let next_strain = if weightless {
            let rate = (1.0 / 120.0) + closeness * closeness * 0.78;
            let floor = closeness * closeness * 0.72;
            let climbed = (self.strain + dt * rate).min(1.0);
            if climbed < floor {
                climbed + (floor - climbed) * (3.0 * dt).clamp(0.0, 1.0)
            } else {
                climbed
            }
        } else {
            (self.strain - dt * 0.6).max(0.0)
        };
        if next_strain >= 1.0 {
            return Self::new(1.0, 1.0, 0.0, true).advance_perish(dt);
        }
        let pace = 1.0 / (1.0 + 1.35 * next_strain * next_strain);
        let next_timer = if next_strain > 0.0 {
            self.cardiac_timer.max(0.0) + dt.max(0.0) * pace
        } else {
            0.0
        };
        let beat = next_strain > 0.15
            && (self.cardiac_timer <= 0.0
                || (self.cardiac_timer / HEARTBEAT_PERIOD).floor() < (next_timer / HEARTBEAT_PERIOD).floor());
        (Self::new(next_hover, next_strain, next_timer, false), beat)
    }

    pub fn cardiac_throb(self) -> f32 {
        let scale = if self.perished { 1.0 } else { self.strain };
        if scale <= 0.0 {
            return 0.0;
        }
        let phase = if self.perished {
            Self::perish_cycles(self.cardiac_timer).fract() * HEARTBEAT_PERIOD
        } else {
            self.cardiac_timer.max(0.0).rem_euclid(HEARTBEAT_PERIOD)
        };
        let wave = if phase < 0.16 {
            (phase / 0.16 * std::f32::consts::PI).sin().powi(2)
        } else if (0.20..0.36).contains(&phase) {
            0.78 * ((phase - 0.20) / 0.16 * std::f32::consts::PI).sin().powi(2)
        } else {
            0.0
        };
        wave * scale
    }

    pub fn cardiac_constriction(self) -> f32 {
        if self.perished {
            let fade = (self.cardiac_timer.max(0.0) / 2.2).clamp(0.0, 1.0);
            let start = self.strain.clamp(0.45, 0.85);
            (start + (1.0 - start) * fade).clamp(0.0, 1.0)
        } else if self.strain > 0.0 {
            (self.strain * 0.85 + 0.04 * self.cardiac_throb()).clamp(0.0, 0.92)
        } else {
            0.0
        }
    }

    pub fn cardiac_fov(self) -> f32 {
        (65.0 - 36.0 * self.cardiac_constriction() - 5.0 * self.cardiac_throb()).to_radians()
    }

    pub fn emit_reticle(self, target: &mut dyn world::MeshTarget) {
        let constrict = self.cardiac_constriction();
        if !self.perished && constrict < 0.65 {
            crate::menu::ScreenPoint::center().emit_crosshair(target);
        }
        if constrict <= 0.01 {
            return;
        }
        let throb = self.cardiac_throb();
        let blackout = ((constrict - 0.58) / 0.42).clamp(0.0, 1.0);
        let soft_inner = (0.78 * (1.0 - constrict) * (1.0 - 0.04 * throb)).max(0.0);
        let (outer_r, feather, edge_alpha, min_alpha) = (1.50, 0.85 + 0.35 * constrict, (constrict * 1.15).clamp(0.18, 1.0), blackout * blackout);
        let alpha_at = |r: f32| {
            let u = ((r - soft_inner * 0.45) / feather).clamp(0.0, 1.0);
            let curve = u * u * (3.0 - 2.0 * u);
            (min_alpha + (1.0 - min_alpha) * curve * edge_alpha).clamp(0.0, 1.0)
        };
        let (bands, segs) = (10usize, 32usize);
        for b in 0..bands {
            let (t0, t1) = ((b as f32) / (bands as f32), ((b + 1) as f32) / (bands as f32));
            let (r0, r1, tm) = (outer_r * t0, outer_r * t1, (t0 + t1) * 0.5);
            let (a0, a1) = (alpha_at(r0), alpha_at(r1).max(1e-4));
            let red = (1.0 - blackout) * (1.0 - tm * 0.75) * (0.16 + 0.12 * throb);
            let (col, grad_norm) = (Vec3::new(red, 0.0, 0.0), Vec3::new(r0, (r1 - r0).max(1e-4), (a0 / a1).clamp(0.0, 1.0)));
            for s in 0..segs {
                let (ang0, ang1) = ((s as f32) * std::f32::consts::TAU / (segs as f32), ((s + 1) as f32) * std::f32::consts::TAU / (segs as f32));
                let (c0, s0, c1, s1) = (ang0.cos(), ang0.sin(), ang1.cos(), ang1.sin());
                let (p0, p1) = (Vec3::new(c0 * r0, s0 * r0, 0.0), Vec3::new(c1 * r0, s1 * r0, 0.0));
                let (p2, p3) = (Vec3::new(c1 * r1, s1 * r1, 0.0), Vec3::new(c0 * r1, s0 * r1, 0.0));
                target.push_quad(world::Quad::glowing(p0, p1, p2, p3, grad_norm, col, -a1));
            }
        }
    }
}
