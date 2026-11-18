//! Windows-only N64 docking by placing a *top-level owned popup* over the CRT.
//!
//! The former SetParent/WS_CHILD approach hid the Vulkan game behind WebView2.
//! Keeping RetroArch's real top-level HWND preserves its GPU presentation path;
//! the owner relationship keeps it above PhiCade without being system-topmost.
//! Undock always restores original owner, frame style and screen geometry.
use std::mem::MaybeUninit;

type Hwnd = isize;
type Bool = i32;
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const GWLP_HWNDPARENT: i32 = -8; // owner of a top-level HWND, not child parent
const GW_OWNER: u32 = 4;
const WS_POPUP: isize = 0x8000_0000u32 as isize;
const WS_CHILD: isize = 0x4000_0000;
const WS_CAPTION: isize = 0x00c0_0000;
const WS_THICKFRAME: isize = 0x0004_0000;
const WS_SYSMENU: isize = 0x0008_0000;
const WS_MINIMIZEBOX: isize = 0x0002_0000;
const WS_MAXIMIZEBOX: isize = 0x0001_0000;
const WS_EX_TOPMOST: isize = 0x0000_0008;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_SHOWWINDOW: u32 = 0x0040;
const HWND_TOP: Hwnd = 0;
const HWND_TOPMOST: Hwnd = -1;
const HWND_NOTOPMOST: Hwnd = -2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[link(name = "user32")]
extern "system" {
    fn EnumWindows(callback: Option<unsafe extern "system" fn(Hwnd, isize) -> Bool>, data: isize) -> Bool;
    fn GetWindowThreadProcessId(hwnd: Hwnd, pid: *mut u32) -> u32;
    fn IsWindowVisible(hwnd: Hwnd) -> Bool;
    fn IsWindow(hwnd: Hwnd) -> Bool;
    fn GetWindow(hwnd: Hwnd, command: u32) -> Hwnd;
    fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    fn ClientToScreen(hwnd: Hwnd, point: *mut Point) -> Bool;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
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
            return Err("Invalid dock rectangle; scroll the PhiCade CRT into view".into());
        }
        Ok(self)
    }
}

pub(crate) struct DockedWindow {
    hwnd: Hwnd,
    owner_hwnd: Hwnd,
    original_style: isize,
    original_owner: Hwnd,
    original_rect: Rect,
    original_topmost: bool,
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

// While docked, the owner may be set, so EnumWindows still enumerates RetroArch
// but this callback won't match. Graceful close must use the stored HWND instead.
fn find_process_window(pid: u32) -> Option<Hwnd> {
    let mut selection = WindowCandidate { pid, hwnd: 0, area: 0 };
    unsafe { EnumWindows(Some(find_callback), (&mut selection as *mut WindowCandidate) as isize); }
    (selection.hwnd != 0).then_some(selection.hwnd)
}

pub(crate) fn request_graceful_close(pid: u32) -> Result<(), String> {
    let hwnd = find_process_window(pid)
        .ok_or_else(|| "RetroArch game window not found; close it manually".to_owned())?;
    request_close_window(hwnd)
}

pub(crate) fn request_close_window(hwnd: Hwnd) -> Result<(), String> {
    if unsafe { IsWindow(hwnd) } == 0 {
        return Err("RetroArch game window has already closed".into());
    }
    if unsafe { PostMessageW(hwnd, 0x0010, 0, 0) } == 0 {
        return Err("Could not request RetroArch close; close its window manually".into());
    }
    Ok(())
}

fn screen_rect(parent: Hwnd, rect: DockRect) -> Result<Rect, String> {
    rect.validate()?;
    let mut point = Point { x: rect.x, y: rect.y };
    if unsafe { ClientToScreen(parent, &mut point) } == 0 {
        return Err("Cannot map PhiCade CRT to Windows desktop coordinates".into());
    }
    Ok(Rect {
        left: point.x,
        top: point.y,
        right: point.x.saturating_add(rect.width),
        bottom: point.y.saturating_add(rect.height),
    })
}

impl DockedWindow {
    pub fn attach(owner: Hwnd, pid: u32, rect: DockRect) -> Result<Self, String> {
        rect.validate()?;
        if owner == 0 {
            return Err("PhiCade native window handle unavailable".into());
        }
        let hwnd = find_process_window(pid).ok_or_else(|| {
            "N64_WINDOW_NOT_READY: RetroArch's game window has not appeared yet".to_owned()
        })?;
        unsafe {
            let mut original_rect = Rect::default();
            if GetWindowRect(hwnd, &mut original_rect) == 0 {
                return Err("Cannot read RetroArch window geometry".into());
            }
            let original_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let original_owner = GetWindow(hwnd, GW_OWNER);
            let original_topmost = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_TOPMOST != 0;
            // Keep the Vulkan surface a top-level HWND. The former WS_CHILD /
            // SetParent route was occluded by WebView2 and sometimes lost GPU frames.
            let popup_style = (original_style | WS_POPUP)
                & !(WS_CHILD | WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX);
            SetWindowLongPtrW(hwnd, GWL_STYLE, popup_style);
            SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner);
            if GetWindow(hwnd, GW_OWNER) != owner {
                SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, original_owner);
                SetWindowLongPtrW(hwnd, GWL_STYLE, original_style);
                return Err("Windows refused to make RetroArch an owned overlay. Game remains external".into());
            }
            let dock = Self { hwnd, owner_hwnd: owner, original_style, original_owner, original_rect, original_topmost };
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

