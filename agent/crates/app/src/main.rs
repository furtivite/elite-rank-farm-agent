#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod dir;
mod instance;
mod log;
mod probe;
mod read_token;
mod relocate;
mod send;
mod shutdown;
mod sink;
mod ui;
mod watch;

use std::process::ExitCode;

use dir::resolve_journal_dir;

const USAGE: &str = "\
elite-agent — Elite Dangerous Rank Farm Navigator

    elite-agent [<каталог журналов>]         прочитать журналы и показать состояние
    elite-agent send [<каталог журналов>]    отправить состояние один раз
    elite-agent watch [<каталог журналов>]   следить за журналом и отправлять
                                             при изменении
    elite-agent rebuild                      попросить сервер пересобрать
                                             снимок галактики

Без аргументов на Windows открывается окно агента.

Без каталога на Windows берётся
%USERPROFILE%\\Saved Games\\Frontier Developments\\Elite Dangerous.

Настройка отправки — окно агента, файл elite-agent.json в
%APPDATA%\\elite-agent либо переменные ELITE_AGENT_BASE_URL и
ELITE_AGENT_DEVICE_TOKEN.";

fn main() -> ExitCode {
    let moved = relocate::move_old_files(&dir::executable_dir(), &dir::data_dir());

    let mut args = std::env::args().skip(1);
    let first = args.next();

    if first.is_some() {
        attach_parent_console();
    }

    if !moved.is_empty() {
        log::append(&format!(
            "файлы агента перенесены из-под программы в {}: {}",
            dir::data_dir().display(),
            moved.join(", ")
        ));
    }

    if cfg!(windows) && first.is_none() {
        ui::install_panic_alert();
        log::append(&format!(
            "запуск окна агента, версия {}",
            env!("CARGO_PKG_VERSION")
        ));

        let _lock = match take_lock() {
            Ok(lock) => lock,
            Err(message) => {
                log::append(&message);
                ui::alert(&message);
                return ExitCode::SUCCESS;
            }
        };

        return match ui::run() {
            Ok(()) => {
                log::append("окно агента закрыто");
                ExitCode::SUCCESS
            }
            Err(reason) => {
                log::append(&format!("окно агента не открылось: {reason}"));
                ui::alert(&format!("Окно агента не открылось.\n\n{reason}"));
                ExitCode::FAILURE
            }
        };
    }

    let (command, path) = match first.as_deref() {
        Some("--help" | "-h") => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Some("send") => (Command::Send, args.next()),
        Some("watch") => (Command::Watch, args.next()),
        Some("rebuild") => (Command::Rebuild, None),
        other => (Command::Probe, other.map(ToOwned::to_owned)),
    };

    if matches!(command, Command::Rebuild) {
        return match watch::rebuild_only() {
            Ok(()) => ExitCode::SUCCESS,
            Err(reason) => {
                eprintln!("{reason}");
                ExitCode::FAILURE
            }
        };
    }

    let dir = match resolve_journal_dir(path) {
        Ok(dir) => dir,
        Err(reason) => {
            eprintln!("{reason}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    println!("Каталог журналов: {}", dir.display());

    let _lock = if matches!(command, Command::Watch) {
        match take_lock() {
            Ok(lock) => lock,
            Err(message) => {
                eprintln!("{message}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };

    let result = match command {
        Command::Probe => probe::collect(&dir, true).map(|state| probe::report(&state)),
        Command::Send => send::run(&dir),
        Command::Watch => watch::run(&dir),
        Command::Rebuild => unreachable!("обработано выше, до поиска журналов"),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("{reason}");
            ExitCode::FAILURE
        }
    }
}

fn take_lock() -> Result<Option<instance::InstanceLock>, String> {
    match instance::acquire() {
        Ok(lock) => Ok(Some(lock)),
        Err(error @ instance::LockError::AlreadyRunning) => Err(error.to_string()),
        Err(error) => {
            log::append(&error.to_string());
            Ok(None)
        }
    }
}

#[derive(Clone, Copy)]
enum Command {
    Probe,
    Send,
    Watch,
    Rebuild,
}

#[cfg(windows)]
fn attach_parent_console() {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn attach_parent_console() {}
