use native_windows_gui as nwg;

use super::controls::Ui;
use crate::text::{Language, Texts};
use crate::window_title;

#[derive(Default)]
pub(super) struct StatusState {
    pub(super) game: Option<bool>,
    pub(super) connection: Option<bool>,
    pub(super) last_sync: Option<String>,
}

impl StatusState {
    pub(super) fn line(&self, t: &Texts) -> String {
        let game = match self.game {
            Some(true) => t.journals_found,
            Some(false) => t.journals_missing,
            None => t.journals_unknown,
        };
        let connection = match self.connection {
            Some(true) => t.connection_ok,
            Some(false) => t.connection_lost,
            None => t.connection_unknown,
        };
        let sync = match &self.last_sync {
            Some(at) => format!("{}: {at}", t.last_sync),
            None => t.no_sync_yet.to_owned(),
        };

        format!("{game}\r\n{connection}\r\n{sync}")
    }
}

pub(super) fn show_language(ui: &Ui, active: Option<bool>, state: &StatusState) {
    let t = ui.texts();
    ui.window.set_text(&window_title(ui.language.get()));

    for (button, text) in [
        (&ui.save_sync_button, t.save),
        (&ui.edit_agent_token_button, t.edit_agent_token),
        (&ui.save_button, t.save),
        (&ui.copy_button, t.copy),
        (&ui.reset_button, t.reset),
        (&ui.dashboard_button, t.open_site),
        (&ui.logs_button, t.open_log),
        (&ui.logs_dir_button, t.logs_folder),
        (&ui.resync_button, t.resync),
    ] {
        button.set_text(text);
    }
    ui.server_label.set_text(t.server_label);
    ui.disclaimer.set_text(t.disclaimer);
    show_agent_token_state(ui, ui.panels.agent_token_stored.get());
    show_token_state(ui, ui.panels.read_token_stored.get());
    show_mode_labels(ui, active);
    show_panel_labels(ui);
    ui.status.set_text(&state.line(t));

    let (english, russian) = match ui.language.get() {
        Language::English => (
            nwg::RadioButtonState::Checked,
            nwg::RadioButtonState::Unchecked,
        ),
        Language::Russian => (
            nwg::RadioButtonState::Unchecked,
            nwg::RadioButtonState::Checked,
        ),
    };
    ui.english_radio.set_check_state(english);
    ui.russian_radio.set_check_state(russian);
}

pub(super) fn append(ui: &Ui, line: &str) {
    let output = &ui.output;
    output.appendln(line);

    let palette = ui.palette.get();
    output.set_background_color(palette.field);

    let end = output.len();
    output.set_selection(0..end);
    output.set_char_format(&nwg::CharFormat {
        text_color: Some(palette.text),
        ..Default::default()
    });
    output.set_selection(end..end);
    output.scroll_lastline();
}

pub(super) fn show_agent_token_state(ui: &Ui, stored: bool) {
    ui.panels.agent_token_stored.set(stored);

    let t = ui.texts();
    ui.agent_token_label.set_text(if stored {
        t.agent_token_saved
    } else {
        t.agent_token_label
    });
}

pub(super) fn show_token_state(ui: &Ui, stored: bool) {
    ui.panels.read_token_stored.set(stored);
    let t = ui.texts();
    ui.token_label.set_text(if stored {
        t.read_token_saved
    } else {
        t.read_token_label
    });
}
pub(super) fn show_mode_labels(ui: &Ui, active: Option<bool>) {
    let t = ui.texts();
    ui.watch_button.set_text(if active == Some(false) {
        t.watch_off
    } else {
        t.watch_on
    });
    ui.probe_button.set_text(if active == Some(true) {
        t.probe_off
    } else {
        t.probe_on
    });
}

pub(super) fn show_panel_labels(ui: &Ui) {
    let t = ui.texts();
    ui.output_button.set_text(if ui.panels.output_open.get() {
        t.output_hide
    } else {
        t.output_show
    });
    ui.settings_button
        .set_text(if ui.panels.settings_open.get() {
            t.settings_hide
        } else {
            t.settings_show
        });
}

pub(super) fn announce_change(handle: nwg::ControlHandle) {
    use windows_sys::Win32::UI::Accessibility::NotifyWinEvent;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CHILDID_SELF, EVENT_OBJECT_NAMECHANGE, OBJID_CLIENT,
    };

    let Some(hwnd) = handle.hwnd() else {
        return;
    };

    unsafe {
        NotifyWinEvent(
            EVENT_OBJECT_NAMECHANGE,
            hwnd as _,
            OBJID_CLIENT,
            CHILDID_SELF as i32,
        )
    };
}
