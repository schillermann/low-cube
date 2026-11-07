use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::attire::{LimbFrame, PhotoShutter, SightMode};
use crate::menu::PixelStamp;

const MOSAIC_COLS: usize = 20;
const MOSAIC_ROWS: usize = 15;
const POLAROID_DURATION_SEC: f32 = 2.8;

struct SquareScreenTarget<'a> {
    x_scale: f32,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> MeshTarget for SquareScreenTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let sx = self.x_scale;
        let scaled = quad.transfer(|v0, v1, v2, v3, norm, col| {
            let map = |v: Vec3| Vec3::new(v.x * sx, v.y, v.z);
            Quad::new(map(v0), map(v1), map(v2), map(v3), norm, col)
        });
        self.inner.push_quad(scaled);
    }
}

/// Captured OpenGL RGBA image plate capable of developing uncompressed PNG files and Polaroid mosaics.
pub struct PhotoPlate {
    width: u32,
    height: u32,
    bottom_up_rgba: Vec<u8>,
}

impl PhotoPlate {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(width: u32, height: u32, bottom_up_rgba: Vec<u8>) -> Self {
        Self { width, height, bottom_up_rgba }
    }

    fn crc32(&self, tag: &[u8; 4], payload: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &byte in tag.iter().chain(payload.iter()) {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
            }
        }
        !crc
    }

    fn append_chunk(&self, out: &mut Vec<u8>, tag: &[u8; 4], payload: &[u8]) {
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        out.extend_from_slice(tag);
        out.extend_from_slice(payload);
        out.extend_from_slice(&self.crc32(tag, payload).to_be_bytes());
    }

    /// Downsamples the bottom-up OpenGL RGBA buffer into a `MOSAIC_COLS x MOSAIC_ROWS` RGB tile grid.
    pub fn sample_mosaic(&self) -> Vec<Vec3> {
        let (w, h) = (self.width.max(1) as usize, self.height.max(1) as usize);
        let mut tiles = Vec::with_capacity(MOSAIC_COLS * MOSAIC_ROWS);
        for r in 0..MOSAIC_ROWS {
            let py = ((r * h) + (h / 2)) / MOSAIC_ROWS;
            for c in 0..MOSAIC_COLS {
                let px = ((c * w) + (w / 2)) / MOSAIC_COLS;
                let idx = (py.min(h - 1) * w + px.min(w - 1)) * 4;
                if idx + 2 < self.bottom_up_rgba.len() {
                    tiles.push(Vec3::new(
                        f32::from(self.bottom_up_rgba[idx]) / 255.0,
                        f32::from(self.bottom_up_rgba[idx + 1]) / 255.0,
                        f32::from(self.bottom_up_rgba[idx + 2]) / 255.0,
                    ));
                } else {
                    tiles.push(Vec3::new(0.35, 0.52, 0.72));
                }
            }
        }
        tiles
    }

    /// Encodes the bottom-up OpenGL RGBA buffer into a top-down standard PNG byte stream.
    pub fn encode_png(&self) -> Vec<u8> {
        let (w, h) = (self.width.max(1) as usize, self.height.max(1) as usize);
        let row_bytes = w * 4;
        assert!(
            self.bottom_up_rgba.len() >= row_bytes * h,
            "PhotoPlate RGBA buffer smaller than width * height * 4"
        );
        let mut raw_scanlines = Vec::with_capacity((row_bytes + 1) * h);
        for row in (0..h).rev() {
            raw_scanlines.push(0u8);
            let start = row * row_bytes;
            raw_scanlines.extend_from_slice(&self.bottom_up_rgba[start..start + row_bytes]);
        }
        let mut zlib_stream = Vec::with_capacity(raw_scanlines.len() + (raw_scanlines.len() / 65535 + 1) * 5 + 6);
        zlib_stream.extend_from_slice(&[0x78, 0x01]);
        let chunks: Vec<&[u8]> = raw_scanlines.chunks(65535).collect();
        let total_chunks = chunks.len().max(1);
        for (idx, slice) in chunks.into_iter().enumerate() {
            let last = u8::from(idx + 1 == total_chunks);
            let len = slice.len() as u16;
            zlib_stream.push(last);
            zlib_stream.extend_from_slice(&len.to_le_bytes());
            zlib_stream.extend_from_slice(&(!len).to_le_bytes());
            zlib_stream.extend_from_slice(slice);
        }
        let (mut s1, mut s2) = (1u32, 0u32);
        for &byte in &raw_scanlines {
            s1 = (s1 + u32::from(byte)) % 65521;
            s2 = (s2 + s1) % 65521;
        }
        zlib_stream.extend_from_slice(&((s2 << 16) | s1).to_be_bytes());
        let mut png = Vec::with_capacity(zlib_stream.len() + 64);
        png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        let mut ihdr = Vec::with_capacity(13);
        ihdr.extend_from_slice(&(w as u32).to_be_bytes());
        ihdr.extend_from_slice(&(h as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
        self.append_chunk(&mut png, b"IHDR", &ihdr);
        self.append_chunk(&mut png, b"IDAT", &zlib_stream);
        self.append_chunk(&mut png, b"IEND", &[]);
        png
    }

    /// Writes the encoded PNG to `folder/screenshot_<millis>_<angle_label>.png` and returns the file path.
    pub fn develop_into(&self, folder: &str, angle_label: &str) -> String {
        fs::create_dir_all(folder).expect("Failed to create screenshot directory");
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).expect("System clock before UNIX_EPOCH").as_millis();
        let path = format!("{folder}/screenshot_{millis}_{angle_label}.png");
        fs::write(&path, self.encode_png()).expect("Failed to write PNG screenshot to disk");
        path
    }

    /// Writes the encoded PNG into the `screenshots` directory.
    pub fn develop_to_disk(&self, angle_label: &str) -> String {
        self.develop_into("screenshots", angle_label)
    }
}

