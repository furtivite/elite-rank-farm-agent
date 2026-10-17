use native_windows_gui as nwg;

use crate::theme::{Palette, DARK, LIGHT};

fn high_contrast_on() -> bool {
    use windows_sys::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETHIGHCONTRAST};

    let mut info = HIGHCONTRASTW {
        cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
        dwFlags: 0,
        lpszDefaultScheme: std::ptr::null_mut(),
    };

    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            info.cbSize,
            (&mut info as *mut HIGHCONTRASTW).cast(),
            0,
        )
    };

    ok != 0 && info.dwFlags & HCF_HIGHCONTRASTON != 0
}

fn system_palette() -> Palette {
    use windows_sys::Win32::Graphics::Gdi::{
        GetSysColor, COLOR_BTNFACE, COLOR_BTNTEXT, COLOR_GRAYTEXT, COLOR_HIGHLIGHT,
        COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWTEXT,
    };

    let color = |index: i32| -> [u8; 3] {
        let value = unsafe { GetSysColor(index) };
        [
            (value & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            ((value >> 16) & 0xFF) as u8,
        ]
    };

    let window = color(COLOR_WINDOW);
    let text = color(COLOR_WINDOWTEXT);

    Palette {
        window,
        text,
        muted: color(COLOR_GRAYTEXT),
        field: window,
        button: color(COLOR_BTNFACE),
        button_pressed: color(COLOR_HIGHLIGHT),
        border: color(COLOR_BTNTEXT),
        accent: color(COLOR_HIGHLIGHT),
        accent_text: color(COLOR_HIGHLIGHTTEXT),
        focus: text,
    }
}

pub(super) fn current_palette() -> Palette {
    if high_contrast_on() {
        system_palette()
    } else if system_prefers_dark() {
        DARK
    } else {
        LIGHT
    }
}

pub(super) fn apply_dark_title_bar(window: &nwg::Window, dark: bool) {
    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

    let Some(hwnd) = window.handle.hwnd() else {
        return;
    };

    let value: i32 = i32::from(dark);
    unsafe {
        windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute(
            hwnd as _,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&value as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
    }
}

pub(super) fn system_prefers_dark() -> bool {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

    let path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

    let mut value: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut value as *mut u32).cast(),
            &mut size,
        )
    };

    status == 0 && value == 0
}
