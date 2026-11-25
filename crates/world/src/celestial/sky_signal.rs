use glam::Vec3;
use crate::{MeshTarget, Quad, SightCone};

const SKY_DOME_CAP: f32 = 1.6e6;

/// Chromatic signature distinguishing distant `FusionTorch` drive plumes from surface `LaserBeacon` emitters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalKind {
    FusionDriveStar,
    OpticalBeacon,
}

/// Pinpoint sky-dome telemetry vector representing a distant fusion burn or line-of-sight surface laser (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkySignal {
    direction: Vec3,
    range_km: f32,
    intensity: f32,
    kind: SignalKind,
}

impl SkySignal {
    pub const WIRE_SPAN: usize = 11;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(direction: Vec3, range_km: f32, intensity: f32, kind: SignalKind) -> Self {
        Self { direction, range_km, intensity, kind }
    }

    pub fn drive_star(direction: Vec3, range_km: f32, intensity: f32) -> Self {
        Self::new(direction.normalize_or_zero(), range_km.max(0.0), intensity.clamp(0.05, 1.0), SignalKind::FusionDriveStar)
    }

    pub fn optical_beacon(direction: Vec3, range_km: f32, intensity: f32) -> Self {
        Self::new(direction.normalize_or_zero(), range_km.max(0.0), intensity.clamp(0.05, 1.0), SignalKind::OpticalBeacon)
    }

    pub const fn kind(self) -> SignalKind { self.kind }
    pub const fn direction(self) -> Vec3 { self.direction }
    pub const fn range_km(self) -> f32 { self.range_km }
    pub const fn intensity(self) -> f32 { self.intensity }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        let dir = self.direction.normalize_or_zero();
        for c in [dir.x, dir.y, dir.z] {
            let q = (c.clamp(-1.0, 1.0) * 32_767.0).round() as i16;
            out.extend_from_slice(&q.to_le_bytes());
        }
        out.extend_from_slice(&self.range_km.max(0.0).to_le_bytes());
        let kind_bit = match self.kind {
            SignalKind::FusionDriveStar => 0u8,
            SignalKind::OpticalBeacon => 0x80u8,
        };
        let lum = (self.intensity.clamp(0.0, 1.0) * 127.0).round() as u8;
        out.push(kind_bit | (lum & 0x7F));
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < Self::WIRE_SPAN {
            return None;
        }
        let qx = i16::from_le_bytes([bytes[0], bytes[1]]) as f32 / 32_767.0;
        let qy = i16::from_le_bytes([bytes[2], bytes[3]]) as f32 / 32_767.0;
        let qz = i16::from_le_bytes([bytes[4], bytes[5]]) as f32 / 32_767.0;
        let dir = Vec3::new(qx, qy, qz).normalize_or_zero();
        if dir.length_squared() < 0.25 {
            return None;
        }
        let range_km = f32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
        if !range_km.is_finite() || range_km < 0.0 {
            return None;
        }
        let meta = bytes[10];
        let kind = if (meta & 0x80) == 0 {
            SignalKind::FusionDriveStar
        } else {
            SignalKind::OpticalBeacon
        };
        let intensity = f32::from(meta & 0x7F) / 127.0;
        Some((Self::new(dir, range_km, intensity.max(0.05), kind), Self::WIRE_SPAN))
    }

    pub fn emit(self, cone: SightCone, target: &mut dyn MeshTarget) {
        let dir = self.direction.normalize_or_zero();
        if dir.length_squared() < 0.25 || !cone.faces_direction(dir) {
            return;
        }
        let viewer = cone.observe_from(|eye| eye);
        let dist = (self.range_km.max(0.18) * 1_000.0).min(SKY_DOME_CAP);
        let center = viewer + dir * dist;
        let right = if dir.y.abs() < 0.92 {
            dir.cross(Vec3::Y).normalize_or_zero()
        } else {
            dir.cross(Vec3::X).normalize_or_zero()
        };
        let up = right.cross(dir).normalize_or_zero();
        let face_n = -dir;
        let lum = self.intensity.clamp(0.1, 1.0);
        match self.kind {
            SignalKind::FusionDriveStar => {
                let core = dist * (0.0018 + 0.0016 * lum);
                let arm = core * 3.6;
                let thin = core * 0.36;
                let core_col = Vec3::new(0.76, 0.97, 1.00);
                let spike_col = Vec3::new(0.32, 0.88, 1.00);
                let (rx, uy) = (right * core, up * core);
                target.push_quad(Quad::glowing(
                    center - rx - uy, center + rx - uy, center + rx + uy, center - rx + uy, face_n, core_col, 1.0,
                ));
                let (h_arm, h_thin) = (right * arm, up * thin);
                target.push_quad(Quad::glowing(
                    center - h_arm - h_thin, center + h_arm - h_thin, center + h_arm + h_thin, center - h_arm + h_thin,
                    face_n, spike_col, 0.92,
                ));
                let (v_thin, v_arm) = (right * thin, up * arm);
                target.push_quad(Quad::glowing(
                    center - v_thin - v_arm, center + v_thin - v_arm, center + v_thin + v_arm, center - v_thin + v_arm,
                    face_n, spike_col, 0.92,
                ));
            }
            SignalKind::OpticalBeacon => {
                let core = dist * (0.0016 + 0.0014 * lum);
                let halo = core * 2.2;
                let core_col = Vec3::new(0.56, 1.00, 0.78);
                let emerald = Vec3::new(0.16, 1.00, 0.58);
                target.push_quad(Quad::glowing(
                    center - right * halo, center - up * halo, center + right * halo, center + up * halo,
                    face_n, emerald, 0.94,
                ));
                let (rx, uy) = (right * core, up * core);
                target.push_quad(Quad::glowing(
                    center - rx - uy, center + rx - uy, center + rx + uy, center - rx + uy, face_n, core_col, 1.0,
                ));
            }
        }
    }
}
