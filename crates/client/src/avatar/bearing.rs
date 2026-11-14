use glam::Vec3;
use world::MeshTarget;
use crate::attire::{CameraMount, PhotoShutter, SightMode, StrideRhythm};
use crate::photo_plate::CameraExposure;

/// Encapsulated kinematic bearing: horizon tilt, stride rhythm, sight mode, and camera exposure state.
pub struct AvatarBearing {
    tilt_radians: f32,
    rhythm: StrideRhythm,
    sight: SightMode,
    exposure: CameraExposure,
}

impl AvatarBearing {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_exposure(tilt_radians: f32, rhythm: StrideRhythm, sight: SightMode, exposure: CameraExposure) -> Self {
        Self { tilt_radians, rhythm, sight, exposure }
    }

    pub const fn new(tilt_radians: f32, rhythm: StrideRhythm, sight: SightMode) -> Self {
        Self::with_exposure(tilt_radians, rhythm, sight, CameraExposure::idle())
    }

    pub const fn rhythm(&self) -> StrideRhythm {
        self.rhythm
    }

    pub const fn sight(&self) -> SightMode {
        self.sight
    }

    pub const fn tilt_radians(&self) -> f32 {
        self.tilt_radians
    }

    pub fn press_shutter(&mut self, mount: CameraMount) -> bool {
        self.exposure.press_shutter(mount)
    }

    pub fn reconcile_mount(&mut self, mount: CameraMount) {
        self.exposure.reconcile_mount(mount);
    }

    pub fn take_shutter(&mut self) -> PhotoShutter {
        self.exposure.take_shutter()
    }

    pub fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) {
        self.exposure.apply_mosaic(mosaic);
    }

    pub fn step_polaroid(&mut self, dt: f32) {
        if self.exposure.step(dt) {
            use acoustics::{AcousticWave, ShutterClick};
            ShutterClick::mechanical().thump();
        }
    }

    #[cfg(test)]
    pub const fn polaroid_tilt(&self) -> f32 {
        self.exposure.polaroid_tilt()
    }

    pub fn emit_polaroid(&self, target: &mut dyn MeshTarget) {
        self.exposure.emit_polaroid(target);
    }

    pub fn calm(&mut self) {
        self.rhythm = StrideRhythm::idle();
    }

    pub fn reset(&mut self) {
        self.tilt_radians = 0.0;
        self.rhythm = StrideRhythm::idle();
        self.sight = SightMode::FirstPersonSight;
        self.exposure = CameraExposure::idle();
    }

    pub fn apply_rhythm(&mut self, rhythm: StrideRhythm) {
        self.rhythm = rhythm;
    }

    pub fn tilt_perish(&mut self, dt: f32) {
        self.rhythm = StrideRhythm::idle();
        self.tilt_radians += (0.22 - self.tilt_radians) * (4.0 * dt).clamp(0.0, 1.0);
    }

    pub fn decay_tilt(&mut self, dt: f32) {
        self.tilt_radians *= 1.0 - (8.0 * dt).min(1.0);
    }

    pub fn advance_stride(&mut self, prev_pos: Vec3, next_pos: Vec3, terrain_up: Vec3, dt: f32) {
        self.rhythm = self.rhythm.advance(prev_pos, next_pos, terrain_up, dt);
    }
}
