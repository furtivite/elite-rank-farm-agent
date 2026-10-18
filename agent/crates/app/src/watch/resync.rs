use std::path::Path;
use std::sync::atomic::AtomicBool;

use agent_core::journal::{HistoryStore, JournalHistory};

use crate::dir::data_dir;
use crate::say_tr;

pub fn resync_until(journal_dir: &Path, running: &AtomicBool) -> Result<(), String> {
    HistoryStore::in_dir(&data_dir())
        .store(&JournalHistory::default())
        .map_err(|error| {
            agent_core::tr!(
                "could not reset the journal history: {error}",
                "не удалось сбросить историю журналов: {error}"
            )
        })?;
    say_tr!(
        "The journal history is reset: everything will be sent again.",
        "История журналов сброшена: всё будет отправлено заново."
    );

    super::run_until(journal_dir, running)
}
