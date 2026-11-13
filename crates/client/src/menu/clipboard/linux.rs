use std::cell::RefCell;
use std::io::Write;
use std::process::{Command, Stdio};
use arboard::{GetExtLinux, LinuxClipboardKind, SetExtLinux};
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use winit::window::Window;
use super::{ScratchClipboard, TextClipboard};

/// Subprocess CLI clipboard bridge (`wl-copy` / `xclip` / `xsel`) decorating `ScratchClipboard`.
pub struct PipeClipboard {
    scratch: ScratchClipboard,
}

impl PipeClipboard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(scratch: ScratchClipboard) -> Self {
        Self { scratch }
    }

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
}

impl TextClipboard for PipeClipboard {
    fn store_text(&self, text: &str) -> bool {
        let _ = self.scratch.store_text(text);
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            self.write_pipe("wl-copy", &[], text)
                || self.write_pipe("xclip", &["-selection", "clipboard"], text)
                || self.write_pipe("xsel", &["--clipboard", "--input"], text)
        } else {
            self.write_pipe("xclip", &["-selection", "clipboard"], text)
                || self.write_pipe("xsel", &["--clipboard", "--input"], text)
                || self.write_pipe("wl-copy", &[], text)
        }
    }

    fn read_text(&self) -> String {
        let native = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            self.read_pipe("wl-paste", &["--no-newline"])
                .or_else(|| self.read_pipe("xclip", &["-selection", "clipboard", "-o"]))
                .or_else(|| self.read_pipe("xsel", &["--clipboard", "--output"]))
        } else {
            self.read_pipe("xclip", &["-selection", "clipboard", "-o"])
                .or_else(|| self.read_pipe("xsel", &["--clipboard", "--output"]))
                .or_else(|| self.read_pipe("wl-paste", &["--no-newline"]))
        };
        native.unwrap_or_else(|| self.scratch.read_text())
    }
}

/// Persistent in-process X11 and Wayland data-control selection bridge.
pub struct SelectionClipboard {
    board: RefCell<arboard::Clipboard>,
    scratch: ScratchClipboard,
    fallback: Box<dyn TextClipboard>,
}

impl SelectionClipboard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        board: RefCell<arboard::Clipboard>,
        scratch: ScratchClipboard,
        fallback: Box<dyn TextClipboard>,
    ) -> Self {
        Self { board, scratch, fallback }
    }
}

impl TextClipboard for SelectionClipboard {
    fn store_text(&self, text: &str) -> bool {
        let _ = self.scratch.store_text(text);
        if let Ok(mut board) = self.board.try_borrow_mut() {
            let clip_ok = board
                .set()
                .clipboard(LinuxClipboardKind::Clipboard)
                .text(text.to_owned())
                .is_ok();
            let _ = board
                .set()
                .clipboard(LinuxClipboardKind::Primary)
                .text(text.to_owned());
            if clip_ok {
                return true;
            }
        }
        self.fallback.store_text(text)
    }

    fn read_text(&self) -> String {
        if let Ok(mut board) = self.board.try_borrow_mut() {
            for kind in [LinuxClipboardKind::Clipboard, LinuxClipboardKind::Primary] {
                if let Ok(val) = board.get().clipboard(kind).text() {
                    let trimmed = val.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }
        self.fallback.read_text()
    }
}

/// Focused-seat Wayland `wl_data_device` clipboard bound directly to the window's `wl_display`.
pub struct WaylandClipboard {
    wayland: smithay_clipboard::Clipboard,
    scratch: ScratchClipboard,
    fallback: Box<dyn TextClipboard>,
}

impl WaylandClipboard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        wayland: smithay_clipboard::Clipboard,
        scratch: ScratchClipboard,
        fallback: Box<dyn TextClipboard>,
    ) -> Self {
        Self { wayland, scratch, fallback }
    }

    /// # Safety
    /// `display` must point to a valid `wl_display` outliving this clipboard instance.
    pub unsafe fn attach(
        display: core::ptr::NonNull<core::ffi::c_void>,
        scratch: ScratchClipboard,
        fallback: Box<dyn TextClipboard>,
    ) -> Self {
        let wayland = unsafe { smithay_clipboard::Clipboard::new(display.as_ptr()) };
        Self::new(wayland, scratch, fallback)
    }
}

impl TextClipboard for WaylandClipboard {
    fn store_text(&self, text: &str) -> bool {
        let _ = self.scratch.store_text(text);
        self.wayland.store(text.to_string());
        self.wayland.store_primary(text.to_string());
        true
    }

    fn read_text(&self) -> String {
        if let Ok(val) = self.wayland.load() {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Ok(val) = self.wayland.load_primary() {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        self.fallback.read_text()
    }
}

/// Linux display-aware clipboard assembly choosing focused Wayland seat vs X11 selection.
pub struct LinuxClipboardOrigin;

impl LinuxClipboardOrigin {
    /// Primary constructor initializing the Linux clipboard origin.
    pub const fn new() -> Self {
        Self
    }

    pub fn attach_window(&self, window: &Window) -> Box<dyn TextClipboard> {
        if let Ok(handle) = window.display_handle() {
            if let RawDisplayHandle::Wayland(wayland) = handle.as_raw() {
                let pipe = Box::new(PipeClipboard::new(ScratchClipboard::temp()));
                let attached = unsafe {
                    WaylandClipboard::attach(wayland.display, ScratchClipboard::temp(), pipe)
                };
                return Box::new(attached);
            }
        }
        self.detached()
    }

    pub fn detached(&self) -> Box<dyn TextClipboard> {
        let pipe: Box<dyn TextClipboard> = Box::new(PipeClipboard::new(ScratchClipboard::temp()));
        match arboard::Clipboard::new() {
            Ok(board) => Box::new(SelectionClipboard::new(
                RefCell::new(board),
                ScratchClipboard::temp(),
                pipe,
            )),
            Err(_) => pipe,
        }
    }
}