/// Instant Polaroid print displayed tilted at the center of the screen after taking a screenshot.
#[derive(Clone, Debug, PartialEq)]
pub struct PolaroidPrint {
    sight: SightMode,
    remaining_sec: f32,
    tilt_step: u8,
    mosaic: Vec<Vec3>,
}

impl PolaroidPrint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sight: SightMode, remaining_sec: f32, tilt_step: u8, mosaic: Vec<Vec3>) -> Self {
        Self { sight, remaining_sec, tilt_step, mosaic }
    }

    pub const fn hidden() -> Self {
        Self::new(SightMode::FirstPersonSight, 0.0, 0, Vec::new())
    }

    pub fn snapped(sight: SightMode, tilt_step: u8) -> Self {
        let mut preview = Vec::with_capacity(MOSAIC_COLS * MOSAIC_ROWS);
        for r in 0..MOSAIC_ROWS {
            for c in 0..MOSAIC_COLS {
                let portrait = !matches!(sight, SightMode::FirstPersonSight) && (7..13).contains(&c) && (2..12).contains(&r);
                let col = if portrait {
                    Vec3::new(0.78, 0.64, 0.48)
                } else if r < MOSAIC_ROWS / 2 {
                    Vec3::new(0.32, 0.56, 0.34)
                } else {
                    Vec3::new(0.42, 0.68, 0.92)
                };
                preview.push(col);
            }
        }
        Self::new(sight, POLAROID_DURATION_SEC, tilt_step, preview)
    }
    pub const fn visible(&self) -> bool { self.remaining_sec > 0.0 }
    pub const fn tilt_radians(&self) -> f32 {
        match self.tilt_step % 6 { 0 => 0.11, 1 => -0.09, 2 => 0.16, 3 => -0.14, 4 => 0.07, _ => -0.12 }
    }
    pub fn step(&mut self, dt: f32) { self.remaining_sec = (self.remaining_sec - dt).max(0.0); }
    pub fn apply_mosaic(&mut self, mosaic: Vec<Vec3>) {
        if mosaic.len() == MOSAIC_COLS * MOSAIC_ROWS {
            self.mosaic = mosaic;
        }
    }

    fn emit_rect(frame: &LimbFrame, min_x: f32, min_y: f32, max_x: f32, max_y: f32, color: Vec3, target: &mut dyn MeshTarget) {
        target.push_quad(Quad::new(frame.point(min_x, min_y, 0.0), frame.point(max_x, min_y, 0.0), frame.point(max_x, max_y, 0.0), frame.point(min_x, max_y, 0.0), Vec3::Z, color));
    }

    pub fn emit_centered(&self, target: &mut dyn MeshTarget) {
        if !self.visible() {
            return;
        }
        let (s, c) = self.tilt_radians().sin_cos();
        let frame = LimbFrame::new(Vec3::new(0.0, 0.02, 0.0), Vec3::new(c, s, 0.0), Vec3::new(-s, c, 0.0), Vec3::NEG_Z);
        let mut screen = SquareScreenTarget { x_scale: 9.0 / 16.0, inner: target };

        Self::emit_rect(&frame, -0.34, -0.45, 0.38, 0.39, Vec3::new(0.04, 0.05, 0.08), &mut screen);
        Self::emit_rect(&frame, -0.365, -0.435, 0.365, 0.405, Vec3::new(0.75, 0.73, 0.68), &mut screen);
        Self::emit_rect(&frame, -0.356, -0.426, 0.356, 0.396, Vec3::new(0.96, 0.95, 0.91), &mut screen);
        Self::emit_rect(&frame, -0.312, -0.172, 0.312, 0.356, Vec3::new(0.12, 0.14, 0.18), &mut screen);

        let (x0, y0, cell_w, cell_h) = (-0.304, -0.164, 0.608 / (MOSAIC_COLS as f32), 0.512 / (MOSAIC_ROWS as f32));
        for r in 0..MOSAIC_ROWS {
            for c_idx in 0..MOSAIC_COLS {
                let idx = r * MOSAIC_COLS + c_idx;
                let col = self.mosaic.get(idx).copied().unwrap_or(Vec3::new(0.38, 0.58, 0.78));
                let (mx0, my0) = (x0 + (c_idx as f32) * cell_w, y0 + (r as f32) * cell_h);
                Self::emit_rect(&frame, mx0, my0, mx0 + cell_w, my0 + cell_h, col, &mut screen);
            }
        }

        let file_line = format!("SAVED: SCREENSHOT {}.PNG", self.sight.photo_label().to_ascii_uppercase());
        PixelStamp::new(0.0052, 0.0074, Vec3::new(0.12, 0.15, 0.22))
            .emit_on_frame(&file_line, 0.0, -0.260, 0.0, &frame, &mut screen);
        PixelStamp::new(0.0044, 0.0062, Vec3::new(0.28, 0.32, 0.40))
            .emit_on_frame("FOLDER: SCREENSHOTS/ ON DISK", 0.0, -0.348, 0.0, &frame, &mut screen);
    }
}

