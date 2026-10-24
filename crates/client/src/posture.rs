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
        if self.perished() { self } else { Self::with_strain(self.elevation, 0.0, 0.0, 2.0, 1.0) }
    }

    /// Indicates whether the avatar has perished in the molten magma core.
    pub fn perished(&self) -> bool {
        self.hover > 1.5
    }

    /// Advances the cardiac arrest timer and indicates whether a booming heartbeat cycle just started.
    pub fn advance_perish(self, dt: f32) -> (Self, bool) {
        if !self.perished() { return (self, false); }
        let next_timer = self.phase_timer.max(0.0) + dt.max(0.0);
        let beat = self.phase_timer <= 0.0 || (self.phase_timer / 0.85).floor() < (next_timer / 0.85).floor();
        (Self::with_strain(self.elevation, 0.0, next_timer, self.hover, 1.0), beat)
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

    /// Progressive red tunnel-vision constriction in [0.0, 1.0] during core exposure or heart-attack death.
    pub fn cardiac_constriction(&self) -> f32 {
        if self.perished() {
            (0.52 + (self.phase_timer.max(0.0) * 0.28).min(0.36) + 0.10 * self.cardiac_throb()).clamp(0.0, 0.96)
        } else if self.strain > 0.0 {
            (self.strain * 0.82 + 0.08 * self.cardiac_throb()).clamp(0.0, 0.90)
        } else {
            0.0
        }
    }

    /// Narrowed camera field of view in radians pulsing with the cardiac strain and heartbeat.
    pub fn cardiac_fov(&self) -> f32 {
        (65.0 - 36.0 * self.cardiac_constriction() - 5.0 * self.cardiac_throb()).to_radians()
    }

    fn emit_red_ring(outer: f32, inner: f32, col: Vec3, target: &mut dyn world::MeshTarget) {
        crate::menu::ScreenBounds::new(-outer, inner, outer, outer).emit_fill(col, target);
        crate::menu::ScreenBounds::new(-outer, -outer, outer, -inner).emit_fill(col, target);
        crate::menu::ScreenBounds::new(-outer, -inner, -inner, inner).emit_fill(col, target);
        crate::menu::ScreenBounds::new(inner, -inner, outer, inner).emit_fill(col, target);
    }

    /// Emits the HUD crosshair and the progressive red tunnel-vision vignette when exposed near the core or perished.
    pub fn emit_reticle(&self, target: &mut dyn world::MeshTarget) {
        let constrict = self.cardiac_constriction();
        if constrict > 0.02 {
            let throb = self.cardiac_throb();
            let ap = (0.96 - 0.72 * constrict - 0.06 * throb).clamp(0.20, 0.96);
            let mid = (ap * 1.28).min(0.98);
            let outer_red = Vec3::new(0.26 + 0.18 * throb, 0.01, 0.01);
            let mid_red = Vec3::new(0.54 + 0.24 * throb, 0.02, 0.02);
            let inner_red = Vec3::new(0.84 + 0.16 * throb, 0.05 + 0.04 * throb, 0.03);
            Self::emit_red_ring(1.0, mid, outer_red, target);
            Self::emit_red_ring(mid, (ap * 1.10).min(mid), mid_red, target);
            Self::emit_red_ring((ap * 1.10).min(mid), ap, inner_red, target);
            let bev = ap * 0.30;
            for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let (x0, x1) = ((sx * (ap - bev)).min(sx * ap), (sx * (ap - bev)).max(sx * ap));
                let (y0, y1) = ((sy * (ap - bev)).min(sy * ap), (sy * (ap - bev)).max(sy * ap));
                crate::menu::ScreenBounds::new(x0, y0, x1, y1).emit_fill(mid_red, target);
            }
            if self.perished() {
                crate::menu::ScreenBounds::new(-0.42, -0.14, 0.42, 0.14)
                    .emit_framed(Vec3::new(0.28 + 0.12 * throb, 0.02, 0.02), Vec3::new(0.96, 0.16 + 0.12 * throb, 0.08), 0.008, target);
                crate::menu::PixelStamp::title(Vec3::new(1.0, 0.82, 0.28)).emit_centered("YOU DIED", 0.0, 0.0, target);
                return;
            }
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
        let next_strain = if weightless { (self.strain + dt / 12.0).min(1.0) } else { (self.strain - dt * 0.6).max(0.0) };
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
