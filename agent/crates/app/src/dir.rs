use std::path::PathBuf;

use agent_core::journal::JOURNAL_DIR_COMPONENTS;

pub fn resolve_journal_dir(argument: Option<String>) -> Result<PathBuf, String> {
    if let Some(argument) = argument {
        return Ok(PathBuf::from(argument));
    }
    if cfg!(windows) {
        let profile = std::env::var("USERPROFILE")
            .map_err(|_| "не задан USERPROFILE — укажите каталог журналов аргументом".to_owned())?;
        let mut path = PathBuf::from(profile).join("Saved Games");
        path.extend(JOURNAL_DIR_COMPONENTS);
        return Ok(path);
    }
    Err("укажите каталог журналов аргументом".to_owned())
}

pub fn data_dir() -> PathBuf {
    agent_sync::default_config_path()
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(executable_dir)
}

pub fn executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}
