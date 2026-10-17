use native_windows_gui as nwg;

use super::controls::Ui;

#[derive(Default)]
pub(super) struct StatusState {
    pub(super) game: Option<bool>,
    pub(super) connection: Option<bool>,
    pub(super) last_sync: Option<String>,
}

impl StatusState {
    pub(super) fn line(&self) -> String {
        let game = match self.game {
            Some(true) => "журналы найдены",
            Some(false) => "журналы не найдены",
            None => "журналы: не проверялось",
        };
        let connection = match self.connection {
            Some(true) => "связь есть",
            Some(false) => "связи нет",
            None => "связь: не проверялась",
        };
        let sync = match &self.last_sync {
            Some(at) => format!("последняя отправка: {at}"),
            None => "отправок ещё не было".to_owned(),
        };

        format!("{game}\r\n{connection}\r\n{sync}")
    }
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

    ui.agent_token_label.set_text(if stored {
        "Ключ агента сохранён"
    } else {
        "Ключ агента — им отправляется состояние"
    });
}

pub(super) fn show_token_state(ui: &Ui, stored: bool) {
    ui.panels.read_token_stored.set(stored);
    ui.token_label.set_text(if stored {
        "Ключ для сайта сохранён"
    } else {
        "Ключ для сайта — вставьте его один раз"
    });
}
const WATCH_ON: &str = "Сессия: включить";
const WATCH_OFF: &str = "Сессия: выключить";
const PROBE_ON: &str = "Разовый прогон";
const PROBE_OFF: &str = "Остановить прогон";
const OUTPUT_SHOW: &str = "Показать вывод";
const OUTPUT_HIDE: &str = "Скрыть вывод";
const SETTINGS_SHOW: &str = "⚙ Настройки";
const SETTINGS_HIDE: &str = "⚙ Скрыть";

pub(super) const TOGGLING_LABELS: [&str; 8] = [
    WATCH_ON,
    WATCH_OFF,
    PROBE_ON,
    PROBE_OFF,
    OUTPUT_SHOW,
    OUTPUT_HIDE,
    SETTINGS_SHOW,
    SETTINGS_HIDE,
];

pub(super) fn show_mode_labels(ui: &Ui, active: Option<bool>) {
    ui.watch_button.set_text(if active == Some(false) {
        WATCH_OFF
    } else {
        WATCH_ON
    });
    ui.probe_button.set_text(if active == Some(true) {
        PROBE_OFF
    } else {
        PROBE_ON
    });
}

pub(super) fn show_panel_labels(ui: &Ui) {
    ui.output_button.set_text(if ui.panels.output_open.get() {
        OUTPUT_HIDE
    } else {
        OUTPUT_SHOW
    });
    ui.settings_button
        .set_text(if ui.panels.settings_open.get() {
            SETTINGS_HIDE
        } else {
            SETTINGS_SHOW
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
