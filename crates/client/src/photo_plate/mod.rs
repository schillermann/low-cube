mod exposure;

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use glam::Vec3;
pub use self::exposure::CameraExposure;

pub(super) const MOSAIC_COLS: usize = 20;
pub(super) const MOSAIC_ROWS: usize = 15;

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
