use std::cell::Cell;
use std::rc::Rc;

use native_windows_gui as nwg;

use super::dpi::{scaled, OUTPUT_FONT_HEIGHT};
use super::paint::{make_owner_drawn, plain_label};
use super::palette::apply_dark_title_bar;
use crate::icon::{set_window_icons, WindowIcons};
use crate::theme::Palette;
use crate::window_title;

pub(super) struct Ui {
    pub(super) window: nwg::Window,
    #[allow(dead_code)]
    pub(super) icons: WindowIcons,
    pub(super) status: nwg::Label,
    pub(super) card: nwg::Label,
    pub(super) disclaimer: nwg::Label,
    pub(super) separator: nwg::Label,
    pub(super) label_hooks: Vec<nwg::RawEventHandler>,
    pub(super) server_label: nwg::Label,
    pub(super) server_input: nwg::TextInput,
    pub(super) agent_token_label: nwg::Label,
    pub(super) agent_token_input: nwg::TextInput,
    pub(super) save_sync_button: nwg::Button,
    pub(super) edit_agent_token_button: nwg::Button,
    pub(super) token_label: nwg::Label,
    pub(super) token_input: nwg::TextInput,
    pub(super) save_button: nwg::Button,
    pub(super) copy_button: nwg::Button,
    pub(super) reset_button: nwg::Button,
    pub(super) probe_button: nwg::Button,
    pub(super) watch_button: nwg::Button,
    pub(super) dashboard_button: nwg::Button,
    pub(super) logs_button: nwg::Button,
    pub(super) logs_dir_button: nwg::Button,
    pub(super) resync_button: nwg::Button,
    pub(super) output_button: nwg::Button,
    pub(super) settings_button: nwg::Button,
    pub(super) output: nwg::RichTextBox,
    pub(super) panels: Panels,
    pub(super) palette: Rc<std::cell::Cell<Palette>>,
}

impl Ui {
    pub(super) fn buttons(&self) -> [&nwg::Button; 13] {
        [
            &self.watch_button,
            &self.probe_button,
            &self.output_button,
            &self.settings_button,
            &self.save_sync_button,
            &self.edit_agent_token_button,
            &self.save_button,
            &self.copy_button,
            &self.reset_button,
            &self.dashboard_button,
            &self.logs_button,
            &self.logs_dir_button,
            &self.resync_button,
        ]
    }
}

#[derive(Default)]
pub(super) struct Panels {
    pub(super) output_open: Cell<bool>,
    pub(super) settings_open: Cell<bool>,
    pub(super) agent_token_stored: Cell<bool>,
    pub(super) read_token_stored: Cell<bool>,
}

