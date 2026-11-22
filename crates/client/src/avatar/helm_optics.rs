use glam::{Mat3, Quat, Vec3};
use world::HullOptic;
use crate::drive::{OpticTelemetry, VehiclePilot};
use crate::menu::{ActiveChannel, CameraFeed, GuidanceReadout};
use crate::orientation::{ConeLook, MouseLook, SensitivityLook, ViewOrientation};

const GIMBAL_RADIUS: f32 = 0.52;

/// Encapsulated cockpit camera channel selector and small-radius pan-tilt gimbal for hull-mounted sensor pods.
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
        let cone = ConeLook::centered(Box::new(MouseLook::idle()), GIMBAL_RADIUS);
        Self::new(0, SensitivityLook::calibrated(Box::new(cone)))
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

    pub(super) fn gimbal_head_angles(&self) -> Option<(f32, f32)> {
        if self.cockpit_view() {
            return None;
        }
        let dir = (self.gimbal.combine_with_surface(Quat::IDENTITY) * Vec3::NEG_Z).normalize_or_zero();
        Some((-dir.x.atan2(-dir.z), dir.y.clamp(-0.9, 0.9).asin()))
    }

    pub(super) fn push_preview_rays(&self, views: &[HullOptic], out: &mut Vec<(Vec3, Vec3, Vec3)>) -> u8 {
        for optic in views {
            let (eye, base_fwd, base_up) = optic.transfer(|e, f, u, _| (e, f, u));
            let fwd = base_fwd.normalize_or_zero();
            let raw_right = fwd.cross(base_up);
            let right = if raw_right.length_squared() > 1e-6 {
                raw_right.normalize()
            } else {
                fwd.any_orthonormal_vector()
            };
            let up = right.cross(fwd).normalize_or_zero();
            out.push((eye, fwd, up));
        }
        self.channel
    }

    pub(super) fn channel_feed(
        &self,
        views: &[HullOptic],
        guidance: GuidanceReadout,
        flip_burn: crate::drive::FlipBurnReadout,
    ) -> Option<CameraFeed> {
        if views.is_empty() && flip_burn == crate::drive::FlipBurnReadout::Inactive {
            return None;
        }
        let total = views.len().min(u8::MAX as usize) as u8;
        if self.cockpit_view() || total == 0 {
            return Some(CameraFeed::new(ActiveChannel::Cockpit, total, guidance, flip_burn));
        }
        let selected = *views.get((self.channel as usize).saturating_sub(1))?;
        let mount = selected.transfer(|_, _, _, m| m);
        Some(CameraFeed::new(
            ActiveChannel::Camera(self.channel, mount),
            total,
            guidance,
            flip_burn,
        ))
    }

    pub(super) fn resolve_boresight(
        &self,
        views: &[HullOptic],
    ) -> Option<((Vec3, Vec3, Vec3), Vec3)> {
        if self.cockpit_view() {
            return None;
        }
        let selected = *views.get((self.channel as usize).saturating_sub(1))?;
        let (eye, base_fwd, base_up) = selected.transfer(|e, f, u, _| (e, f, u));
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
        let lens = (eye, (aimed * Vec3::NEG_Z).normalize(), (aimed * Vec3::Y).normalize());
        Some((lens, eye + fwd))
    }

    pub(super) fn optic_telemetry(&self, views: &[HullOptic], pilot: &VehiclePilot) -> OpticTelemetry {
        if self.cockpit_view() {
            return OpticTelemetry::Silent;
        }
        let Some(selected) = views.get((self.channel as usize).saturating_sub(1)) else {
            return OpticTelemetry::Silent;
        };
        pilot.optic_telemetry(selected.transfer(|_, _, _, m| m))
    }

    pub(super) fn nose_sight_rays(&self, views: &[HullOptic]) -> Option<((Vec3, Vec3), (Vec3, Vec3))> {
        let selected = (!self.cockpit_view())
            .then(|| views.get((self.channel as usize).saturating_sub(1)))
            .flatten()?;
        if !matches!(selected.transfer(|_, _, _, m| m), world::OpticMount::RocketNose) {
            return None;
        }
        let ((eye, aimed_fwd, _), aim_world) = self.resolve_boresight(views)?;
        Some(((eye, (aim_world - eye).normalize_or_zero()), (eye, aimed_fwd)))
    }
}
