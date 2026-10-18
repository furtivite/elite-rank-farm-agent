#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Russian,
}

pub(crate) const LANGUAGE_LABEL: &str = "Language · Язык";
pub(crate) const ENGLISH: &str = "English";
pub(crate) const RUSSIAN: &str = "Русский";

pub(crate) struct Texts {
    pub(crate) agent: &'static str,

    pub(crate) journals_found: &'static str,
    pub(crate) journals_missing: &'static str,
    pub(crate) journals_unknown: &'static str,
    pub(crate) connection_ok: &'static str,
    pub(crate) connection_lost: &'static str,
    pub(crate) connection_unknown: &'static str,
    pub(crate) last_sync: &'static str,
    pub(crate) no_sync_yet: &'static str,

    pub(crate) watch_on: &'static str,
    pub(crate) watch_off: &'static str,
    pub(crate) probe_on: &'static str,
    pub(crate) probe_off: &'static str,
    pub(crate) output_show: &'static str,
    pub(crate) output_hide: &'static str,
    pub(crate) settings_show: &'static str,
    pub(crate) settings_hide: &'static str,

    pub(crate) server_label: &'static str,
    pub(crate) agent_token_label: &'static str,
    pub(crate) agent_token_saved: &'static str,
    pub(crate) read_token_label: &'static str,
    pub(crate) read_token_saved: &'static str,

    pub(crate) save: &'static str,
    pub(crate) edit_agent_token: &'static str,
    pub(crate) copy: &'static str,
    pub(crate) reset: &'static str,
    pub(crate) open_site: &'static str,
    pub(crate) open_log: &'static str,
    pub(crate) logs_folder: &'static str,
    pub(crate) resync: &'static str,

    pub(crate) disclaimer: &'static str,

    pub(crate) settings_saved: &'static str,
    pub(crate) settings_not_saved: &'static str,
    pub(crate) key_empty: &'static str,
    pub(crate) key_saved: &'static str,
    pub(crate) key_not_saved: &'static str,
    pub(crate) nothing_to_copy: &'static str,
    pub(crate) key_copied: &'static str,
    pub(crate) key_forgotten: &'static str,
    pub(crate) key_not_reset: &'static str,
    pub(crate) site_not_opened: &'static str,
    pub(crate) log_not_opened: &'static str,
    pub(crate) logs_folder_not_opened: &'static str,
    pub(crate) stop_before_resync: &'static str,
    pub(crate) stopped: &'static str,
    pub(crate) mode_already_running: &'static str,
    pub(crate) mode_finished: &'static str,
    pub(crate) language_not_saved: &'static str,

    pub(crate) ui_init_failed: &'static str,
    pub(crate) font_failed: &'static str,
    pub(crate) window_failed: &'static str,
    pub(crate) output_font_failed: &'static str,
}

impl Texts {
    pub(crate) fn toggling_labels(&self) -> [&'static str; 8] {
        [
            self.watch_on,
            self.watch_off,
            self.probe_on,
            self.probe_off,
            self.output_show,
            self.output_hide,
            self.settings_show,
            self.settings_hide,
        ]
    }
}

pub(crate) fn texts(language: Language) -> &'static Texts {
    match language {
        Language::English => &EN,
        Language::Russian => &RU,
    }
}

static EN: Texts = Texts {
    agent: "agent",

    journals_found: "journals found",
    journals_missing: "journals not found",
    journals_unknown: "journals: not checked",
    connection_ok: "connected",
    connection_lost: "no connection",
    connection_unknown: "connection: not checked",
    last_sync: "last sent",
    no_sync_yet: "nothing sent yet",

    watch_on: "Session: on",
    watch_off: "Session: off",
    probe_on: "One-off run",
    probe_off: "Stop the run",
    output_show: "Show output",
    output_hide: "Hide output",
    settings_show: "⚙ Settings",
    settings_hide: "⚙ Hide",

    server_label: "Server address, e.g. https://example.com",
    agent_token_label: "Agent key — the state is sent with it",
    agent_token_saved: "Agent key saved",
    read_token_label: "Site key — paste it once",
    read_token_saved: "Site key saved",

    save: "Save",
    edit_agent_token: "Change agent key",
    copy: "Copy",
    reset: "Reset",
    open_site: "Open site",
    open_log: "Open log",
    logs_folder: "Logs folder",
    resync: "Send everything again",

    disclaimer: "Unofficial fan project. Not affiliated with or endorsed by Frontier Developments plc.\r\nElite Dangerous and related materials are the property of their respective owners.",

    settings_saved: "Settings saved to the user profile.",
    settings_not_saved: "Settings not saved",
    key_empty: "The key is empty: paste it into the field and press “Save”.",
    key_saved: "Key saved: it is encrypted by Windows and tied to your user account.",
    key_not_saved: "Key not saved",
    nothing_to_copy: "Nothing to copy: the key is not saved yet.",
    key_copied: "Key copied to the clipboard.",
    key_forgotten: "Key forgotten. Paste a new one when you need it.",
    key_not_reset: "Key not reset",
    site_not_opened: "Could not open the site",
    log_not_opened: "Could not open the log",
    logs_folder_not_opened: "Could not open the logs folder",
    stop_before_resync: "Stop the running mode first — then “Send everything again”.",
    stopped: "Stopped.",
    mode_already_running: "A mode is already running — stop it first.",
    mode_finished: "--- mode finished",
    language_not_saved: "Language not saved",

    ui_init_failed: "could not start the interface",
    font_failed: "could not pick the font",
    window_failed: "could not create the window",
    output_font_failed: "could not pick the output font",
};

