use std::sync::Arc;

use agent_sync::{SyncConfig, DEFAULT_BASE_URL};
use agent_ui::{AppHooks, Report};

use crate::dir::resolve_journal_dir;
use crate::{log, probe, read_token, say, sink, watch};

#[cfg(windows)]
pub fn alert(text: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let text = wide(text);
    let caption = wide("Elite Rank Farm Navigator — агент");
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_ICONERROR | MB_OK,
        );
    }
}

#[cfg(not(windows))]
pub fn alert(text: &str) {
    eprintln!("{text}");
}

pub fn install_panic_alert() {
    std::panic::set_hook(Box::new(|info| {
        let text = format!("Агент аварийно завершился.\n\n{info}");
        log::append(&text);
        alert(&text);
    }));
}

pub fn run() -> Result<(), String> {
    agent_ui::run(AppHooks {
        load_sync: Box::new(load_sync),
        save_sync: Box::new(save_sync),
        load_read_token: Box::new(read_token::load),
        save_read_token: Box::new(read_token::save),
        clear_read_token: Box::new(read_token::clear),
        open_dashboard: Box::new(open_dashboard),
        open_logs: Box::new(open_logs),
        open_logs_dir: Box::new(open_logs_dir),
        probe: Arc::new(|out| {
            with_reports(out, || {
                let dir = resolve_journal_dir(None)?;
                say!("Каталог журналов: {}", dir.display());
                let state = probe::collect(&dir, true)?;
                probe::report(&state);
                Ok(())
            });
        }),
        watch: Arc::new(|out, running| {
            with_reports(out, || {
                let dir = resolve_journal_dir(None)?;
                say!("Каталог журналов: {}", dir.display());
                watch::run_until(&dir, running)
            });
        }),
        resync: Arc::new(|out, running| {
            with_reports(out, || {
                let dir = resolve_journal_dir(None)?;
                say!("Каталог журналов: {}", dir.display());
                watch::resync_until(&dir, running)
            });
        }),
    })
}

fn with_reports(out: agent_ui::Sink, work: impl FnOnce() -> Result<(), String>) {
    let lines = Arc::clone(&out);
    sink::install(Box::new(move |text| lines(Report::Line(text))));

    let statuses = Arc::clone(&out);
    sink::install_status(Box::new(move |update| {
        statuses(match update {
            sink::Status::Game(found) => Report::Game(found),
            sink::Status::Connection(ok) => Report::Connection(ok),
            sink::Status::LastSync(at) => Report::LastSync(at),
            sink::Status::Commander(card) => Report::Commander(agent_ui::CommanderLines {
                cmdr: card.cmdr,
                ship: card.ship,
                suit: card.suit,
                system: card.system,
                station: card.station,
            }),
        });
    }));

    if let Err(reason) = work() {
        say!("Ошибка: {reason}");
    }
}

fn load_sync() -> agent_ui::SyncSettings {
    match SyncConfig::load() {
        Ok(config) => agent_ui::SyncSettings {
            base_url: config.base_url().to_owned(),
            token_saved: true,
        },
        Err(_) => agent_ui::SyncSettings::default(),
    }
}

fn save_sync(token: &str) -> Result<(), String> {
    let token = token.trim();
    let token = if token.is_empty() {
        SyncConfig::load()
            .map(|config| config.device_token().to_owned())
            .map_err(|_| "ключ агента ещё не сохранён — вставьте его".to_owned())?
    } else {
        token.to_owned()
    };

    SyncConfig::new(DEFAULT_BASE_URL, token)
        .map_err(|error| error.to_string())?
        .save()
        .map_err(|error| error.to_string())
}

fn open_dashboard() -> Result<(), String> {
    open(DEFAULT_BASE_URL)
}

fn open_logs() -> Result<(), String> {
    let path = log::path();
    if !path.exists() {
        return Err(format!(
            "лог ещё не заведён: он появится в папке агента ({})",
            path.display()
        ));
    }

    open_text_file(&path)
}

fn open_logs_dir() -> Result<(), String> {
    let path = log::path();
    let dir = path
        .parent()
        .ok_or_else(|| format!("не удалось определить папку лога: {}", path.display()))?;

    open_directory(dir)
}

#[cfg(windows)]
fn open_directory(path: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(not(windows))]
fn open_directory(path: &std::path::Path) -> Result<(), String> {
    open(&path.display().to_string())
}

#[cfg(windows)]
fn open_text_file(path: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("notepad.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(not(windows))]
fn open_text_file(path: &std::path::Path) -> Result<(), String> {
    open(&path.display().to_string())
}

#[cfg(windows)]
fn open(target: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    std::process::Command::new("cmd")
        .args(["/C", "start", "", target])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn open(target: &str) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
