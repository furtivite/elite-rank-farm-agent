use std::fs::File;
use std::path::{Path, PathBuf};

use agent_sync::default_config_path;

pub const LOCK_FILE_NAME: &str = "elite-agent.lock";

#[derive(Debug)]
pub enum LockError {
    AlreadyRunning,
    Unavailable { path: PathBuf, reason: String },
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => f.write_str(&agent_core::tr!(
                "the agent is already running — no need for a second window",
                "агент уже работает — второе окно не нужно"
            )),
            Self::Unavailable { path, reason } => f.write_str(&agent_core::tr!(
                "could not create the lock {}: {reason}",
                "не удалось завести замок {}: {reason}",
                path.display()
            )),
        }
    }
}

impl std::error::Error for LockError {}

#[derive(Debug)]
pub struct InstanceLock {
    _file: File,
}

pub fn lock_path() -> PathBuf {
    match default_config_path().parent() {
        Some(dir) => dir.join(LOCK_FILE_NAME),
        None => PathBuf::from(LOCK_FILE_NAME),
    }
}

pub fn acquire() -> Result<InstanceLock, LockError> {
    acquire_at(lock_path())
}

pub fn acquire_at(path: PathBuf) -> Result<InstanceLock, LockError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| LockError::Unavailable {
                path: path.clone(),
                reason: error.to_string(),
            })?;
        }
    }

    match open_exclusive(&path) {
        Ok(file) => Ok(InstanceLock { _file: file }),
        Err(error) if is_sharing_violation(&error) => Err(LockError::AlreadyRunning),
        Err(error) => Err(LockError::Unavailable {
            path,
            reason: error.to_string(),
        }),
    }
}

#[cfg(windows)]
fn open_exclusive(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;

    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(path)
}

#[cfg(not(windows))]
fn open_exclusive(path: &Path) -> std::io::Result<File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
}

#[cfg(windows)]
fn is_sharing_violation(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::AlreadyExists
    ) || error.raw_os_error() == Some(32)
}

#[cfg(not(windows))]
fn is_sharing_violation(_error: &std::io::Error) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_lives_next_to_the_settings() {
        let path = lock_path();
        assert_eq!(path.file_name().unwrap(), LOCK_FILE_NAME);
        assert_eq!(path.parent(), default_config_path().parent());
    }

    #[cfg(windows)]
    #[test]
    fn second_agent_does_not_get_the_lock() {
        let path = std::env::temp_dir().join("elite-agent-lock-test.lock");
        let _ = std::fs::remove_file(&path);

        let first = acquire_at(path.clone()).expect("первый замок берётся");
        assert!(matches!(
            acquire_at(path.clone()),
            Err(LockError::AlreadyRunning)
        ));

        drop(first);
        acquire_at(path).expect("после закрытия замок свободен");
    }
}
