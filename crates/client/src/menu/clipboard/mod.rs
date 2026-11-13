use std::path::PathBuf;
use winit::window::Window;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// Interface for storing and reading plain-text invite tickets and server addresses.
pub trait TextClipboard {
    fn store_text(&self, text: &str) -> bool;
    fn read_text(&self) -> String;
}

/// Local temporary-file clipboard backing in-game paste when no OS display clipboard exists.
pub struct ScratchClipboard {
    path: PathBuf,
}

impl ScratchClipboard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn temp() -> Self {
        Self::new(std::env::temp_dir().join("low-cube-portal-clipboard.txt"))
    }
}

impl TextClipboard for ScratchClipboard {
    fn store_text(&self, text: &str) -> bool {
        let _ = std::fs::write(&self.path, text);
        false
    }

    fn read_text(&self) -> String {
        std::fs::read_to_string(&self.path)
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
}

/// Operating-system clipboard factory assembling Wayland, X11, or Win32 selection bridges.
pub struct SystemClipboard;

impl SystemClipboard {
    /// Primary constructor initializing the platform clipboard factory.
    pub const fn new() -> Self {
        Self
    }

    #[cfg(target_os = "linux")]
    pub fn attach_window(&self, window: &Window) -> Box<dyn TextClipboard> {
        linux::LinuxClipboardOrigin::new().attach_window(window)
    }

    #[cfg(target_os = "windows")]
    pub fn attach_window(&self, _window: &Window) -> Box<dyn TextClipboard> {
        self.detached()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    pub fn attach_window(&self, _window: &Window) -> Box<dyn TextClipboard> {
        self.detached()
    }

    #[cfg(target_os = "linux")]
    pub fn detached(&self) -> Box<dyn TextClipboard> {
        linux::LinuxClipboardOrigin::new().detached()
    }

    #[cfg(target_os = "windows")]
    pub fn detached(&self) -> Box<dyn TextClipboard> {
        Box::new(windows::Win32Clipboard::new(ScratchClipboard::temp()))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    pub fn detached(&self) -> Box<dyn TextClipboard> {
        Box::new(ScratchClipboard::temp())
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
