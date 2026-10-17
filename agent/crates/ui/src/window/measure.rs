use native_windows_gui as nwg;
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{
    DrawTextW, GetDC, ReleaseDC, SelectObject, DT_CALCRECT, DT_NOPREFIX, DT_SINGLELINE,
    DT_WORDBREAK, HGDIOBJ,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_GETFONT};

use super::dpi::{logical, scaled};

pub(super) fn wrapped_height(handle: nwg::ControlHandle, text: &str, width: i32) -> i32 {
    let rect = measure(handle, text, scaled(width), DT_WORDBREAK);
    logical((rect.bottom - rect.top).max(0) as u32) as i32
}

pub(super) fn line_width(handle: nwg::ControlHandle, text: &str) -> i32 {
    let rect = measure(handle, text, 0, DT_SINGLELINE);
    logical((rect.right - rect.left).max(0) as u32) as i32
}

fn measure(handle: nwg::ControlHandle, text: &str, width: i32, flags: u32) -> RECT {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: width,
        bottom: 0,
    };
    let Some(hwnd) = handle.hwnd() else {
        return rect;
    };
    let wide: Vec<u16> = text.encode_utf16().collect();

    unsafe {
        let hdc = GetDC(hwnd as _);
        if hdc.is_null() {
            return rect;
        }
        let font = SendMessageW(hwnd as _, WM_GETFONT, 0, 0) as HGDIOBJ;
        let previous = if font.is_null() {
            std::ptr::null_mut()
        } else {
            SelectObject(hdc, font)
        };

        DrawTextW(
            hdc,
            wide.as_ptr(),
            wide.len() as i32,
            &mut rect,
            DT_CALCRECT | DT_NOPREFIX | flags,
        );

        if !previous.is_null() {
            SelectObject(hdc, previous);
        }
        ReleaseDC(hwnd as _, hdc);
    }
    rect
}
