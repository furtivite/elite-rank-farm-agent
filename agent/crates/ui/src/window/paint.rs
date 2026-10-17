use std::cell::RefCell;
use std::rc::Rc;

use native_windows_gui as nwg;

use super::brushes::Brushes;
use super::controls::Ui;
use super::palette::current_palette;
use crate::theme::{rgb, Palette};

pub(super) struct Theme {
    pub(super) palette: Rc<std::cell::Cell<Palette>>,
    pub(super) muted: usize,
    pub(super) separator: usize,
    pub(super) probe: usize,
    pub(super) watch: usize,
    pub(super) active: Rc<std::cell::Cell<Option<bool>>>,
}

pub(super) fn bind_theme(window: &nwg::Window, theme: Theme) -> Option<nwg::RawEventHandler> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::InvalidateRect;
    use windows_sys::Win32::Graphics::Gdi::{
        DrawTextW, FillRect, FrameRect, SetBkColor, SetBkMode, SetTextColor, DT_CENTER,
        DT_SINGLELINE, DT_VCENTER, TRANSPARENT,
    };
    use windows_sys::Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_FOCUS, ODS_SELECTED};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClientRect, GetWindowTextW, WM_DRAWITEM, WM_ERASEBKGND, WM_SYSCOLORCHANGE,
        WM_THEMECHANGED,
    };

    const THEME_HANDLER_ID: usize = 0x0001_4545;
    const WM_CTLCOLORSTATIC: u32 = 0x0138;
    const WM_CTLCOLOREDIT: u32 = 0x0133;

    let brushes = RefCell::new(Brushes::new(theme.palette.get()));
    let palette_cell = Rc::clone(&theme.palette);

    nwg::bind_raw_event_handler(
        &window.handle,
        THEME_HANDLER_ID,
        move |hwnd, msg, wparam, lparam| match msg {
            WM_SYSCOLORCHANGE | WM_THEMECHANGED => unsafe {
                let fresh = current_palette();
                palette_cell.set(fresh);
                brushes.borrow_mut().replace_with(fresh);
                InvalidateRect(hwnd as _, std::ptr::null(), 1);
                Some(0)
            },
            WM_ERASEBKGND => unsafe {
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetClientRect(hwnd as _, &mut rect);
                FillRect(wparam as _, &rect, brushes.borrow().window);
                Some(1)
            },
            WM_CTLCOLORSTATIC if lparam as usize == theme.separator => {
                Some(brushes.borrow().border as isize)
            }
            WM_CTLCOLORSTATIC => unsafe {
                let palette = palette_cell.get();
                SetBkMode(wparam as _, TRANSPARENT as i32);
                SetTextColor(
                    wparam as _,
                    if lparam as usize == theme.muted {
                        rgb(palette.muted)
                    } else {
                        rgb(palette.text)
                    },
                );
                Some(brushes.borrow().window as isize)
            },
            WM_CTLCOLOREDIT => unsafe {
                let palette = palette_cell.get();
                SetTextColor(wparam as _, rgb(palette.text));
                SetBkColor(wparam as _, rgb(palette.field));
                Some(brushes.borrow().field as isize)
            },
            WM_DRAWITEM => unsafe {
                let item = &*(lparam as *const DRAWITEMSTRUCT);
                let hdc = item.hDC;
                let mut rect = item.rcItem;

                let mode = match item.hwndItem as usize {
                    it if it == theme.probe => Some(true),
                    it if it == theme.watch => Some(false),
                    _ => None,
                };
                let on = mode.is_some() && theme.active.get() == mode;
                let held = item.itemState & ODS_SELECTED != 0;

                let palette = palette_cell.get();
                let held_text = if palette.button_pressed == palette.accent {
                    palette.accent_text
                } else {
                    palette.text
                };
                let current = brushes.borrow();
                let (fill, label) = match (on, held) {
                    (true, _) => (current.accent, rgb(palette.accent_text)),
                    (false, true) => (current.pressed, rgb(held_text)),
                    (false, false) => (current.button, rgb(palette.text)),
                };

                FillRect(hdc, &rect, fill);
                FrameRect(hdc, &rect, current.border);

                if item.itemState & ODS_FOCUS != 0 {
                    let ring = if on {
                        current.accent_text
                    } else {
                        current.focus
                    };
                    for inset in 1..3 {
                        let inner = RECT {
                            left: rect.left + inset,
                            top: rect.top + inset,
                            right: rect.right - inset,
                            bottom: rect.bottom - inset,
                        };
                        FrameRect(hdc, &inner, ring);
                    }
                }

                let mut caption = [0u16; 128];
                let length =
                    GetWindowTextW(item.hwndItem, caption.as_mut_ptr(), caption.len() as i32);
                if length > 0 {
                    SetBkMode(hdc, TRANSPARENT as i32);
                    SetTextColor(hdc, label);
                    DrawTextW(
                        hdc,
                        caption.as_ptr(),
                        length,
                        &mut rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                }

                Some(1)
            },
            _ => None,
        },
    )
    .ok()
}

pub(super) fn plain_label(label: &nwg::Label) -> Option<nwg::RawEventHandler> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER,
        WM_NCCALCSIZE, WM_NCPAINT,
    };

    const PLAIN_LABEL_ID: usize = 0x0001_4547;

    let handler =
        nwg::bind_raw_event_handler(&label.handle, PLAIN_LABEL_ID, |_, msg, _, _| match msg {
            WM_NCCALCSIZE | WM_NCPAINT => Some(0),
            _ => None,
        })
        .ok()?;

    if let Some(hwnd) = label.handle.hwnd() {
        unsafe {
            SetWindowPos(
                hwnd as _,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
            );
        }
    }
    Some(handler)
}

pub(super) fn make_owner_drawn(handle: nwg::ControlHandle) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, BS_OWNERDRAW, GWL_STYLE,
    };

    const BUTTON_TYPE_MASK: isize = 0x0F;

    let Some(hwnd) = handle.hwnd() else {
        return;
    };

    unsafe {
        let style = GetWindowLongPtrW(hwnd as _, GWL_STYLE);
        SetWindowLongPtrW(
            hwnd as _,
            GWL_STYLE,
            (style & !BUTTON_TYPE_MASK) | BS_OWNERDRAW as isize,
        );
    }
}

pub(super) fn hwnd_of(handle: nwg::ControlHandle) -> usize {
    handle.hwnd().map_or(0, |hwnd| hwnd as usize)
}

pub(super) fn repaint_modes(ui: &Ui) {
    use windows_sys::Win32::Graphics::Gdi::InvalidateRect;

    for handle in [ui.probe_button.handle, ui.watch_button.handle] {
        if let Some(hwnd) = handle.hwnd() {
            unsafe {
                InvalidateRect(hwnd as _, std::ptr::null(), 1);
            }
        }
    }
}
