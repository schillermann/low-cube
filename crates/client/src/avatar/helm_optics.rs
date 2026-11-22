use glam::{Mat3, Mat4, Quat, Vec3};
use world::{HullOptic, MeshTarget};
use crate::drive::{OpticTelemetry, VehiclePilot};
use crate::menu::{
    ActiveChannel, CameraFeed, ChannelStripOverlay, GuidanceReadout, OpticOverlay, PixelStamp,
    ScreenPoint, ShipOpticHud,
};
use crate::orientation::{ConeLook, MouseLook, SensitivityLook, ViewOrientation};
use crate::posture::AvatarPosture;
use super::sight::AvatarSight;

const GIMBAL_RADIUS: f32 = 0.52;

/// Polymorphic optical telescope magnification stages for spacecraft hull cameras (`1×`, `6×`, `24×`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OpticZoom {
    Wide,
    Medium,
    Telephoto,
}

impl OpticZoom {
    pub(super) const fn cycled(self) -> Self {
        match self {
            Self::Wide => Self::Medium,
            Self::Medium => Self::Telephoto,
            Self::Telephoto => Self::Wide,
        }
    }

    pub(super) const fn factor(self) -> f32 {
        match self {
            Self::Wide => 1.0,
            Self::Medium => 6.0,
            Self::Telephoto => 24.0,
        }
    }

    pub(super) const fn badge(self) -> Option<&'static str> {
        match self {
            Self::Wide => None,
            Self::Medium => Some("OPTIC 6X"),
            Self::Telephoto => Some("OPTIC 24X"),
        }
    }
}

/// Immutable 3D gimbaled camera lens and fixed physical hull boresight target.
#[derive(Clone, Copy, Debug)]
pub(super) struct HullBoresight {
    eye: Vec3,
    aimed_fwd: Vec3,
    aimed_up: Vec3,
    aim_world: Vec3,
}

impl HullBoresight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(eye: Vec3, aimed_fwd: Vec3, aimed_up: Vec3, aim_world: Vec3) -> Self {
        Self { eye, aimed_fwd, aimed_up, aim_world }
    }

    pub(super) fn lens_with_zoom(self, zoom: OpticZoom) -> (Vec3, Vec3, Vec3, f32) {
        (self.eye, self.aimed_fwd, self.aimed_up, zoom.factor())
    }

    pub(super) fn sight_rays(self) -> ((Vec3, Vec3), (Vec3, Vec3)) {
        ((self.eye, (self.aim_world - self.eye).normalize_or_zero()), (self.eye, self.aimed_fwd))
    }

    pub(super) fn reticle_point(self, sight: &AvatarSight, aspect: f32, zoom: OpticZoom) -> ScreenPoint {
        let ndc = sight
            .project_zoomed_lens(aspect, self.lens_with_zoom(zoom))
            .project_point3(self.aim_world);
        ScreenPoint::new(ndc.x.clamp(-0.96, 0.96), ndc.y.clamp(-0.96, 0.96))
    }
}

/// Encapsulated cockpit camera channel, optical zoom, and pan-tilt gimbal for hull-mounted sensor pods.
pub(super) struct HelmOptics {
    channel: u8,
    zoom: OpticZoom,
    gimbal: SensitivityLook,
}

impl HelmOptics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(channel: u8, zoom: OpticZoom, gimbal: SensitivityLook) -> Self {
        Self { channel, zoom, gimbal }
    }

    pub(super) fn cockpit() -> Self {
        let cone = ConeLook::centered(Box::new(MouseLook::idle()), GIMBAL_RADIUS);
        Self::new(0, OpticZoom::Wide, SensitivityLook::calibrated(Box::new(cone)))
    }

    pub(super) const fn cockpit_view(&self) -> bool {
        self.channel == 0
    }

    pub(super) const fn active(&self) -> bool {
        self.channel > 0
    }

    pub(super) const fn zoom(&self) -> OpticZoom {
        self.zoom
    }

    pub(super) fn cycle_zoom(&mut self) {
        self.zoom = self.zoom.cycled();
    }

    pub(super) fn select_channel(&mut self, channel: u8) {
        self.channel = channel;
        self.zoom = OpticZoom::Wide;
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
        let s = 1.0 / self.zoom.factor();
        self.gimbal.rotate(dx * s, dy * s);
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

    fn channel_feed(
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

    pub(super) fn resolve_boresight(&self, views: &[HullOptic]) -> Option<HullBoresight> {
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
        Some(HullBoresight::new(
            eye,
            (aimed * Vec3::NEG_Z).normalize(),
            (aimed * Vec3::Y).normalize(),
            eye + fwd,
        ))
    }

    fn optic_telemetry(&self, views: &[HullOptic], pilot: &VehiclePilot) -> OpticTelemetry {
        if self.cockpit_view() {
            return OpticTelemetry::Silent;
        }
        let Some(selected) = views.get((self.channel as usize).saturating_sub(1)) else {
            return OpticTelemetry::Silent;
        };
        pilot.optic_telemetry(selected.transfer(|_, _, _, m| m))
    }

    pub(super) fn emit_hud(
        &self,
        pilot: &VehiclePilot,
        posture: AvatarPosture,
        sight: &AvatarSight,
        vp: Mat4,
        aspect: f32,
        target: &mut dyn MeshTarget,
    ) {
        if !pilot.driving() {
            posture.emit_reticle_at(ScreenPoint::center(), target);
            return;
        }
        let views = pilot.collected_views();
        let aim = self
            .resolve_boresight(&views)
            .map_or_else(ScreenPoint::center, |b| b.reticle_point(sight, aspect, self.zoom));
        posture.emit_reticle_at(aim, target);
        let ship_hud = ShipOpticHud::new(self.optic_telemetry(&views, pilot), vp, aim, aspect);
        if let Some(feed) = self.channel_feed(&views, pilot.guidance_readout(), pilot.flip_burn_readout()) {
            ChannelStripOverlay::new(ship_hud, feed, aspect).emit(target);
        } else {
            ship_hud.emit(target);
        }
        if let Some(badge) = self.zoom.badge().filter(|_| self.active()) {
            let ux = (1.0 / aspect.max(0.5)).clamp(0.25, 1.5);
            PixelStamp::new(0.0052 * ux, 0.0076, Vec3::new(0.42, 1.00, 0.78))
                .emit_centered(badge, 0.0, 0.88, target);
        }
    }

    pub(super) fn nose_sight_rays(&self, views: &[HullOptic]) -> Option<((Vec3, Vec3), (Vec3, Vec3))> {
        let selected = (!self.cockpit_view())
            .then(|| views.get((self.channel as usize).saturating_sub(1)))
            .flatten()?;
        if !matches!(selected.transfer(|_, _, _, m| m), world::OpticMount::RocketNose) {
            return None;
        }
        Some(self.resolve_boresight(views)?.sight_rays())
    }
}
