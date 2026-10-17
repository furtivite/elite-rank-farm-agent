use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use native_windows_gui as nwg;

use super::controls::Ui;
use super::layout::fit_height;

pub(super) const UI_FONT_HEIGHT: u32 = 16;
pub(super) const OUTPUT_FONT_HEIGHT: u32 = 16;

const BASE_DPI: u32 = 96;

static UI_DPI: AtomicU32 = AtomicU32::new(BASE_DPI);

pub(super) fn read_from_system() {
    UI_DPI.store(system_dpi(), Ordering::Relaxed);
}

fn dpi() -> u32 {
    UI_DPI.load(Ordering::Relaxed)
}

pub(super) fn scaled(value: i32) -> i32 {
    value * dpi() as i32 / BASE_DPI as i32
}

pub(super) fn logical(value: u32) -> u32 {
    value * BASE_DPI / dpi()
}

pub(super) fn move_control(handle: nwg::ControlHandle, x: i32, y: i32, width: u32, height: u32) {
    if let Some(hwnd) = handle.hwnd() {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::MoveWindow(
                hwnd as _,
                scaled(x),
                scaled(y),
                scaled(width as i32),
                scaled(height as i32),
                1,
            );
        }
    }
}

fn system_dpi() -> u32 {
    let value = unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForSystem() };

    if (BASE_DPI..=BASE_DPI * 4).contains(&value) {
        value
    } else {
        BASE_DPI
    }
}

pub(super) fn bind_dpi_change(ui: &Rc<Ui>) -> Option<nwg::RawEventHandler> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::DeleteObject;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

    const DPI_HANDLER_ID: usize = 0x0001_4546;
    const WM_DPICHANGED: u32 = 0x02E0;

    let handle = ui.window.handle;
    let ui = Rc::clone(ui);
    let fonts: RefCell<Option<(nwg::Font, nwg::Font)>> = RefCell::new(None);

    nwg::bind_raw_event_handler(&handle, DPI_HANDLER_ID, move |hwnd, msg, wparam, lparam| {
        if msg != WM_DPICHANGED {
            return None;
        }

        let fresh = (wparam & 0xFFFF) as u32;
        if !(BASE_DPI..=BASE_DPI * 4).contains(&fresh) {
            return Some(0);
        }
        UI_DPI.store(fresh, Ordering::Relaxed);

        let (next_ui_font, next_output_font) = match new_fonts() {
            Some(pair) => pair,
            None => return Some(0),
        };
        apply_fonts(&ui, &next_ui_font, &next_output_font);

        let previous = fonts.borrow_mut().replace((next_ui_font, next_output_font));
        if let Some((old_ui, old_output)) = previous {
            unsafe {
                DeleteObject(old_ui.handle as _);
                DeleteObject(old_output.handle as _);
            }
        }

        unsafe {
            let suggested = lparam as *const RECT;
            if !suggested.is_null() {
                let rect = &*suggested;
                SetWindowPos(
                    hwnd as _,
                    std::ptr::null_mut(),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }

        fit_height(&ui);
        Some(0)
    })
    .ok()
}

fn new_fonts() -> Option<(nwg::Font, nwg::Font)> {
    let mut ui_font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size_absolute(scaled(UI_FONT_HEIGHT as i32) as u32)
        .build(&mut ui_font)
        .ok()?;

    let mut output_font = nwg::Font::default();
    nwg::Font::builder()
        .family("Consolas")
        .size_absolute(scaled(OUTPUT_FONT_HEIGHT as i32) as u32)
        .build(&mut output_font)
        .ok()?;

    Some((ui_font, output_font))
}

fn apply_fonts(ui: &Ui, ui_font: &nwg::Font, output_font: &nwg::Font) {
    ui.status.set_font(Some(ui_font));
    ui.card.set_font(Some(ui_font));
    ui.disclaimer.set_font(Some(ui_font));
    ui.server_label.set_font(Some(ui_font));
    ui.server_input.set_font(Some(ui_font));
    ui.agent_token_label.set_font(Some(ui_font));
    ui.agent_token_input.set_font(Some(ui_font));
    ui.save_sync_button.set_font(Some(ui_font));
    ui.edit_agent_token_button.set_font(Some(ui_font));
    ui.token_label.set_font(Some(ui_font));
    ui.token_input.set_font(Some(ui_font));
    ui.save_button.set_font(Some(ui_font));
    ui.copy_button.set_font(Some(ui_font));
    ui.reset_button.set_font(Some(ui_font));
    ui.probe_button.set_font(Some(ui_font));
    ui.watch_button.set_font(Some(ui_font));
    ui.dashboard_button.set_font(Some(ui_font));
    ui.logs_button.set_font(Some(ui_font));
    ui.logs_dir_button.set_font(Some(ui_font));
    ui.resync_button.set_font(Some(ui_font));
    ui.output_button.set_font(Some(ui_font));
    ui.settings_button.set_font(Some(ui_font));
    ui.output.set_font(Some(output_font));
}
