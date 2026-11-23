use core::ffi::c_void;
use core::mem::size_of;
use std::time::Duration;
use super::CHUNK_SAMPLES;

const WAVE_MAPPER: u32 = 0xFFFF_FFFF;
const WHDR_DONE: u32 = 0x0000_0001;
const RING_SLOTS: usize = 6;

#[repr(C, packed(2))]
struct PcmWaveFormat {
    tag_channels: [u16; 2],
    rates: [u32; 2],
    frame_spec: [u16; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WaveControl {
    user_tag: usize,
    flags_loops: [u32; 2],
}

#[repr(C)]
struct WinWaveHeader {
    data_ptr: *mut u8,
    byte_spans: [u32; 2],
    control: WaveControl,
    driver_links: [usize; 2],
}

unsafe impl Send for WinWaveHeader {}

#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(period_ms: u32) -> u32;
    fn waveOutOpen(
        out_handle: *mut *mut c_void,
        device_id: u32,
        format: *const PcmWaveFormat,
        callback: usize,
        instance: usize,
        flags: u32,
    ) -> u32;
    fn waveOutPrepareHeader(handle: *mut c_void, header: *mut WinWaveHeader, size: u32) -> u32;
    fn waveOutWrite(handle: *mut c_void, header: *mut WinWaveHeader, size: u32) -> u32;
}

pub(super) struct PlatformSink {
    handle: usize,
    buffers: Box<[[i16; CHUNK_SAMPLES]; RING_SLOTS]>,
    headers: Box<[WinWaveHeader; RING_SLOTS]>,
    cursor: usize,
}

impl PlatformSink {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(
        handle: usize,
        buffers: Box<[[i16; CHUNK_SAMPLES]; RING_SLOTS]>,
        headers: Box<[WinWaveHeader; RING_SLOTS]>,
        cursor: usize,
    ) -> Self {
        Self { handle, buffers, headers, cursor }
    }

    pub(super) fn open() -> Self {
        let fmt = PcmWaveFormat {
            tag_channels: [1, 2],
            rates: [24000, 96000],
            frame_spec: [4, 16, 0],
        };
        unsafe {
            let _ = timeBeginPeriod(1);
        }
        let mut raw_handle: *mut c_void = core::ptr::null_mut();
        let rc = unsafe { waveOutOpen(&mut raw_handle, WAVE_MAPPER, &fmt, 0, 0, 0) };
        let mut buffers = Box::new([[0i16; CHUNK_SAMPLES]; RING_SLOTS]);
        let zero_ctl = WaveControl { user_tag: 0, flags_loops: [0, 0] };
        let blank_hdr = || WinWaveHeader {
            data_ptr: core::ptr::null_mut(),
            byte_spans: [0, 0],
            control: zero_ctl,
            driver_links: [0, 0],
        };
        let mut headers = Box::new([
            blank_hdr(),
            blank_hdr(),
            blank_hdr(),
            blank_hdr(),
            blank_hdr(),
            blank_hdr(),
        ]);
        if rc == 0 && !raw_handle.is_null() {
            let hdr_bytes = size_of::<WinWaveHeader>() as u32;
            let pcm_bytes = (CHUNK_SAMPLES * 2) as u32;
            for slot in 0..RING_SLOTS {
                headers[slot].data_ptr = buffers[slot].as_mut_ptr().cast::<u8>();
                headers[slot].byte_spans = [pcm_bytes, 0];
                unsafe {
                    let _ = waveOutPrepareHeader(raw_handle, &mut headers[slot], hdr_bytes);
                }
                headers[slot].control.flags_loops[0] |= WHDR_DONE;
            }
            Self::new(raw_handle as usize, buffers, headers, 0)
        } else {
            Self::new(0, buffers, headers, 0)
        }
    }

    pub(super) fn write_chunk(&mut self, samples: &[i16; CHUNK_SAMPLES]) -> bool {
        if self.handle == 0 {
            return false;
        }
        let slot = self.cursor % RING_SLOTS;
        for _ in 0..28 {
            let flags = unsafe { core::ptr::read_volatile(&self.headers[slot].control.flags_loops[0]) };
            if (flags & WHDR_DONE) != 0 {
                self.buffers[slot].copy_from_slice(samples);
                self.headers[slot].control.flags_loops[0] = flags & !WHDR_DONE;
                let h = self.handle as *mut c_void;
                let hdr_bytes = size_of::<WinWaveHeader>() as u32;
                unsafe {
                    let _ = waveOutWrite(h, &mut self.headers[slot], hdr_bytes);
                }
                self.cursor = self.cursor.wrapping_add(1);
                return true;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        true
    }
}
