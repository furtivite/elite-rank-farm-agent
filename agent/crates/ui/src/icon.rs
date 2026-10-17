#[cfg(windows)]
mod windows_only {
    use super::best_frame;

    use native_windows_gui as nwg;

    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateIconFromResourceEx, DestroyIcon, GetSystemMetrics, SendMessageW, ICON_BIG,
        ICON_SMALL, IMAGE_FLAGS, LR_DEFAULTCOLOR, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON,
        WM_SETICON,
    };

    pub(crate) struct WindowIcons {
        handles: Vec<isize>,
    }

    impl Drop for WindowIcons {
        fn drop(&mut self) {
            for handle in self.handles.drain(..) {
                unsafe { DestroyIcon(handle as _) };
            }
        }
    }

    pub(crate) fn set_window_icons(window: &nwg::Window, dark: bool) -> WindowIcons {
        let ico: &[u8] = if dark {
            include_bytes!("../../../assets/window-dark.ico")
        } else {
            include_bytes!("../../../assets/window-light.ico")
        };

        let mut icons = WindowIcons {
            handles: Vec::new(),
        };

        let Some(hwnd) = window.handle.hwnd() else {
            return icons;
        };

        for (which, metric_x, metric_y) in [
            (ICON_BIG, SM_CXICON, SM_CYICON),
            (ICON_SMALL, SM_CXSMICON, SM_CYSMICON),
        ] {
            let (width, height) =
                unsafe { (GetSystemMetrics(metric_x), GetSystemMetrics(metric_y)) };

            let Some(image) = best_frame(ico, width.max(0) as u32) else {
                continue;
            };

            let handle = unsafe {
                CreateIconFromResourceEx(
                    image.as_ptr(),
                    image.len() as u32,
                    1,
                    0x0003_0000,
                    width,
                    height,
                    LR_DEFAULTCOLOR as IMAGE_FLAGS,
                )
            };

            if handle.is_null() {
                continue;
            }

            unsafe { SendMessageW(hwnd as _, WM_SETICON, which as usize, handle as isize) };
            icons.handles.push(handle as isize);
        }

        icons
    }
}

#[cfg(windows)]
pub(crate) use windows_only::{set_window_icons, WindowIcons};

fn best_frame(ico: &[u8], want: u32) -> Option<&[u8]> {
    const HEADER: usize = 6;
    const ENTRY: usize = 16;

    let count = usize::from(u16::from_le_bytes([*ico.get(4)?, *ico.get(5)?]));

    let mut best: Option<(u32, u32, &[u8])> = None;
    for index in 0..count {
        let entry = ico.get(HEADER + index * ENTRY..HEADER + (index + 1) * ENTRY)?;
        let width = if entry[0] == 0 {
            256
        } else {
            u32::from(entry[0])
        };
        let length = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
        let image = ico.get(offset..offset.checked_add(length)?)?;

        let distance = width.abs_diff(want);
        if best.is_none_or(|(best_distance, best_width, _)| {
            distance < best_distance || (distance == best_distance && width > best_width)
        }) {
            best = Some((distance, width, image));
        }
    }

    best.map(|(_, _, image)| image)
}

#[cfg(test)]
mod tests {
    use super::best_frame;

    const DARK: &[u8] = include_bytes!("../../../assets/window-dark.ico");

    #[test]
    fn picks_the_frame_of_the_asked_size() {
        for (want, expected) in [
            (16, 16),
            (32, 32),
            (48, 48),
            (256, 256),
            (40, 48),
            (100, 48),
            (200, 256),
        ] {
            let frame = best_frame(DARK, want).expect("картинка найдена");
            assert_eq!(&frame[..4], b"\x89PNG", "кадр {want} — не PNG");
            let width = u32::from_be_bytes([frame[16], frame[17], frame[18], frame[19]]);
            assert_eq!(width, expected, "для {want} выбран кадр {width}");
        }
    }

    #[test]
    fn broken_file_gives_nothing_instead_of_panic() {
        assert!(best_frame(&[], 32).is_none());
        assert!(best_frame(&DARK[..20], 32).is_none());
        assert!(best_frame(&DARK[..100], 32).is_none());
    }
}