/// Encapsulated camera shutter state, 6-second self-timer countdown, and instant Polaroid print overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraExposure {
    shutter: PhotoShutter,
    polaroid: PolaroidPrint,
    shot_counter: u8,
    countdown: (SightMode, f32),
}

impl CameraExposure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(shutter: PhotoShutter, polaroid: PolaroidPrint, shot_counter: u8, countdown: (SightMode, f32)) -> Self {
        Self { shutter, polaroid, shot_counter, countdown }
    }

    pub const fn idle() -> Self {
        Self::new(PhotoShutter::Idle, PolaroidPrint::hidden(), 0, (SightMode::FirstPersonSight, 0.0))
    }

    pub fn start_countdown(&mut self, sight: SightMode) {
        self.countdown = (sight, 6.0);
    }

    pub fn sync_countdown(&mut self, sight: SightMode) {
        if self.countdown.1 > 0.0 {
            self.countdown.0 = sight;
        }
    }

    pub fn trigger(&mut self, sight: SightMode) {
        let next_step = self.shot_counter;
        self.shot_counter = self.shot_counter.wrapping_add(1);
        self.shutter = PhotoShutter::Triggered(sight);
        self.polaroid = PolaroidPrint::snapped(sight, next_step);
    }

    pub fn take_shutter(&mut self) -> PhotoShutter {
        let pending = self.shutter;
        self.shutter = PhotoShutter::Idle;
        pending
    }

    pub fn apply_mosaic(&mut self, mosaic: Vec<Vec3>) {
        self.polaroid.apply_mosaic(mosaic);
    }

    pub fn step(&mut self, dt: f32) -> bool {
        self.polaroid.step(dt);
        if self.countdown.1 > 0.0 {
            self.countdown.1 = (self.countdown.1 - dt).max(0.0);
            if self.countdown.1 == 0.0 {
                self.trigger(self.countdown.0);
                return true;
            }
        }
        false
    }

    #[cfg(test)]
    pub const fn polaroid_tilt(&self) -> f32 {
        self.polaroid.tilt_radians()
    }

    pub fn emit_polaroid(&self, target: &mut dyn MeshTarget) {
        self.polaroid.emit_centered(target);
        if self.countdown.1 > 0.0 {
            let digit = (self.countdown.1.ceil() as u8).clamp(1, 6);
            let phase = (self.countdown.1 - f32::from(digit - 1)).clamp(0.0, 1.0);
            let cell_h = 0.016 + 0.044 * phase;
            let cell_w = cell_h * (9.0 / 16.0);
            let label = [b'0' + digit];
            let text = std::str::from_utf8(&label).unwrap_or("6");
            PixelStamp::new(cell_w, cell_h, Vec3::new(0.06, 0.08, 0.14)).emit_centered(text, 0.008, -0.010, target);
            PixelStamp::new(cell_w, cell_h, Vec3::new(0.99, 0.88, 0.36)).emit_centered(text, 0.0, 0.0, target);
        }
    }
}
