use std::io::Write;
use std::path::{Path, PathBuf};

use agent_core::journal::Observation;

pub const OBSERVATIONS_FILE_NAME: &str = "observations.jsonl";

pub const MAX_BATCH: usize = 200;

pub const MAX_QUEUED: usize = 5_000;

#[derive(Debug)]
pub enum QueueFileError {
    Io { path: PathBuf, reason: String },
}

impl std::fmt::Display for QueueFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, reason } => write!(f, "{}: {reason}", path.display()),
        }
    }
}

impl std::error::Error for QueueFileError {}

pub struct ObservationQueue {
    path: PathBuf,
}

impl ObservationQueue {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn in_dir(dir: &Path) -> Self {
        Self::at(dir.join(OBSERVATIONS_FILE_NAME))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, observations: &[Observation]) -> Result<(), QueueFileError> {
        if observations.is_empty() {
            return Ok(());
        }

        let mut buffer = String::new();
        for observation in observations {
            match serde_json::to_string(observation) {
                Ok(line) => {
                    buffer.push_str(&line);
                    buffer.push('\n');
                }
                Err(error) => eprintln!("наблюдение не сериализовалось: {error}"),
            }
        }

        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| QueueFileError::Io {
                    path: parent.to_path_buf(),
                    reason: error.to_string(),
                })?;
            }
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| QueueFileError::Io {
                path: self.path.clone(),
                reason: error.to_string(),
            })?;

        file.write_all(buffer.as_bytes())
            .map_err(|error| QueueFileError::Io {
                path: self.path.clone(),
                reason: error.to_string(),
            })
    }

    pub fn load(&self) -> Result<Vec<Observation>, QueueFileError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(QueueFileError::Io {
                    path: self.path.clone(),
                    reason: error.to_string(),
                })
            }
        };

        let mut result = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Observation>(line) {
                Ok(observation) => result.push(observation),
                Err(error) => eprintln!("строка очереди наблюдений не разобралась: {error}"),
            }
        }

        if result.len() > MAX_QUEUED {
            let dropped = result.len() - MAX_QUEUED;
            eprintln!("очередь наблюдений переполнена, отброшено самых старых: {dropped}");
            result.drain(..dropped);
        }

        Ok(result)
    }

    pub fn replace(&self, observations: &[Observation]) -> Result<(), QueueFileError> {
        if observations.is_empty() {
            return self.clear();
        }

        let mut buffer = String::new();
        for observation in observations {
            if let Ok(line) = serde_json::to_string(observation) {
                buffer.push_str(&line);
                buffer.push('\n');
            }
        }

        let temporary = self.path.with_extension("jsonl.tmp");
        std::fs::write(&temporary, buffer).map_err(|error| QueueFileError::Io {
            path: temporary.clone(),
            reason: error.to_string(),
        })?;

        std::fs::rename(&temporary, &self.path).map_err(|error| QueueFileError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })
    }

    pub fn clear(&self) -> Result<(), QueueFileError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(QueueFileError::Io {
                path: self.path.clone(),
                reason: error.to_string(),
            }),
        }
    }
}
