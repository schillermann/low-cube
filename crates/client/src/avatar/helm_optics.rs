use glam::{Mat3, Quat, Vec3};
use crate::orientation::{MouseLook, SensitivityLook, ViewOrientation};

/// Encapsulated cockpit camera channel selector and 3D pan-tilt gimbal for hull-mounted sensor pods.
pub(super) struct HelmOptics {
    channel: u8,
    gimbal: SensitivityLook,
}

impl HelmOptics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(channel: u8, gimbal: SensitivityLook) -> Self {
        Self { channel, gimbal }
    }

    pub(super) fn cockpit() -> Self {
        Self::new(0, SensitivityLook::calibrated(Box::new(MouseLook::idle())))
    }

    pub(super) const fn cockpit_view(&self) -> bool {
        self.channel == 0
    }

    pub(super) const fn active(&self) -> bool {
        self.channel > 0
    }

    pub(super) fn select_channel(&mut self, channel: u8) {
        self.channel = channel;
        self.gimbal.anchor_heading(0.0, 0.0);
    }

    pub(super) fn reset(&mut self) {
        self.select_channel(0);
    }

    pub(super) fn cycle(&mut self, total: u8, forward: bool) {
        let next = if forward {
            (self.channel + 1) % total
        } else {
            (self.channel + total - 1) % total
        };
        self.select_channel(next);
    }

    pub(super) fn rotate(&mut self, dx: f32, dy: f32) {
        self.gimbal.rotate(dx, dy);
    }

    pub(super) fn resolve_lens(
        &self,
        views: &[(Vec3, Vec3, Vec3)],
        gimbal_enabled: bool,
    ) -> Option<(Vec3, Vec3, Vec3)> {
        if self.cockpit_view() {
            return None;
        }
        let (eye, base_fwd, base_up) = *views.get((self.channel as usize).saturating_sub(1))?;
        if !gimbal_enabled {
            return Some((eye, base_fwd, base_up));
        }
        let fwd = base_fwd.normalize_or_zero();
        let raw_right = fwd.cross(base_up);
        let right = if raw_right.length_squared() > 1e-6 {
            raw_right.normalize()
        } else {
            fwd.any_orthonormal_vector()
        };
        let up = right.cross(fwd).normalize_or_zero();
        let basis = Quat::from_mat3(&Mat3::from_cols(right, up, -fwd)).normalize();
        let aimed = self.gimbal.combine_with_surface(basis);
        Some((eye, (aimed * Vec3::NEG_Z).normalize(), (aimed * Vec3::Y).normalize()))
    }
}
