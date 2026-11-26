mod cone;
#[cfg(test)]
mod tests;

use glam::Quat;
pub use self::cone::ConeLook;

/// Polymorphic upper-body yaw and pitch play when shoulders are wedged in a narrow shaft.
#[derive(Clone, Copy, Debug)]
pub enum ShoulderArc {
    Unobstructed(f32),
    Wedged(f32, f32),
}

impl ShoulderArc {
    pub fn clamp_yaw(self, yaw: f32) -> f32 {
        match self {
            Self::Unobstructed(_) => yaw,
            Self::Wedged(anchor, play) => yaw.clamp(anchor - play, anchor + play),
        }
    }

    pub fn clamp_pitch(self, pitch: f32) -> f32 {
        let limit = match self {
            Self::Unobstructed(_) => 1.54,
            Self::Wedged(_, _) => 0.96,
        };
        pitch.clamp(-limit, limit)
    }

    pub fn shift_anchor(self, delta: f32) -> Self {
        match self {
            Self::Unobstructed(yaw) => Self::Unobstructed(yaw + delta),
            Self::Wedged(anchor, play) => Self::Wedged(anchor + delta, play),
        }
    }

    pub fn transition(self, wedged: bool, current_yaw: f32, play: f32) -> Self {
        match (self, wedged) {
            (Self::Unobstructed(free_yaw), true) => Self::Wedged(free_yaw, play),
            (Self::Wedged(anchor, _), true) => Self::Wedged(anchor, play),
            (_, false) => Self::Unobstructed(current_yaw),
        }
    }
}

/// Interface for viewpoint rotation and surface alignment.
pub trait ViewOrientation {
    fn rotate(&mut self, dx: f32, dy: f32);
    fn level_pitch(&mut self);
    fn constrain_shoulders(&mut self, wedged: bool, play: f32);
    fn anchor_heading(&mut self, yaw: f32, play: f32);
    fn shift_anchor(&mut self, dx: f32);
    fn combine_with_surface(&self, surface_rot: Quat) -> Quat;
}

/// Unconstrained first-person Euler look orientation knowing nothing of shafts, sensitivity, or vehicles.
pub struct MouseLook {
    yaw: f32,
    pitch: f32,
}

impl MouseLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }

    /// Secondary constructor initializing idle zero angles.
    pub const fn idle() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl ViewOrientation for MouseLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx;
        self.pitch = (self.pitch - dy).clamp(-1.54, 1.54);
    }

    fn level_pitch(&mut self) {
        self.pitch = 0.0;
    }

    fn constrain_shoulders(&mut self, _wedged: bool, _play: f32) {}

    fn anchor_heading(&mut self, yaw: f32, _play: f32) {
        self.yaw = yaw;
        self.pitch = 0.0;
    }

    fn shift_anchor(&mut self, dx: f32) {
        self.yaw -= dx;
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        let local_rot = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
        surface_rot * local_rot
    }
}

/// Decorator constraining `ViewOrientation` yaw and pitch when shoulders are wedged in a narrow shaft or vehicle cockpit.
pub struct ShaftLook {
    look: Box<dyn ViewOrientation>,
    yaw: f32,
    pitch: f32,
    arc: ShoulderArc,
}

impl ShaftLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(look: Box<dyn ViewOrientation>, yaw: f32, pitch: f32, arc: ShoulderArc) -> Self {
        Self { look, yaw, pitch, arc }
    }

    pub fn unobstructed(look: Box<dyn ViewOrientation>) -> Self {
        Self::new(look, 0.0, 0.0, ShoulderArc::Unobstructed(0.0))
    }
}

impl ViewOrientation for ShaftLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        let next_yaw = self.arc.clamp_yaw(self.yaw - dx);
        let next_pitch = self.arc.clamp_pitch(self.pitch - dy);
        self.look.rotate(self.yaw - next_yaw, self.pitch - next_pitch);
        self.yaw = next_yaw;
        self.pitch = next_pitch;
    }

    fn level_pitch(&mut self) {
        self.look.level_pitch();
        self.pitch = 0.0;
    }

    fn constrain_shoulders(&mut self, wedged: bool, play: f32) {
        self.arc = self.arc.transition(wedged, self.yaw, play);
        let next_yaw = self.arc.clamp_yaw(self.yaw);
        let next_pitch = self.arc.clamp_pitch(self.pitch);
        self.look.rotate(self.yaw - next_yaw, self.pitch - next_pitch);
        self.yaw = next_yaw;
        self.pitch = next_pitch;
    }

    fn anchor_heading(&mut self, yaw: f32, play: f32) {
        self.look.anchor_heading(yaw, play);
        self.yaw = yaw;
        self.pitch = 0.0;
        self.arc = ShoulderArc::Wedged(yaw, play);
    }

    fn shift_anchor(&mut self, dx: f32) {
        self.arc = self.arc.shift_anchor(-dx);
        let next_yaw = self.arc.clamp_yaw(self.yaw - dx);
        self.look.rotate(self.yaw - next_yaw, 0.0);
        self.yaw = next_yaw;
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        self.look.combine_with_surface(surface_rot)
    }
}

/// Decorator scaling mouse rotational delta by an encapsulated sensitivity coefficient.
pub struct SensitivityLook {
    look: Box<dyn ViewOrientation>,
    sensitivity: f32,
}

impl SensitivityLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(look: Box<dyn ViewOrientation>, sensitivity: f32) -> Self {
        Self { look, sensitivity }
    }

    /// Secondary constructor wrapping orientation with calibrated sensitivity (0.003).
    pub fn calibrated(look: Box<dyn ViewOrientation>) -> Self {
        Self::new(look, 0.003)
    }
}

impl ViewOrientation for SensitivityLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        self.look.rotate(dx * self.sensitivity, dy * self.sensitivity);
    }

    fn level_pitch(&mut self) {
        self.look.level_pitch();
    }

    fn constrain_shoulders(&mut self, wedged: bool, play: f32) {
        self.look.constrain_shoulders(wedged, play);
    }

    fn anchor_heading(&mut self, yaw: f32, play: f32) {
        self.look.anchor_heading(yaw, play);
    }

    fn shift_anchor(&mut self, dx: f32) {
        self.look.shift_anchor(dx * self.sensitivity);
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        self.look.combine_with_surface(surface_rot)
    }
}
