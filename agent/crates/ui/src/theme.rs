#[derive(Clone, Copy)]
pub struct Palette {
    pub window: [u8; 3],
    pub text: [u8; 3],
    pub muted: [u8; 3],
    pub field: [u8; 3],
    pub button: [u8; 3],
    pub button_pressed: [u8; 3],
    pub border: [u8; 3],
    pub accent: [u8; 3],
    pub accent_text: [u8; 3],
    pub focus: [u8; 3],
}

const UBUNTU_ORANGE: [u8; 3] = [0xE9, 0x54, 0x20];
const WARM_GREY: [u8; 3] = [0xAE, 0xA7, 0x9F];

const COOL_GREY: [u8; 3] = [0x33, 0x33, 0x33];

pub const DARK: Palette = Palette {
    window: COOL_GREY,
    text: [0xFF, 0xFF, 0xFF],
    muted: WARM_GREY,
    field: [0x26, 0x26, 0x26],
    button: [0x26, 0x26, 0x26],
    button_pressed: [0x1B, 0x1B, 0x1B],
    border: WARM_GREY,
    accent: UBUNTU_ORANGE,
    accent_text: [0x00, 0x00, 0x00],
    focus: [0xFF, 0xFF, 0xFF],
};

pub const LIGHT: Palette = Palette {
    window: [0xFF, 0xFF, 0xFF],
    text: [0x11, 0x11, 0x11],
    muted: [0x5E, 0x5E, 0x5E],
    field: [0xFF, 0xFF, 0xFF],
    button: [0xF2, 0xF1, 0xF0],
    button_pressed: [0xDF, 0xDC, 0xD9],
    border: [0x66, 0x66, 0x66],
    accent: UBUNTU_ORANGE,
    accent_text: [0x00, 0x00, 0x00],
    focus: UBUNTU_ORANGE,
};

pub fn rgb(color: [u8; 3]) -> u32 {
    u32::from(color[0]) | (u32::from(color[1]) << 8) | (u32::from(color[2]) << 16)
}

#[cfg(test)]
fn luminance(color: [u8; 3]) -> f64 {
    fn channel(value: u8) -> f64 {
        let value = f64::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel(color[0]) + 0.7152 * channel(color[1]) + 0.0722 * channel(color[2])
}

#[cfg(test)]
pub fn contrast(first: [u8; 3], second: [u8; 3]) -> f64 {
    let (first, second) = (luminance(first), luminance(second));
    let (lighter, darker) = if first > second {
        (first, second)
    } else {
        (second, first)
    };

    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: f64 = 4.5;
    const NON_TEXT: f64 = 3.0;

    fn check(palette: Palette, name: &str) {
        let pairs = [
            ("текст на фоне окна", palette.text, palette.window, TEXT),
            ("текст в поле", palette.text, palette.field, TEXT),
            ("текст на кнопке", palette.text, palette.button, TEXT),
            (
                "текст на нажатой кнопке",
                palette.text,
                palette.button_pressed,
                TEXT,
            ),
            (
                "текст на включённом режиме",
                palette.accent_text,
                palette.accent,
                TEXT,
            ),
            ("приглушённый текст", palette.muted, palette.window, TEXT),
            (
                "граница на фоне окна",
                palette.border,
                palette.window,
                NON_TEXT,
            ),
            (
                "граница на кнопке",
                palette.border,
                palette.button,
                NON_TEXT,
            ),
            (
                "граница на нажатой кнопке",
                palette.border,
                palette.button_pressed,
                NON_TEXT,
            ),
            (
                "включённый режим виден среди обычных кнопок",
                palette.accent,
                palette.button,
                NON_TEXT,
            ),
            (
                "рамка фокуса на фоне окна",
                palette.focus,
                palette.window,
                NON_TEXT,
            ),
            (
                "рамка фокуса на кнопке",
                palette.focus,
                palette.button,
                NON_TEXT,
            ),
            (
                "рамка фокуса на включённом режиме",
                palette.accent_text,
                palette.accent,
                NON_TEXT,
            ),
        ];

        for (label, first, second, floor) in pairs {
            let ratio = contrast(first, second);
            assert!(
                ratio >= floor,
                "{name}: {label} — {ratio:.2}:1, нужно не ниже {floor}:1",
            );
        }
    }

    #[test]
    fn dark_palette_meets_wcag() {
        check(DARK, "тёмная тема");
    }

    #[test]
    fn light_palette_meets_wcag() {
        check(LIGHT, "светлая тема");
    }

    #[test]
    fn contrast_of_black_on_white_is_the_known_maximum() {
        let ratio = contrast([0, 0, 0], [255, 255, 255]);
        assert!((ratio - 21.0).abs() < 0.01, "{ratio}");
    }

    #[test]
    fn rgb_puts_blue_in_the_high_byte() {
        assert_eq!(rgb([0x11, 0x22, 0x33]), 0x0033_2211);
    }
}
