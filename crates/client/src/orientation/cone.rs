use glam::Quat;
use super::ViewOrientation;

/// Decorator confining `ViewOrientation` pan and tilt within a circular angular cone around its optical axis.
pub struct ConeLook {
    look: Box<dyn ViewOrientation>,
    yaw: f32,
    pitch: f32,
    radius: f32,
}

impl ConeLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(look: Box<dyn ViewOrientation>, yaw: f32, pitch: f32, radius: f32) -> Self {
        Self { look, yaw, pitch, radius }
    }

    /// Secondary constructor centering pan and tilt on the optical axis within `radius` radians.
    pub fn centered(look: Box<dyn ViewOrientation>, radius: f32) -> Self {
        Self::new(look, 0.0, 0.0, radius)
    }
}

impl ViewOrientation for ConeLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        let raw_yaw = self.yaw - dx;
        let raw_pitch = (self.pitch - dy).clamp(-1.4, 1.4);
        let dist = raw_yaw.hypot(raw_pitch);
        let (next_yaw, next_pitch) = if dist > self.radius && dist > 1e-6 {
            let scale = self.radius / dist;
            (raw_yaw * scale, raw_pitch * scale)
        } else {
            (raw_yaw, raw_pitch)
        };
        self.look.rotate(self.yaw - next_yaw, self.pitch - next_pitch);
        self.yaw = next_yaw;
        self.pitch = next_pitch;
    }

    fn level_pitch(&mut self) {
        self.look.level_pitch();
        self.pitch = 0.0;
    }

    fn constrain_shoulders(&mut self, wedged: bool, play: f32) {
        self.look.constrain_shoulders(wedged, play);
    }

    fn anchor_heading(&mut self, yaw: f32, play: f32) {
        self.look.anchor_heading(yaw, play);
        self.yaw = 0.0;
        self.pitch = 0.0;
    }

    fn shift_anchor(&mut self, dx: f32) {
        self.look.shift_anchor(dx);
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        self.look.combine_with_surface(surface_rot)
    }
}
