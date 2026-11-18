//! Experimental Windows-only docking of a user-started RetroArch game HWND.
//! This does not provide an in-process graphics context. Win32 owns the child
//! window and RetroArch continues to render using its selected Vulkan/GL core.
//! Window handles must never be guessed or supplied by JavaScript.
use std::mem::MaybeUninit;

type Hwnd = isize;
type Bool = i32;
const GWL_STYLE: i32 = -16;
const GW_OWNER: u32 = 4;
const WS_CHILD: isize = 0x4000_0000;
const WS_POPUP: isize = 0x8000_0000u32 as isize;
const WS_CAPTION: isize = 0x00c0_0000;
const WS_THICKFRAME: isize = 0x0004_0000;
const WS_SYSMENU: isize = 0x0008_0000;
const WS_MINIMIZEBOX: isize = 0x0002_0000;
const WS_MAXIMIZEBOX: isize = 0x0001_0000;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_SHOWWINDOW: u32 = 0x0040;
const HWND_TOP: Hwnd = 0;
const HWND_NOTOPMOST: Hwnd = -2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[link(name = "user32")]
extern "system" {
    fn EnumWindows(callback: Option<unsafe extern "system" fn(Hwnd, isize) -> Bool>, data: isize) -> Bool;
    fn GetWindowThreadProcessId(hwnd: Hwnd, pid: *mut u32) -> u32;
    fn IsWindowVisible(hwnd: Hwnd) -> Bool;
    fn IsWindow(hwnd: Hwnd) -> Bool;
    fn GetWindow(hwnd: Hwnd, command: u32) -> Hwnd;
    fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    fn GetParent(hwnd: Hwnd) -> Hwnd;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn SetParent(hwnd: Hwnd, parent: Hwnd) -> Hwnd;
    fn PostMessageW(hwnd: Hwnd, message: u32, wparam: usize, lparam: isize) -> Bool;
    fn SetWindowPos(hwnd: Hwnd, after: Hwnd, x: i32, y: i32, width: i32, height: i32, flags: u32) -> Bool;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DockRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl DockRect {
    pub fn validate(self) -> Result<Self, String> {
        if self.width < 64 || self.height < 64 || self.width > 16384 || self.height > 16384
            || self.x < -16384 || self.x > 16384 || self.y < -16384 || self.y > 16384
        {
            return Err("Invalid dock rectangle; try scrolling the PhiCade CRT into view".into());
        }
        Ok(self)
    }
}

/// The original window styles and geometry are saved, so Undock is reversible.
/// Windows can reset cross-process DPI awareness; treat this as experimental.
pub(crate) struct DockedWindow {
    hwnd: Hwnd,
    original_style: isize,
    original_parent: Hwnd,
    original_rect: Rect,
}

struct WindowCandidate {
    pid: u32,
    hwnd: Hwnd,
    area: i64,
}

unsafe extern "system" fn find_callback(hwnd: Hwnd, data: isize) -> Bool {
    let selection = &mut *(data as *mut WindowCandidate);
    if IsWindowVisible(hwnd) == 0 || GetWindow(hwnd, GW_OWNER) != 0 {
        return 1;
    }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != selection.pid {
        return 1;
    }
    let mut rect = MaybeUninit::<Rect>::uninit();
    if GetWindowRect(hwnd, rect.as_mut_ptr()) == 0 {
        return 1;
    }
    let rect = rect.assume_init();
    let width = i64::from(rect.right - rect.left);
    let height = i64::from(rect.bottom - rect.top);
    let area = width.saturating_mul(height);
    if width > 200 && height > 100 && area > selection.area {
        selection.hwnd = hwnd;
        selection.area = area;
    }
    1
}

fn find_process_window(pid: u32) -> Option<Hwnd> {
    let mut selection = WindowCandidate { pid, hwnd: 0, area: 0 };
    // The enum callback reads only windows owned by the child PID we launched.
    unsafe {
        EnumWindows(Some(find_callback), (&mut selection as *mut WindowCandidate) as isize);
    }
    (selection.hwnd != 0).then_some(selection.hwnd)
}

/// Ask RetroArch to close itself normally so save data has a chance to flush.
pub(crate) fn request_graceful_close(pid: u32) -> Result<(), String> {
    let hwnd = find_process_window(pid)
        .ok_or_else(|| "RetroArch game window not found; close it manually".to_owned())?;
    // WM_CLOSE is the normal Win32 shutdown request, unlike Child::kill.
    if unsafe { PostMessageW(hwnd, 0x0010, 0, 0) } == 0 {
        return Err("Could not request RetroArch close; close its window manually".into());
    }
    Ok(())
}

impl DockedWindow {
    pub fn attach(parent: Hwnd, pid: u32, rect: DockRect) -> Result<Self, String> {
        rect.validate()?;
        if parent == 0 {
            return Err("PhiCade native window handle unavailable".into());
        }
        let hwnd = find_process_window(pid)
            .ok_or_else(|| "N64_WINDOW_NOT_READY: RetroArch's game window has not appeared yet".to_owned())?;
        unsafe {
            let mut original_rect = Rect::default();
            if GetWindowRect(hwnd, &mut original_rect) == 0 {
                return Err("Could not read RetroArch window geometry".into());
            }
            let original_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let original_parent = GetParent(hwnd);
            // Windows SetParent does not update WS_CHILD/WS_POPUP automatically.
            let child_style = (original_style | WS_CHILD)
                & !(WS_POPUP | WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX);
            SetWindowLongPtrW(hwnd, GWL_STYLE, child_style);
            SetParent(hwnd, parent);
            if GetParent(hwnd) != parent {
                SetWindowLongPtrW(hwnd, GWL_STYLE, original_style);
                return Err("Windows refused to dock the RetroArch window (possibly DPI/integrity mismatch)".into());
            }
            let dock = Self { hwnd, original_style, original_parent, original_rect };
            if let Err(error) = dock.resize(rect) {
                dock.detach();
                return Err(error);
            }
            Ok(dock)
        }
    }

    pub fn is_alive(&self) -> bool {
        unsafe { IsWindow(self.hwnd) != 0 }
    }

    pub fn resize(&self, rect: DockRect) -> Result<(), String> {
        rect.validate()?;
        if !self.is_alive() {
            return Err("RetroArch game window has closed".into());
        }
        let success = unsafe {
            SetWindowPos(self.hwnd, HWND_TOP, rect.x, rect.y, rect.width, rect.height,
                         SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW)
        };
        if success == 0 {
            return Err("Could not resize the docked RetroArch window".into());
        }
        Ok(())
    }

    pub fn detach(&self) {
        unsafe {
            if IsWindow(self.hwnd) == 0 {
                return;
            }
            // Restore to the desktop (or its former parent) *before* top-level style.
            SetParent(self.hwnd, self.original_parent);
            SetWindowLongPtrW(self.hwnd, GWL_STYLE, self.original_style);
            let width = (self.original_rect.right - self.original_rect.left).max(1);
            let height = (self.original_rect.bottom - self.original_rect.top).max(1);
            let _ = SetWindowPos(
                self.hwnd, HWND_NOTOPMOST, self.original_rect.left, self.original_rect.top,
                width, height, SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_reject_invalid_coordinates_and_sizes() {
        assert!(DockRect { x: 20, y: 30, width: 640, height: 480 }.validate().is_ok());
        assert!(DockRect { x: 0, y: 0, width: 0, height: 480 }.validate().is_err());
        assert!(DockRect { x: 0, y: 0, width: 64000, height: 480 }.validate().is_err());
    }
}
