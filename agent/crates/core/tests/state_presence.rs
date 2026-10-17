mod common;

use agent_core::journal::parse_line;
use agent_core::PresenceStatus;
use chrono::{TimeZone, Utc};
use common::{apply, baseline};

#[test]
fn without_a_signal_presence_stays_unknown() {
    assert!(baseline()
        .build()
        .expect("состояние должно собраться")
        .presence
        .is_none());
}

#[test]
fn load_game_means_the_game_is_running() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[r#"{"timestamp":"2026-09-06T01:26:00Z","event":"LoadGame","Commander":"Jameson"}"#],
    );

    let presence = builder
        .build()
        .expect("состояние должно собраться")
        .presence
        .expect("вход в игру виден");

    assert_eq!(presence.status, PresenceStatus::Playing);
    assert_eq!(
        presence.at,
        Utc.with_ymd_and_hms(2026, 9, 6, 1, 26, 0).unwrap()
    );
}

#[test]
fn shutdown_means_the_game_is_closed() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-06T01:26:00Z","event":"LoadGame","Commander":"Jameson"}"#,
            r#"{"timestamp":"2026-09-06T04:10:00Z","event":"Shutdown"}"#,
        ],
    );

    let presence = builder
        .build()
        .expect("состояние должно собраться")
        .presence
        .expect("выход из игры виден");

    assert_eq!(presence.status, PresenceStatus::GameExited);
}

#[test]
fn entering_the_game_again_overrides_the_previous_exit() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-06T04:10:00Z","event":"Shutdown"}"#,
            r#"{"timestamp":"2026-09-06T04:20:00Z","event":"LoadGame","Commander":"Jameson"}"#,
        ],
    );

    let presence = builder
        .build()
        .expect("состояние должно собраться")
        .presence
        .expect("вход в игру виден");

    assert_eq!(presence.status, PresenceStatus::Playing);
}

#[test]
fn agent_stop_is_marked_only_for_the_last_send() {
    let builder = baseline();
    let at = Utc.with_ymd_and_hms(2026, 9, 6, 5, 0, 0).unwrap();

    let stopping = builder
        .build_stopping(at)
        .expect("состояние должно собраться");

    assert_eq!(
        stopping.presence.expect("остановка отмечена").status,
        PresenceStatus::AgentStopped
    );
    assert!(builder.build().expect("состояние").presence.is_none());
}

#[test]
fn recent_systems_are_collected_newest_first() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:10:00Z","event":"FSDJump","StarSystem":"Canopus","StarPos":[0.0,0.0,0.0]}"#,
            r#"{"timestamp":"2026-09-01T20:20:00Z","event":"FSDJump","StarSystem":"Ngalinn","StarPos":[-33.65,72.46,-20.65]}"#,
        ],
    );

    let state = builder.build().expect("состояние должно собраться");
    let recent = state.recent_systems.clone().expect("история должна быть");

    let names: Vec<&str> = recent.iter().map(|visit| visit.name.as_str()).collect();
    assert_eq!(names, vec!["Ngalinn", "Canopus", "Ngalinn"]);
    assert!(state.validate().is_empty(), "{:?}", state.validate());
}

#[test]
fn staying_in_the_same_system_does_not_fill_the_history() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:05:00Z","event":"Location","StarSystem":"Ngalinn","StarPos":[-33.65,72.46,-20.65],"Docked":true,"StationName":"Hickam Survey"}"#,
            r#"{"timestamp":"2026-09-01T20:06:00Z","event":"Location","StarSystem":"Ngalinn","StarPos":[-33.65,72.46,-20.65],"Docked":false}"#,
        ],
    );

    let state = builder.build().expect("состояние должно собраться");
    let recent = state.recent_systems.expect("история должна быть");

    assert_eq!(recent.len(), 1);
    assert_eq!(
        recent[0].at,
        Utc.with_ymd_and_hms(2026, 9, 1, 20, 6, 0).unwrap(),
        "время верхней записи обновляется: «мы всё ещё здесь» это факт о сейчас",
    );
}

#[test]
fn history_stops_at_the_contract_limit() {
    let mut builder = baseline();
    for index in 0..20 {
        let line = format!(
            r#"{{"timestamp":"2026-09-01T21:00:00Z","event":"FSDJump","StarSystem":"Система {index}","StarPos":[0.0,0.0,0.0]}}"#
        );
        builder.apply_line(&parse_line(&line));
    }

    let state = builder.build().expect("состояние должно собраться");
    let recent = state.recent_systems.expect("история должна быть");

    assert_eq!(recent.len(), agent_core::RECENT_SYSTEMS_LIMIT);
    assert_eq!(recent[0].name, "Система 19");
}

#[test]
fn approaching_a_body_replaces_the_star_with_it() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:10:00Z","event":"ApproachBody","StarSystem":"Ngalinn","Body":"Ngalinn A 3"}"#,
        ],
    );

    assert_eq!(
        builder.build().unwrap().current_body.as_deref(),
        Some("Ngalinn A 3")
    );
}

#[test]
fn touchdown_names_the_body_too() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:12:00Z","event":"Touchdown","PlayerControlled":true,"Body":"Ngalinn A 3","Latitude":1.5,"Longitude":2.5}"#,
        ],
    );

    assert_eq!(
        builder.build().unwrap().current_body.as_deref(),
        Some("Ngalinn A 3")
    );
}

#[test]
fn touchdown_without_a_body_keeps_what_was_known() {
    let mut builder = baseline();
    let before = builder.build().unwrap().current_body.clone();
    apply(
        &mut builder,
        &[r#"{"timestamp":"2026-09-01T20:12:00Z","event":"Touchdown","PlayerControlled":true}"#],
    );

    assert_eq!(builder.build().unwrap().current_body, before);
}

#[test]
fn leaving_a_body_clears_it() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:10:00Z","event":"ApproachBody","StarSystem":"Ngalinn","Body":"Ngalinn A 3"}"#,
            r#"{"timestamp":"2026-09-01T20:30:00Z","event":"LeaveBody","StarSystem":"Ngalinn","Body":"Ngalinn A 3"}"#,
        ],
    );

    assert_eq!(builder.build().unwrap().current_body, None);
}

#[test]
fn liftoff_alone_keeps_the_body() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:10:00Z","event":"ApproachBody","StarSystem":"Ngalinn","Body":"Ngalinn A 3"}"#,
            r#"{"timestamp":"2026-09-01T20:20:00Z","event":"Liftoff","PlayerControlled":true,"Latitude":1.5,"Longitude":2.5}"#,
        ],
    );

    assert_eq!(
        builder.build().unwrap().current_body.as_deref(),
        Some("Ngalinn A 3")
    );
}