static RU: Texts = Texts {
    agent: "агент",

    journals_found: "журналы найдены",
    journals_missing: "журналы не найдены",
    journals_unknown: "журналы: не проверялось",
    connection_ok: "связь есть",
    connection_lost: "связи нет",
    connection_unknown: "связь: не проверялась",
    last_sync: "последняя отправка",
    no_sync_yet: "отправок ещё не было",

    watch_on: "Сессия: включить",
    watch_off: "Сессия: выключить",
    probe_on: "Разовый прогон",
    probe_off: "Остановить прогон",
    output_show: "Показать вывод",
    output_hide: "Скрыть вывод",
    settings_show: "⚙ Настройки",
    settings_hide: "⚙ Скрыть",

    server_label: "Адрес сервера, например https://пример.ру",
    agent_token_label: "Ключ агента — им отправляется состояние",
    agent_token_saved: "Ключ агента сохранён",
    read_token_label: "Ключ для сайта — вставьте его один раз",
    read_token_saved: "Ключ для сайта сохранён",

    save: "Сохранить",
    edit_agent_token: "Изменить ключ агента",
    copy: "Копировать",
    reset: "Сбросить",
    open_site: "Открыть сайт",
    open_log: "Открыть логи",
    logs_folder: "Папка с логами",
    resync: "Отправить всё заново",

    disclaimer: "Неофициальный фанатский проект. Не связан с Frontier Developments plc и не одобрен ею.\r\nElite Dangerous и связанные материалы принадлежат правообладателям.",

    settings_saved: "Настройки сохранены в профиле пользователя.",
    settings_not_saved: "Настройки не сохранены",
    key_empty: "Ключ пуст: вставьте его в поле и нажмите «Сохранить».",
    key_saved: "Ключ сохранён: он зашифрован средствами Windows и привязан к вашей учётной записи.",
    key_not_saved: "Ключ не сохранён",
    nothing_to_copy: "Копировать нечего: ключ ещё не сохранён.",
    key_copied: "Ключ скопирован в буфер обмена.",
    key_forgotten: "Ключ забыт. Вставьте новый, когда понадобится.",
    key_not_reset: "Ключ не сброшен",
    site_not_opened: "Не удалось открыть сайт",
    log_not_opened: "Не удалось открыть лог",
    logs_folder_not_opened: "Не удалось открыть папку с логами",
    stop_before_resync: "Сначала остановите работающий режим — потом «Отправить всё заново».",
    stopped: "Остановлено.",
    mode_already_running: "Один режим уже работает — сначала остановите его.",
    mode_finished: "--- режим завершён",
    language_not_saved: "Язык не сохранён",

    ui_init_failed: "не удалось инициализировать интерфейс",
    font_failed: "не удалось выбрать шрифт",
    window_failed: "не удалось создать окно",
    output_font_failed: "не удалось выбрать шрифт вывода",
};

#[cfg(test)]
mod tests {
    use super::*;

    fn all(texts: &Texts) -> Vec<&'static str> {
        let mut values = vec![
            texts.agent,
            texts.journals_found,
            texts.journals_missing,
            texts.journals_unknown,
            texts.connection_ok,
            texts.connection_lost,
            texts.connection_unknown,
            texts.last_sync,
            texts.no_sync_yet,
            texts.server_label,
            texts.agent_token_label,
            texts.agent_token_saved,
            texts.read_token_label,
            texts.read_token_saved,
            texts.save,
            texts.edit_agent_token,
            texts.copy,
            texts.reset,
            texts.open_site,
            texts.open_log,
            texts.logs_folder,
            texts.resync,
            texts.disclaimer,
            texts.settings_saved,
            texts.settings_not_saved,
            texts.key_empty,
            texts.key_saved,
            texts.key_not_saved,
            texts.nothing_to_copy,
            texts.key_copied,
            texts.key_forgotten,
            texts.key_not_reset,
            texts.site_not_opened,
            texts.log_not_opened,
            texts.logs_folder_not_opened,
            texts.stop_before_resync,
            texts.stopped,
            texts.mode_already_running,
            texts.mode_finished,
            texts.language_not_saved,
            texts.ui_init_failed,
            texts.font_failed,
            texts.window_failed,
            texts.output_font_failed,
        ];
        values.extend(texts.toggling_labels());
        values
    }

    fn has_cyrillic(text: &str) -> bool {
        text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
    }

    #[test]
    fn english_is_the_default() {
        assert_eq!(Language::default(), Language::English);
    }

    #[test]
    fn english_window_has_no_russian_words() {
        for text in all(texts(Language::English)) {
            assert!(!text.is_empty());
            assert!(!has_cyrillic(text), "{text}");
        }
    }

    #[test]
    fn russian_window_is_filled_in() {
        for text in all(texts(Language::Russian)) {
            assert!(!text.is_empty());
        }
        assert!(has_cyrillic(texts(Language::Russian).watch_on));
    }

    #[test]
    fn the_disclaimer_keeps_both_sentences_in_both_languages() {
        for language in [Language::English, Language::Russian] {
            let disclaimer = texts(language).disclaimer;
            assert_eq!(disclaimer.matches("\r\n").count(), 1, "{disclaimer}");
            assert!(
                disclaimer.contains("Frontier Developments plc"),
                "{disclaimer}"
            );
        }
    }
}
