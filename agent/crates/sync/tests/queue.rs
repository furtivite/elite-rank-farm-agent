use agent_core::{CommanderState, SystemPosition, AGENT_VERSION, SCHEMA_VERSION};
use agent_sync::PendingState;
use chrono::{TimeZone, Utc};

fn state(system: &str) -> CommanderState {
    CommanderState {
        schema_version: SCHEMA_VERSION,
        agent_version: AGENT_VERSION.to_owned(),
        fined_factions: Vec::new(),
        commander_name: "Jameson".to_owned(),
        current_system: SystemPosition {
            name: system.to_owned(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        current_station: None,
        current_body: None,
        ship: "diamondbackxl".to_owned(),
        required_pad_size: None,
        jump_range_ly: 43.5,
        imperial_rank: 7,
        imperial_progress: 71.0,
        imperial_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        federal_rank: 2,
        federal_progress: 100.0,
        federal_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        observed_at: Utc.with_ymd_and_hms(2026, 9, 1, 22, 16, 9).unwrap(),
        presence: None,
        loadout: None,
        recent_systems: None,
        recent_supercruise: None,
    }
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-sync-queue-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn empty_queue_returns_nothing() {
    let pending = PendingState::in_dir(&temp_dir("empty"));
    assert_eq!(pending.load().unwrap(), None);
}

#[test]
fn stored_state_survives_a_new_process() {
    let dir = temp_dir("roundtrip");
    PendingState::in_dir(&dir).store(&state("Ngalinn")).unwrap();

    let restored = PendingState::in_dir(&dir).load().unwrap();
    assert_eq!(restored, Some(state("Ngalinn")));
}

#[test]
fn newer_state_replaces_the_older_one() {
    let dir = temp_dir("replace");
    let pending = PendingState::in_dir(&dir);
    pending.store(&state("Ngalinn")).unwrap();
    pending.store(&state("Mainani")).unwrap();

    assert_eq!(
        pending.load().unwrap().unwrap().current_system.name,
        "Mainani"
    );
}

#[test]
fn clear_removes_the_pending_state_and_is_idempotent() {
    let dir = temp_dir("clear");
    let pending = PendingState::in_dir(&dir);
    pending.store(&state("Ngalinn")).unwrap();

    pending.clear().unwrap();
    assert_eq!(pending.load().unwrap(), None);
    pending.clear().unwrap();
}

#[test]
fn corrupted_pending_file_is_reported_not_silently_dropped() {
    let dir = temp_dir("corrupt");
    let pending = PendingState::in_dir(&dir);
    std::fs::write(pending.path(), "{ обрезано").unwrap();

    assert!(pending.load().is_err());
}

#[test]
fn no_temporary_file_is_left_behind() {
    let dir = temp_dir("tmpfile");
    PendingState::in_dir(&dir).store(&state("Ngalinn")).unwrap();

    let names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["pending-state.json".to_owned()]);
}