pub(super) fn build(
    dark: bool,
    palette: Palette,
    palette_cell: &Rc<Cell<Palette>>,
) -> Result<(Ui, nwg::Notice), String> {
    let mut window = nwg::Window::default();
    nwg::Window::builder()
        .size((scaled(200), scaled(200)))
        .position((300, 180))
        .title(&window_title())
        .flags(nwg::WindowFlags::MAIN_WINDOW | nwg::WindowFlags::VISIBLE)
        .build(&mut window)
        .map_err(|error| format!("не удалось создать окно: {error}"))?;

    let mut status = nwg::Label::default();
    let mut card = nwg::Label::default();
    let mut server_label = nwg::Label::default();
    let mut agent_token_label = nwg::Label::default();
    let mut token_label = nwg::Label::default();
    let mut disclaimer = nwg::Label::default();
    let mut separator = nwg::Label::default();

    let label = |target: &mut nwg::Label, text: &str| -> Result<(), String> {
        nwg::Label::builder()
            .text(text)
            .parent(&window)
            .build(target)
            .map_err(|error| error.to_string())
    };

    label(&mut status, "")?;
    label(&mut card, "")?;

    let mut watch_button = nwg::Button::default();
    let mut probe_button = nwg::Button::default();
    let mut output_button = nwg::Button::default();
    let mut settings_button = nwg::Button::default();
    for (target, text) in [
        (&mut watch_button, ""),
        (&mut probe_button, ""),
        (&mut output_button, ""),
        (&mut settings_button, ""),
    ] {
        nwg::Button::builder()
            .text(text)
            .parent(&window)
            .build(target)
            .map_err(|error| error.to_string())?;
    }

    label(
        &mut server_label,
        "Адрес сервера, например https://пример.ру",
    )?;

    let mut server_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .parent(&window)
        .readonly(true)
        .build(&mut server_input)
        .map_err(|error| error.to_string())?;

    label(
        &mut agent_token_label,
        "Ключ агента — им отправляется состояние",
    )?;

    let mut agent_token_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .password(Some('•'))
        .parent(&window)
        .build(&mut agent_token_input)
        .map_err(|error| error.to_string())?;

    label(&mut token_label, "Ключ для сайта — только чтение")?;

    let mut token_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .password(Some('•'))
        .parent(&window)
        .build(&mut token_input)
        .map_err(|error| error.to_string())?;

    label(
        &mut disclaimer,
        "Неофициальный фанатский проект. Не связан с Frontier Developments plc и не одобрен ею.\r\nElite Dangerous и связанные материалы принадлежат правообладателям.",
    )?;

    label(&mut separator, "")?;

    let label_hooks = [
        &status,
        &card,
        &server_label,
        &agent_token_label,
        &token_label,
        &disclaimer,
        &separator,
    ]
    .into_iter()
    .filter_map(plain_label)
    .collect();

    let mut save_sync_button = nwg::Button::default();
    let mut save_button = nwg::Button::default();
    let mut copy_button = nwg::Button::default();
    let mut reset_button = nwg::Button::default();
    let mut dashboard_button = nwg::Button::default();
    let mut logs_button = nwg::Button::default();
    let mut logs_dir_button = nwg::Button::default();
    let mut resync_button = nwg::Button::default();
    let mut edit_agent_token_button = nwg::Button::default();

    for (target, text) in [
        (&mut save_sync_button, "Сохранить"),
        (&mut edit_agent_token_button, "Изменить ключ агента"),
        (&mut save_button, "Сохранить"),
        (&mut copy_button, "Копировать"),
        (&mut reset_button, "Сбросить"),
        (&mut dashboard_button, "Открыть сайт"),
        (&mut logs_button, "Открыть логи"),
        (&mut logs_dir_button, "Папка с логами"),
        (&mut resync_button, "Отправить всё заново"),
    ] {
        nwg::Button::builder()
            .text(text)
            .parent(&window)
            .build(target)
            .map_err(|error| error.to_string())?;
    }

    let mut output = nwg::RichTextBox::default();
    nwg::RichTextBox::builder()
        .parent(&window)
        .readonly(true)
        .build(&mut output)
        .map_err(|error| error.to_string())?;

    let mut notice = nwg::Notice::default();
    nwg::Notice::builder()
        .parent(&window)
        .build(&mut notice)
        .map_err(|error| error.to_string())?;

    let mut output_font = nwg::Font::default();
    nwg::Font::builder()
        .family("Consolas")
        .size_absolute(scaled(OUTPUT_FONT_HEIGHT as i32) as u32)
        .build(&mut output_font)
        .map_err(|error| format!("не удалось выбрать шрифт вывода: {error}"))?;
    output.set_font(Some(&output_font));

    output.set_background_color(palette.field);
    output.set_char_format(&nwg::CharFormat {
        text_color: Some(palette.text),
        ..Default::default()
    });

    let icons = set_window_icons(&window, dark);

    apply_dark_title_bar(&window, dark);

    for handle in [
        save_sync_button.handle,
        save_button.handle,
        copy_button.handle,
        reset_button.handle,
        edit_agent_token_button.handle,
        dashboard_button.handle,
        logs_button.handle,
        logs_dir_button.handle,
        resync_button.handle,
        probe_button.handle,
        watch_button.handle,
        output_button.handle,
        settings_button.handle,
    ] {
        make_owner_drawn(handle);
    }

    Ok((
        Ui {
            window,
            icons,
            status,
            card,
            disclaimer,
            separator,
            label_hooks,
            server_label,
            server_input,
            agent_token_label,
            agent_token_input,
            save_sync_button,
            edit_agent_token_button,
            token_label,
            token_input,
            save_button,
            copy_button,
            reset_button,
            probe_button,
            watch_button,
            dashboard_button,
            logs_button,
            logs_dir_button,
            resync_button,
            output_button,
            settings_button,
            output,
            panels: Panels::default(),
            palette: Rc::clone(palette_cell),
        },
        notice,
    ))
}