    pub fn hwnd(&self) -> Hwnd { self.hwnd }

    pub fn resize(&self, rect: DockRect) -> Result<(), String> {
        if !self.is_alive() {
            return Err("RetroArch game window has closed".into());
        }
        let pos = screen_rect(self.owner_hwnd, rect)?;
        if unsafe {
            SetWindowPos(self.hwnd, HWND_TOP, pos.left, pos.top, rect.width, rect.height,
                SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW)
        } == 0 {
            return Err("Cannot position RetroArch over PhiCade's CRT".into());
        }
        // Validate the actual position instead of claiming DOCKED on an API call
        // that succeeds while the window is still offscreen or unchanged.
        let mut actual = Rect::default();
        if unsafe { GetWindowRect(self.hwnd, &mut actual) } == 0 {
            return Err("Cannot verify RetroArch overlay position".into());
        }
        let d = |a: i32, b: i32| (i64::from(a) - i64::from(b)).abs();
        if d(actual.left, pos.left) > 12 || d(actual.top, pos.top) > 12
            || d(actual.right - actual.left, rect.width) > 12
            || d(actual.bottom - actual.top, rect.height) > 12
        {
            return Err(format!(
                "RetroArch overlay geometry mismatch: expected {}x{} at {},{}; observed {}x{} at {},{}",
                rect.width, rect.height, pos.left, pos.top,
                actual.right - actual.left, actual.bottom - actual.top,
                actual.left, actual.top
            ));
        }
        Ok(())
    }

    pub fn detach(&self) {
        unsafe {
            if IsWindow(self.hwnd) == 0 { return; }
            // Restore original owner, style, z-order, and original desktop
            // geometry. No game reload or loss of save state.
            SetWindowLongPtrW(self.hwnd, GWLP_HWNDPARENT, self.original_owner);
            SetWindowLongPtrW(self.hwnd, GWL_STYLE, self.original_style);
            let width = (self.original_rect.right - self.original_rect.left).max(1);
            let height = (self.original_rect.bottom - self.original_rect.top).max(1);
            let _ = SetWindowPos(self.hwnd,
                if self.original_topmost { HWND_TOPMOST } else { HWND_NOTOPMOST },
                self.original_rect.left, self.original_rect.top, width, height,
                SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW);
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
    #[test]
    fn overlay_remains_top_level_not_child() {
        let original = WS_CAPTION | WS_THICKFRAME | WS_SYSMENU;
        let popup = (original | WS_POPUP)
            & !(WS_CHILD | WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX);
        assert_ne!(popup & WS_POPUP, 0);
        assert_eq!(popup & WS_CHILD, 0);
    }
}
