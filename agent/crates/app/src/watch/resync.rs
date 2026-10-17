use std::path::Path;
use std::sync::atomic::AtomicBool;

use agent_core::journal::{HistoryStore, JournalHistory};

use crate::dir::data_dir;
use crate::say;

pub fn resync_until(journal_dir: &Path, running: &AtomicBool) -> Result<(), String> {
    HistoryStore::in_dir(&data_dir())
        .store(&JournalHistory::default())
        .map_err(|error| format!("не удалось сбросить историю журналов: {error}"))?;
    say!("История журналов сброшена: всё будет отправлено заново.");

    super::run_until(journal_dir, running)
}
