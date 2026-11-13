#[cfg(target_os = "linux")]
use std::io::Write;
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

/// Interface for storing and reading plain-text invite tickets and server addresses.
pub trait TextClipboard {
    fn store_text(&self, text: &str) -> bool;
    fn read_text(&self) -> String;
}

/// Operating-system clipboard backed by Win32 / Wayland / X11 with local scratch fallback.
pub struct SystemClipboard;

impl SystemClipboard {
    /// Primary constructor initializing the platform clipboard bridge.
    pub const fn new() -> Self {
        Self
    }

    fn scratch_path(&self) -> PathBuf {
        std::env::temp_dir().join("low-cube-portal-clipboard.txt")
    }

    #[cfg(target_os = "linux")]
    fn write_pipe(&self, program: &str, args: &[&str], text: &str) -> bool {
        let Ok(mut child) = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return false;
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        child.wait().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    fn read_pipe(&self, program: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    }

    #[cfg(target_os = "linux")]
    fn store_native(&self, text: &str) -> bool {
        self.write_pipe("wl-copy", &[], text)
            || self.write_pipe("xclip", &["-selection", "clipboard"], text)
            || self.write_pipe("xsel", &["--clipboard", "--input"], text)
    }

    #[cfg(target_os = "linux")]
    fn read_native(&self) -> Option<String> {
        self.read_pipe("wl-paste", &["--no-newline"])
            .or_else(|| self.read_pipe("xclip", &["-selection", "clipboard", "-o"]))
            .or_else(|| self.read_pipe("xsel", &["--clipboard", "--output"]))
    }

    #[cfg(target_os = "windows")]
    fn store_native(&self, text: &str) -> bool {
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

    #[cfg(target_os = "windows")]
    fn read_native(&self) -> Option<String> {
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

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn store_native(&self, _text: &str) -> bool {
        false
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn read_native(&self) -> Option<String> {
        None
    }
}

impl TextClipboard for SystemClipboard {
    fn store_text(&self, text: &str) -> bool {
        let file_ok = std::fs::write(self.scratch_path(), text).is_ok();
        let os_ok = self.store_native(text);
        os_ok || file_ok
    }

    fn read_text(&self) -> String {
        self.read_native()
            .or_else(|| std::fs::read_to_string(self.scratch_path()).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
pub struct MemoryClipboard {
    content: std::cell::RefCell<String>,
}

#[cfg(test)]
impl MemoryClipboard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(content: std::cell::RefCell<String>) -> Self {
        Self { content }
    }

    pub fn seeded(initial: &str) -> Self {
        Self::new(std::cell::RefCell::new(initial.to_string()))
    }
}

#[cfg(test)]
impl TextClipboard for MemoryClipboard {
    fn store_text(&self, text: &str) -> bool {
        *self.content.borrow_mut() = text.to_string();
        true
    }

    fn read_text(&self) -> String {
        self.content.borrow().clone()
    }
}
