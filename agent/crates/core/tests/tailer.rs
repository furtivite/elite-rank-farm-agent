use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use agent_core::journal::{latest_journal, JournalTailer, JournalWatcher, ParsedLine};

const EVENT_A: &str =
    r#"{"timestamp":"2026-09-01T20:00:00Z","event":"Commander","FID":"F1","Name":"Jameson"}"#;
const EVENT_B: &str =
    r#"{"timestamp":"2026-09-01T20:00:02Z","event":"Rank","Empire":7,"Federation":2}"#;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-core-{name}"));
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

fn events(lines: &[ParsedLine]) -> Vec<&ParsedLine> {
    lines
        .iter()
        .filter(|line| matches!(line, ParsedLine::Event(_)))
        .collect()
}

#[test]
fn reads_only_new_lines_between_polls() {
    let dir = temp_dir("incremental");
    let path = dir.join("Journal.2026-09-01T200000.01.log");
    append(&path, &format!("{EVENT_A}\n"));

    let mut tailer = JournalTailer::new(path.clone());
    assert_eq!(events(&tailer.read_new().unwrap()).len(), 1);
    assert!(tailer.read_new().unwrap().is_empty());

    append(&path, &format!("{EVENT_B}\n"));
    assert_eq!(events(&tailer.read_new().unwrap()).len(), 1);
}

#[test]
fn incomplete_line_is_held_until_it_is_finished() {
    let dir = temp_dir("partial");
    let path = dir.join("Journal.2026-09-01T200000.01.log");
    let (head, tail) = EVENT_A.split_at(40);

    append(&path, head);
    let mut tailer = JournalTailer::new(path.clone());
    assert!(tailer.read_new().unwrap().is_empty());

    append(&path, &format!("{tail}\n"));
    assert_eq!(events(&tailer.read_new().unwrap()).len(), 1);
}

#[test]
fn resume_at_offset_does_not_replay_processed_lines() {
    let dir = temp_dir("resume");
    let path = dir.join("Journal.2026-09-01T200000.01.log");
    append(&path, &format!("{EVENT_A}\n"));

    let mut first = JournalTailer::new(path.clone());
    first.read_new().unwrap();
    let offset = first.offset();

    append(&path, &format!("{EVENT_B}\n"));
    let mut resumed = JournalTailer::resume_at(path, offset);
    let lines = resumed.read_new().unwrap();
    assert_eq!(events(&lines).len(), 1);
}

#[test]
fn watcher_switches_to_new_journal_after_rotation() {
    let dir = temp_dir("rotation");
    let first = dir.join("Journal.2026-09-01T200000.01.log");
    append(&first, &format!("{EVENT_A}\n"));

    let mut watcher = JournalWatcher::new(dir.clone());
    assert_eq!(events(&watcher.poll().unwrap()).len(), 1);

    let second = dir.join("Journal.2026-09-01T210000.01.log");
    append(&first, &format!("{EVENT_B}\n"));
    append(&second, &format!("{EVENT_A}\n"));

    let lines = watcher.poll().unwrap();
    assert_eq!(events(&lines).len(), 2);
    assert_eq!(watcher.position().unwrap().0, second);
}

#[test]
fn latest_journal_ignores_other_files() {
    let dir = temp_dir("filter");
    fs::write(dir.join("Status.json"), "{}").unwrap();
    fs::write(dir.join("Market.json"), "{}").unwrap();
    fs::write(dir.join("Journal.2026-09-01T200000.01.log"), "").unwrap();

    let latest = latest_journal(&dir)
        .unwrap()
        .expect("журнал должен найтись");
    assert_eq!(
        latest.file_name().unwrap(),
        "Journal.2026-09-01T200000.01.log"
    );
}

#[test]
fn watcher_can_resume_from_a_known_position() {
    let dir = temp_dir("watcher-resume");
    let path = dir.join("Journal.2026-09-01T200000.01.log");
    append(&path, &format!("{EVENT_A}\n"));

    let offset = std::fs::metadata(&path).unwrap().len();
    let mut watcher = JournalWatcher::resume(dir, path.clone(), offset);
    assert!(watcher.poll().unwrap().is_empty());

    append(&path, &format!("{EVENT_B}\n"));
    assert_eq!(events(&watcher.poll().unwrap()).len(), 1);
}

#[test]
fn returning_to_a_known_journal_does_not_replay_it() {
    let dir = temp_dir("return");
    let first = dir.join("Journal.2026-09-01T200000.01.log");
    append(&first, &format!("{EVENT_A}\n"));

    let mut watcher = JournalWatcher::new(dir.clone());
    assert_eq!(events(&watcher.poll().unwrap()).len(), 1);

    let second = dir.join("Journal.2026-09-01T210000.01.log");
    append(&second, &format!("{EVENT_A}\n"));
    watcher.poll().unwrap();
    assert_eq!(watcher.position().unwrap().0, second);

    fs::remove_file(&second).unwrap();
    append(&first, &format!("{EVENT_B}\n"));

    let lines = watcher.poll().unwrap();
    assert_eq!(watcher.position().unwrap().0, first);
    assert_eq!(events(&lines).len(), 1, "первая строка уже была разобрана");
}

#[test]
fn only_files_shaped_like_a_journal_are_taken() {
    let dir = temp_dir("shape");
    fs::write(dir.join("Journal.2026-09-01T200000.01.log"), "").unwrap();
    fs::write(dir.join("Journal.backup.log"), "").unwrap();
    fs::write(dir.join("Journal.2026-09-01T220000.01.log.bak"), "").unwrap();
    fs::write(dir.join("JournalAlpha.2026-09-01T230000.01.log"), "").unwrap();
    fs::write(dir.join("JournalBeta.2026-09-01T230000.01.log"), "").unwrap();

    let latest = latest_journal(&dir)
        .unwrap()
        .expect("журнал должен найтись");
    assert_eq!(
        latest.file_name().unwrap(),
        "Journal.2026-09-01T200000.01.log"
    );
}

#[test]
fn the_old_name_format_is_still_a_journal() {
    let dir = temp_dir("old-name");
    fs::write(dir.join("Journal.200102030405.01.log"), "").unwrap();

    let latest = latest_journal(&dir)
        .unwrap()
        .expect("журнал должен найтись");
    assert_eq!(latest.file_name().unwrap(), "Journal.200102030405.01.log");
}
