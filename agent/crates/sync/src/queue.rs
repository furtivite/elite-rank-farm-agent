use std::path::{Path, PathBuf};

use agent_core::CommanderState;

pub const PENDING_FILE_NAME: &str = "pending-state.json";

#[derive(Debug)]
pub enum QueueError {
    Io { path: PathBuf, reason: String },
    Malformed { path: PathBuf, reason: String },
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::Malformed { path, reason } => f.write_str(&agent_core::tr!(
                "{}: not readable as Commander State: {reason}",
                "{}: не разбирается как Commander State: {reason}",
                path.display()
            )),
        }
    }
}

impl std::error::Error for QueueError {}

pub struct PendingState {
    path: PathBuf,
}

impl PendingState {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn in_dir(dir: &Path) -> Self {
        Self::at(dir.join(PENDING_FILE_NAME))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn store(&self, state: &CommanderState) -> Result<(), QueueError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| QueueError::Io {
                    path: parent.to_path_buf(),
                    reason: error.to_string(),
                })?;
            }
        }
        let json = serde_json::to_string_pretty(state).map_err(|error| QueueError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;

        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, json).map_err(|error| QueueError::Io {
            path: temporary.clone(),
            reason: error.to_string(),
        })?;
        std::fs::rename(&temporary, &self.path).map_err(|error| QueueError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })
    }

    pub fn load(&self) -> Result<Option<CommanderState>, QueueError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(QueueError::Io {
                    path: self.path.clone(),
                    reason: error.to_string(),
                })
            }
        };

        serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| QueueError::Malformed {
                path: self.path.clone(),
                reason: error.to_string(),
            })
    }

    pub fn clear(&self) -> Result<(), QueueError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(QueueError::Io {
                path: self.path.clone(),
                reason: error.to_string(),
            }),
        }
    }
}
