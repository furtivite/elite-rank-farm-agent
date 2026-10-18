use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const POSITION_FILE_NAME: &str = "journal-position.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalPosition {
    pub path: PathBuf,
    pub offset: u64,
}

#[derive(Debug)]
pub enum PositionError {
    Io { path: PathBuf, reason: String },
    Malformed { path: PathBuf, reason: String },
}

impl std::fmt::Display for PositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::Malformed { path, reason } => f.write_str(&crate::tr!(
                "{}: not readable as a journal position: {reason}",
                "{}: не разбирается как позиция журнала: {reason}",
                path.display()
            )),
        }
    }
}

impl std::error::Error for PositionError {}

pub struct PositionStore {
    path: PathBuf,
}

impl PositionStore {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn in_dir(dir: &Path) -> Self {
        Self::at(dir.join(POSITION_FILE_NAME))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn store(&self, position: &JournalPosition) -> Result<(), PositionError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| PositionError::Io {
                    path: parent.to_path_buf(),
                    reason: error.to_string(),
                })?;
            }
        }
        let json = serde_json::to_string(position).map_err(|error| PositionError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;

        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, json).map_err(|error| PositionError::Io {
            path: temporary.clone(),
            reason: error.to_string(),
        })?;
        std::fs::rename(&temporary, &self.path).map_err(|error| PositionError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })
    }

    pub fn load(&self) -> Result<Option<JournalPosition>, PositionError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(PositionError::Io {
                    path: self.path.clone(),
                    reason: error.to_string(),
                })
            }
        };

        serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| PositionError::Malformed {
                path: self.path.clone(),
                reason: error.to_string(),
            })
    }
}
