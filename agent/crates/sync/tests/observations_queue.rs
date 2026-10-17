use agent_core::journal::{observations_from, parse_line, Observation, ParsedLine};
use agent_sync::ObservationQueue;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-sync-observations-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn observation(system: &str, address: u64) -> Observation {
    let line = format!(
        r#"{{"timestamp":"2026-09-02T20:00:00Z","event":"FSDJump","StarSystem":"{system}",
        "SystemAddress":{address},"StarPos":[1,2,3],"Factions":[]}}"#
    );
    let ParsedLine::Event(parsed) = parse_line(&line) else {
        panic!("ожидалось событие");
    };
    observations_from(&parsed).remove(0)
}

#[test]
fn empty_queue_reads_as_nothing() {
    let queue = ObservationQueue::in_dir(&temp_dir("empty"));
    assert!(queue.load().unwrap().is_empty());
}

#[test]
fn appended_observations_survive_reload() {
    let queue = ObservationQueue::in_dir(&temp_dir("roundtrip"));
    let batch = vec![observation("Ngalinn", 1), observation("Mainani", 2)];

    queue.append(&batch).unwrap();
    assert_eq!(queue.load().unwrap(), batch);
}

#[test]
fn append_adds_to_the_end_instead_of_replacing() {
    let queue = ObservationQueue::in_dir(&temp_dir("append"));

    queue.append(&[observation("Первая", 1)]).unwrap();
    queue.append(&[observation("Вторая", 2)]).unwrap();

    let loaded = queue.load().unwrap();
    assert_eq!(loaded.len(), 2);
    let Observation::System { system } = &loaded[1] else {
        panic!("ожидалось наблюдение системы");
    };
    assert_eq!(system.star_system, "Вторая");
}

#[test]
fn broken_line_costs_one_observation_not_the_whole_queue() {
    let dir = temp_dir("broken");
    let queue = ObservationQueue::in_dir(&dir);
    queue.append(&[observation("Целая", 1)]).unwrap();

    let mut text = std::fs::read_to_string(queue.path()).unwrap();
    text.push_str("{\"type\":\"system\",\"system\":{\"schemaVer");
    std::fs::write(queue.path(), text).unwrap();

    let loaded = queue.load().unwrap();
    assert_eq!(loaded.len(), 1);
}

#[test]
fn replace_keeps_only_what_was_not_sent() {
    let queue = ObservationQueue::in_dir(&temp_dir("replace"));
    queue
        .append(&[observation("Отправлено", 1), observation("Осталось", 2)])
        .unwrap();

    let rest = vec![observation("Осталось", 2)];
    queue.replace(&rest).unwrap();

    assert_eq!(queue.load().unwrap(), rest);
    assert!(!queue.path().with_extension("jsonl.tmp").exists());
}

#[test]
fn replacing_with_nothing_removes_the_file() {
    let dir = temp_dir("drain");
    let queue = ObservationQueue::in_dir(&dir);
    queue.append(&[observation("Одна", 1)]).unwrap();

    queue.replace(&[]).unwrap();

    assert!(queue.load().unwrap().is_empty());
    assert!(!queue.path().exists());
}

#[test]
fn overflowing_queue_keeps_the_freshest() {
    let queue = ObservationQueue::in_dir(&temp_dir("overflow"));
    let batch: Vec<Observation> = (0..agent_sync::MAX_QUEUED as u64 + 10)
        .map(|index| observation("S", index + 1))
        .collect();

    queue.append(&batch).unwrap();

    let loaded = queue.load().unwrap();
    assert_eq!(loaded.len(), agent_sync::MAX_QUEUED);
    let Observation::System { system } = loaded.last().unwrap() else {
        panic!("ожидалось наблюдение системы");
    };
    assert_eq!(system.system_address, agent_sync::MAX_QUEUED as u64 + 10);
}
