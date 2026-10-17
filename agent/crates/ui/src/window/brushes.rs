use crate::theme::{rgb, Palette};

pub(super) struct Brushes {
    pub(super) window: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    pub(super) field: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    pub(super) button: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    pub(super) pressed: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    pub(super) accent: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    pub(super) accent_text: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    pub(super) border: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    pub(super) focus: windows_sys::Win32::Graphics::Gdi::HBRUSH,
}

impl Brushes {
    pub(super) fn new(palette: Palette) -> Self {
        use windows_sys::Win32::Graphics::Gdi::CreateSolidBrush;

        unsafe {
            Self {
                window: CreateSolidBrush(rgb(palette.window)),
                field: CreateSolidBrush(rgb(palette.field)),
                button: CreateSolidBrush(rgb(palette.button)),
                pressed: CreateSolidBrush(rgb(palette.button_pressed)),
                accent: CreateSolidBrush(rgb(palette.accent)),
                accent_text: CreateSolidBrush(rgb(palette.accent_text)),
                border: CreateSolidBrush(rgb(palette.border)),
                focus: CreateSolidBrush(rgb(palette.focus)),
            }
        }
    }

    pub(super) fn replace_with(&mut self, palette: Palette) {
        use windows_sys::Win32::Graphics::Gdi::DeleteObject;

        let old = std::mem::replace(self, Self::new(palette));
        for brush in [
            old.window,
            old.field,
            old.button,
            old.pressed,
            old.accent,
            old.accent_text,
            old.border,
            old.focus,
        ] {
            unsafe { DeleteObject(brush as _) };
        }
    }
}
