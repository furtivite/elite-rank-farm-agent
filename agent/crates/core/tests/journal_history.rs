use std::fs;
use std::path::PathBuf;

use agent_core::journal::{
    newest_per_target, observations_from, parse_line, HistoryStore, JournalHistory, Observation,
    ObservationTarget, ParsedLine,
};

const DOCKED_EARLY: &str = r#"{"timestamp":"2026-01-02T20:05:00Z","event":"Docked","StationName":"Gamow Orbital",
    "StationType":"Coriolis","StarSystem":"Chaxiraxi","SystemAddress":16063849047465,"MarketID":3229583104,
    "StationFaction":{"Name":"Alliance Office of Statistics"},"StationServices":["dock","missions"]}"#;

const DOCKED_LATE: &str = r#"{"timestamp":"2026-09-02T20:05:00Z","event":"Docked","StationName":"Gamow Orbital",
    "StationType":"Coriolis","StarSystem":"Chaxiraxi","SystemAddress":16063849047465,"MarketID":3229583104,
    "StationFaction":{"Name":"Nationals of Chaxiraxi"},"StationServices":["dock","missions"]}"#;

const JUMP: &str = r#"{"timestamp":"2026-05-02T20:00:00Z","event":"FSDJump","StarSystem":"Ngalinn",
    "SystemAddress":2557887247114,"StarPos":[1,2,3],"Factions":[]}"#;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-core-history-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("создать каталог");
    dir
}

fn observation(line: &str) -> Observation {
    match parse_line(line) {
        ParsedLine::Event(event) => observations_from(&event)
            .into_iter()
            .next_back()
            .expect("событие даёт наблюдение"),
        other => panic!("строка не разобралась: {other:?}"),
    }
}

#[test]
fn missing_file_means_nothing_was_read_yet() {
    let store = HistoryStore::in_dir(&temp_dir("missing"));
    let history = store.load().expect("пустая история — не ошибка");

    assert!(history.is_empty());
    assert_eq!(history.offset_of("Journal.2026-09-06T012600.01.log"), 0);
}

#[test]
fn recorded_offsets_survive_restart() {
    let dir = temp_dir("roundtrip");
    let store = HistoryStore::in_dir(&dir);

    let mut history = JournalHistory::default();
    history.record("Journal.A.log", 120);
    history.record("Journal.B.log", 4096);
    store.store(&history).expect("записать историю");

    let loaded = HistoryStore::in_dir(&dir)
        .load()
        .expect("прочитать историю");
    assert_eq!(loaded.offset_of("Journal.A.log"), 120);
    assert_eq!(loaded.offset_of("Journal.B.log"), 4096);
    assert_eq!(loaded.offset_of("Journal.C.log"), 0);
}

#[test]
fn deleted_journals_drop_out_of_history() {
    let mut history = JournalHistory::default();
    history.record("Journal.A.log", 10);
    history.record("Journal.B.log", 20);

    history.retain_existing(&["Journal.B.log".to_owned()]);

    assert_eq!(history.offset_of("Journal.A.log"), 0);
    assert_eq!(history.offset_of("Journal.B.log"), 20);
}

#[test]
fn broken_history_file_is_an_error_not_a_silent_zero() {
    let dir = temp_dir("broken");
    let store = HistoryStore::in_dir(&dir);
    fs::write(store.path(), "не json").expect("испортить файл");

    assert!(store.load().is_err());
}

#[test]
fn repeated_visits_collapse_to_the_newest_one() {
    let collapsed = newest_per_target(vec![
        observation(DOCKED_EARLY),
        observation(JUMP),
        observation(DOCKED_LATE),
    ]);

    assert_eq!(collapsed.len(), 2);

    let station = collapsed
        .iter()
        .find(|observation| observation.target() == ObservationTarget::Station(3229583104))
        .expect("станция осталась");
    let Observation::Station { station } = station else {
        panic!("ожидалась станция");
    };
    assert_eq!(station.controlling_faction.name, "Nationals of Chaxiraxi");
}

#[test]
fn collapsed_observations_go_oldest_first() {
    let collapsed = newest_per_target(vec![observation(DOCKED_LATE), observation(JUMP)]);

    let times: Vec<_> = collapsed
        .iter()
        .map(|observation| observation.observed_at())
        .collect();

    assert!(times[0] < times[1], "порядок должен быть от старых к новым");
}

#[test]
fn different_stations_are_not_collapsed_into_one() {
    let other = r#"{"timestamp":"2026-09-02T21:05:00Z","event":"Docked","StationName":"Hickam Survey",
        "StationType":"CraterOutpost","StarSystem":"Ngalinn","SystemAddress":2557887247114,"MarketID":128882222,
        "StationFaction":{"Name":"Nationals of Ngalinn"},"StationServices":["dock"]}"#;

    let collapsed = newest_per_target(vec![observation(DOCKED_LATE), observation(other)]);

    assert_eq!(collapsed.len(), 2);
}
