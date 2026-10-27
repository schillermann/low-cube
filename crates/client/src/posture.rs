use glam::{Mat4, Quat, Vec3};
use crate::attire::{BodyPose, StrideRhythm, YouthPhysique};
use crate::horizon::ViewFrame;

/// Kinematic posture defining the vertical eye level offset for standing, crouching, jumping, and weightless horizontal hover.
#[derive(Clone, Copy, Debug)]
pub struct AvatarPosture {
    elevation: f32, leap_flex: f32, phase_timer: f32, hover: f32, strain: f32,
}

impl AvatarPosture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_strain(elevation: f32, leap_flex: f32, phase_timer: f32, hover: f32, strain: f32) -> Self {
        Self { elevation, leap_flex, phase_timer, hover, strain }
    }
    pub fn with_hover(elevation: f32, leap_flex: f32, phase_timer: f32, hover: f32) -> Self { Self::with_strain(elevation, leap_flex, phase_timer, hover, 0.0) }
    pub fn with_leap(elevation: f32, leap_flex: f32, phase_timer: f32) -> Self { Self::with_hover(elevation, leap_flex, phase_timer, 0.0) }
    pub fn new(elevation: f32) -> Self { Self::with_leap(elevation, 0.0, 0.0) }

    /// Secondary constructor for upright standing posture.
    pub fn standing() -> Self { Self::new(0.8) }

    /// Secondary constructor for crouched posture in die Hocke.
    #[cfg(test)]
    pub fn crouching() -> Self { Self::new(0.36) }

    /// Effective vertical eye offset above the avatar center, dropping to 0.0 when horizontally hovering in zero-G.
    pub fn eye_elevation(&self) -> f32 {
        self.elevation * (1.0 - self.hover.min(1.0))
    }

    /// Marks the posture as perished in the molten magma core, preserving elapsed cardiac phase if already perished.
    pub fn perish(self) -> Self {
        if self.perished() { self } else { Self::with_strain(self.elevation, 0.0, 0.0, 2.0, self.strain.max(0.45)) }
    }

    /// Indicates whether the avatar has perished in the molten magma core.
    pub fn perished(&self) -> bool {
        self.hover > 1.5
    }

    /// Indicates whether the dying blackout has completely closed to pitch black so the avatar wakes up at spawn.
    pub fn blackout_complete(&self) -> bool {
        self.perished() && self.phase_timer >= 2.6
    }

    /// Advances the cardiac arrest timer and indicates whether a booming heartbeat cycle just started.
    pub fn advance_perish(self, dt: f32) -> (Self, bool) {
        if !self.perished() { return (self, false); }
        let next_timer = self.phase_timer.max(0.0) + dt.max(0.0);
        let beat = self.phase_timer <= 0.0 || (self.phase_timer / 0.85).floor() < (next_timer / 0.85).floor();
        (Self::with_strain(self.elevation, 0.0, next_timer, self.hover, self.strain), beat)
    }

    /// Instantaneous "lub-dub" cardiac pulse intensity in [0.0, 1.0].
    pub fn cardiac_throb(&self) -> f32 {
        let scale = if self.perished() { 1.0 } else { self.strain };
        if scale <= 0.0 { return 0.0; }
        let phase = self.phase_timer.max(0.0).rem_euclid(0.85);
        let wave = if phase < 0.16 {
            (phase / 0.16 * std::f32::consts::PI).sin().powi(2)
        } else if (0.20..0.36).contains(&phase) {
            0.78 * ((phase - 0.20) / 0.16 * std::f32::consts::PI).sin().powi(2)
        } else {
            0.0
        };
        wave * scale
    }

    /// Progressive tunnel-vision constriction in [0.0, 1.0] during core exposure or blackout.
    pub fn cardiac_constriction(&self) -> f32 {
        if self.perished() {
            let fade = (self.phase_timer.max(0.0) / 2.2).clamp(0.0, 1.0);
            let start = self.strain.clamp(0.45, 0.85);
            (start + (1.0 - start) * fade).clamp(0.0, 1.0)
        } else if self.strain > 0.0 {
            (self.strain * 0.85 + 0.04 * self.cardiac_throb()).clamp(0.0, 0.92)
        } else {
            0.0
        }
    }

    /// Narrowed camera field of view in radians pulsing with the cardiac strain and heartbeat.
    pub fn cardiac_fov(&self) -> f32 {
        (65.0 - 36.0 * self.cardiac_constriction() - 5.0 * self.cardiac_throb()).to_radians()
    }

    /// Emits the HUD crosshair and the shrinking circular tunnel-vision vignette with a transparent gradient to black.
    pub fn emit_reticle(&self, target: &mut dyn world::MeshTarget) {
        let constrict = self.cardiac_constriction();
        if constrict > 0.01 {
            let throb = self.cardiac_throb();
            let blackout = ((constrict - 0.72) / 0.28).clamp(0.0, 1.0);
            let inner_r = (1.22 * (1.0 - constrict) - 0.03 * throb * (1.0 - constrict)).max(0.0);
            let (outer_r, edge_alpha, min_alpha) = (1.48, (constrict * 1.15).clamp(0.18, 1.0), blackout * blackout);
            let (bands, segs) = (8usize, 24usize);
            for b in 0..bands {
                let (t0, t1) = ((b as f32) / (bands as f32), ((b + 1) as f32) / (bands as f32));
                let (r0, r1, tm) = (inner_r + (outer_r - inner_r) * t0, inner_r + (outer_r - inner_r) * t1, (t0 + t1) * 0.5);
                let a0 = (min_alpha + (1.0 - min_alpha) * t0.powf(1.3) * edge_alpha).clamp(0.02, 1.0);
                let a1 = (min_alpha + (1.0 - min_alpha) * t1.powf(1.3) * edge_alpha).clamp(0.04, 1.0);
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
            if constrict >= 0.96 { return; }
        }
        crate::menu::ScreenPoint::center().emit_crosshair(target);
    }

    /// Smoothly transitions posture towards a target elevation across elapsed time.
    pub fn transition_to(&self, target_elevation: f32, dt: f32) -> Self {
        if self.perished() { return *self; }
        let factor = (15.0 * dt).clamp(0.0, 1.0);
        let updated = self.elevation + (target_elevation - self.elevation) * factor;
        Self::with_strain(updated, self.leap_flex, self.phase_timer, self.hover, self.strain)
    }

    /// Smoothly transitions hover posture and accumulates or recovers core-proximity strain across elapsed time.
    pub fn advance_hover(self, weightless: bool, dt: f32) -> (Self, bool) {
        if self.perished() { return self.advance_perish(dt); }
        let target = if weightless { 1.0 } else { 0.0 };
        let next_hover = self.hover + (target - self.hover) * (14.0 * dt).clamp(0.0, 1.0);
        let next_strain = if weightless { (self.strain + dt / 120.0).min(1.0) } else { (self.strain - dt * 0.6).max(0.0) };
        if next_strain >= 1.0 {
            return Self::with_strain(self.elevation, 0.0, self.phase_timer.max(0.0), 2.0, 1.0).advance_perish(dt);
        }
        let next_timer = if next_strain > 0.0 { self.phase_timer.max(0.0) + dt.max(0.0) } else { self.phase_timer };
        let beat = next_strain > 0.15 && (self.phase_timer <= 0.0 || (self.phase_timer / 0.85).floor() < (next_timer / 0.85).floor());
        (Self::with_strain(self.elevation, self.leap_flex, next_timer, next_hover, next_strain), beat)
    }

    /// Smoothly advances takeoff knee bend, airborne leg extension, and landing knee compression without discontinuities.
    pub fn advance_leap(self, previous: Vec3, next: Vec3, terrain_up: Vec3, dt: f32) -> Self {
        let v_up = if dt > 1e-5 { (next - previous).dot(terrain_up) / dt } else { 0.0 };
        let (target, rate, leap_timer) = if self.phase_timer < 0.0 || v_up.abs() > 0.35 {
            let air_time = if self.phase_timer < 0.0 { -self.phase_timer + dt } else { dt };
            let landed = v_up.abs() <= 1e-3 && (air_time > 0.42 || self.leap_flex > 0.08);
            if landed {
                (-0.40, 26.0, 0.16)
            } else if air_time < 0.08 && v_up > 2.2 {
                (-0.38, 32.0, -air_time)
            } else {
                (0.36, 12.0, -air_time)
            }
        } else if self.phase_timer > 0.0 {
            (-0.40, 26.0, (self.phase_timer - dt).max(0.0))
        } else {
            (0.0, 10.0, 0.0)
        };
        let next_flex = self.leap_flex + (target - self.leap_flex) * (rate * dt).clamp(0.0, 1.0);
        let next_timer = if self.strain > 0.0 { self.phase_timer } else { leap_timer };
        Self::with_strain(self.elevation, next_flex, next_timer, self.hover, self.strain)
    }

    /// Computes view matrix using view frame and encapsulated eye elevation offset.
    pub fn view_matrix(&self, frame: &dyn ViewFrame, position: Vec3) -> Mat4 {
        frame.view_matrix_at(position, self.eye_elevation())
    }

    /// Assembles a kinematic BodyPose using this posture's encapsulated eye elevation, jump flex, and horizontal hover.
    pub fn body_pose(&self, position: Vec3, surface_rot: Quat, view_rot: Quat, rhythm: StrideRhythm) -> BodyPose {
        BodyPose::new(position, surface_rot, view_rot, self.eye_elevation(), rhythm, YouthPhysique::SlenderSquire)
            .with_leap(self.leap_flex * (1.0 - self.hover.min(1.0)))
            .with_hover(self.hover.min(1.0))
    }

    /// Verifies whether the avatar is currently in a crouched state.
    #[cfg(test)]
    pub fn crouched(&self) -> bool { self.elevation < 0.6 }
}
