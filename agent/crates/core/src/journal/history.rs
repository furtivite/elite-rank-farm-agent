use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const HISTORY_FILE_NAME: &str = "journal-history.json";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalHistory {
    pub files: BTreeMap<String, u64>,
}

impl JournalHistory {
    pub fn offset_of(&self, file_name: &str) -> u64 {
        self.files.get(file_name).copied().unwrap_or(0)
    }

    pub fn record(&mut self, file_name: &str, offset: u64) {
        self.files.insert(file_name.to_owned(), offset);
    }

    pub fn retain_existing(&mut self, existing: &[String]) {
        self.files
            .retain(|name, _| existing.iter().any(|it| it == name));
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[derive(Debug)]
pub enum HistoryError {
    Io { path: PathBuf, reason: String },
    Malformed { path: PathBuf, reason: String },
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::Malformed { path, reason } => f.write_str(&crate::tr!(
                "{}: not readable as the journal history: {reason}",
                "{}: не разбирается как история журналов: {reason}",
                path.display()
            )),
        }
    }
}

impl std::error::Error for HistoryError {}

pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn in_dir(dir: &Path) -> Self {
        Self::at(dir.join(HISTORY_FILE_NAME))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<JournalHistory, HistoryError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(JournalHistory::default())
            }
            Err(error) => {
                return Err(HistoryError::Io {
                    path: self.path.clone(),
                    reason: error.to_string(),
                })
            }
        };

        serde_json::from_str(&text).map_err(|error| HistoryError::Malformed {
            path: self.path.clone(),
            reason: error.to_string(),
        })
    }

    pub fn store(&self, history: &JournalHistory) -> Result<(), HistoryError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| HistoryError::Io {
                    path: parent.to_path_buf(),
                    reason: error.to_string(),
                })?;
            }
        }

        let json = serde_json::to_string(history).map_err(|error| HistoryError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;

        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, json).map_err(|error| HistoryError::Io {
            path: temporary.clone(),
            reason: error.to_string(),
        })?;
        std::fs::rename(&temporary, &self.path).map_err(|error| HistoryError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })
    }
}
