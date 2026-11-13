use super::{ScratchClipboard, TextClipboard};

/// Native Win32 `CF_UNICODETEXT` system clipboard decorating `ScratchClipboard`.
pub struct Win32Clipboard {
    scratch: ScratchClipboard,
}

impl Win32Clipboard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(scratch: ScratchClipboard) -> Self {
        Self { scratch }
    }

    fn store_win32(&self, text: &str) -> bool {
        const CF_UNICODETEXT: u32 = 13;
        const GMEM_MOVEABLE: u32 = 0x0002;
        #[link(name = "user32")]
        extern "system" {
            fn OpenClipboard(hwnd: *mut core::ffi::c_void) -> i32;
            fn EmptyClipboard() -> i32;
            fn SetClipboardData(format: u32, mem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
            fn CloseClipboard() -> i32;
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalAlloc(flags: u32, bytes: usize) -> *mut core::ffi::c_void;
            fn GlobalLock(mem: *mut core::ffi::c_void) -> *mut u16;
            fn GlobalUnlock(mem: *mut core::ffi::c_void) -> i32;
        }
        let mut utf16: Vec<u16> = text.encode_utf16().collect();
        utf16.push(0);
        unsafe {
            if OpenClipboard(core::ptr::null_mut()) == 0 {
                return false;
            }
            let _ = EmptyClipboard();
            let handle = GlobalAlloc(GMEM_MOVEABLE, utf16.len() * 2);
            if handle.is_null() {
                let _ = CloseClipboard();
                return false;
            }
            let ptr = GlobalLock(handle);
            if !ptr.is_null() {
                core::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                let _ = GlobalUnlock(handle);
            }
            let ok = !SetClipboardData(CF_UNICODETEXT, handle).is_null();
            let _ = CloseClipboard();
            ok
        }
    }

    fn read_win32(&self) -> Option<String> {
        const CF_UNICODETEXT: u32 = 13;
        #[link(name = "user32")]
        extern "system" {
            fn OpenClipboard(hwnd: *mut core::ffi::c_void) -> i32;
            fn GetClipboardData(format: u32) -> *mut core::ffi::c_void;
            fn CloseClipboard() -> i32;
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalLock(mem: *mut core::ffi::c_void) -> *const u16;
            fn GlobalUnlock(mem: *mut core::ffi::c_void) -> i32;
        }
        unsafe {
            if OpenClipboard(core::ptr::null_mut()) == 0 {
                return None;
            }
            let handle = GetClipboardData(CF_UNICODETEXT);
            if handle.is_null() {
                let _ = CloseClipboard();
                return None;
            }
            let ptr = GlobalLock(handle);
            let mut units = Vec::new();
            if !ptr.is_null() {
                let mut len = 0usize;
                while len < 4096 && *ptr.add(len) != 0 {
                    units.push(*ptr.add(len));
                    len += 1;
                }
                let _ = GlobalUnlock(handle);
            }
            let _ = CloseClipboard();
            let decoded = String::from_utf16_lossy(&units).trim().to_string();
            if decoded.is_empty() { None } else { Some(decoded) }
        }
    }
}

impl TextClipboard for Win32Clipboard {
    fn store_text(&self, text: &str) -> bool {
        let _ = self.scratch.store_text(text);
        self.store_win32(text)
    }

    fn read_text(&self) -> String {
        self.read_win32().unwrap_or_else(|| self.scratch.read_text())
    }
}
