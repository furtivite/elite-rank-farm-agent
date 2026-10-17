use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use agent_core::journal::{JournalPosition, JournalTailer, JournalWatcher, PositionStore};

const EVENT_A: &str =
    r#"{"timestamp":"2026-09-01T20:00:00Z","event":"Commander","FID":"F1","Name":"Jameson"}"#;
const EVENT_B: &str =
    r#"{"timestamp":"2026-09-01T20:00:02Z","event":"Rank","Empire":7,"Federation":2}"#;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-core-position-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("создать каталог");
    dir
}

fn append(path: &PathBuf, text: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("открыть журнал");
    file.write_all(text.as_bytes()).expect("дописать");
}

#[test]
fn missing_file_means_no_position() {
    let store = PositionStore::in_dir(&temp_dir("missing"));
    assert_eq!(store.load().unwrap(), None);
}

#[test]
fn stored_position_survives_reload() {
    let dir = temp_dir("roundtrip");
    let store = PositionStore::in_dir(&dir);
    let position = JournalPosition {
        path: dir.join("Journal.2026-09-01T200000.01.log"),
        offset: 1234,
    };

    store.store(&position).unwrap();
    assert_eq!(store.load().unwrap(), Some(position));
    assert!(!store.path().with_extension("json.tmp").exists());
}

#[test]
fn newer_position_replaces_older() {
    let dir = temp_dir("replace");
    let store = PositionStore::in_dir(&dir);
    let path = dir.join("Journal.2026-09-01T200000.01.log");

    store
        .store(&JournalPosition {
            path: path.clone(),
            offset: 10,
        })
        .unwrap();
    store
        .store(&JournalPosition {
            path: path.clone(),
            offset: 20,
        })
        .unwrap();

    assert_eq!(store.load().unwrap().unwrap().offset, 20);
}

#[test]
fn malformed_file_is_reported_not_ignored() {
    let dir = temp_dir("malformed");
    let store = PositionStore::in_dir(&dir);
    fs::write(store.path(), "{not json").unwrap();

    let error = store.load().unwrap_err();
    assert!(error.to_string().contains("позиция журнала"), "{error}");
}

#[test]
fn watcher_resumed_from_stored_position_does_not_reread_old_lines() {
    let dir = temp_dir("resume");
    let journal = dir.join("Journal.2026-09-01T200000.01.log");
    append(&journal, &format!("{EVENT_A}\n"));

    let store = PositionStore::in_dir(&dir);
    let stored = {
        let mut tailer = JournalTailer::new(journal.clone());
        assert_eq!(tailer.read_new().unwrap().len(), 1);
        let position = JournalPosition {
            path: tailer.path().to_path_buf(),
            offset: tailer.offset(),
        };
        store.store(&position).unwrap();
        position
    };

    append(&journal, &format!("{EVENT_B}\n"));

    let loaded = store.load().unwrap().expect("позиция сохранена");
    assert_eq!(loaded, stored);
    let mut watcher = JournalWatcher::resume(dir.clone(), loaded.path, loaded.offset);
    let lines = watcher.poll().unwrap();
    assert_eq!(
        lines.len(),
        1,
        "после рестарта читается только новая строка"
    );
}
