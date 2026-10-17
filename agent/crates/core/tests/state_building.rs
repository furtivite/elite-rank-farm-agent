mod common;

use agent_core::journal::ParsedLine;
use agent_core::{CommanderStateBuilder, MissingField};
use common::{apply, baseline, BASELINE};

#[test]
fn baseline_stream_produces_valid_state() {
    let state = baseline().build().expect("состояние должно собраться");

    assert_eq!(state.commander_name, "Jameson");
    assert_eq!(state.current_system.name, "Ngalinn");
    assert_eq!(state.current_system.x, -33.65);
    assert_eq!(state.ship, "diamondbackxl");
    assert_eq!(state.jump_range_ly, 43.487682);
    assert_eq!(state.imperial_rank, 7);
    assert_eq!(state.federal_progress, 100.0);
    assert_eq!(state.current_station, None);
    assert!(state.validate().is_empty(), "{:?}", state.validate());
}

#[test]
fn incomplete_stream_reports_what_is_missing() {
    let mut builder = CommanderStateBuilder::new();
    apply(&mut builder, &BASELINE[..1]);

    assert_eq!(builder.build(), None);
    let missing = builder.missing();
    assert!(!missing.contains(&MissingField::CommanderName));
    assert!(missing.contains(&MissingField::Ship));
    assert_eq!(MissingField::Ship.filled_by(), "Loadout");
}

#[test]
fn docking_and_undocking_track_current_station() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T21:59:03Z","event":"Docked","StationName":"Archimedes City","StationType":"Coriolis","StarSystem":"Ngalinn"}"#,
        ],
    );
    assert_eq!(
        builder.build().unwrap().current_station.as_deref(),
        Some("Archimedes City")
    );

    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T21:59:48Z","event":"Undocked","StationName":"Archimedes City"}"#,
        ],
    );
    assert_eq!(builder.build().unwrap().current_station, None);
}

#[test]
fn jump_clears_previous_station() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T22:08:35Z","event":"Docked","StationName":"Kandel Sanctuary","StationType":"Outpost"}"#,
            r#"{"timestamp":"2026-09-01T22:11:49Z","event":"FSDJump","StarSystem":"Mainani","StarPos":[10.40625,-219.8125,-64.21875],"Body":"Mainani","BodyType":"Star"}"#,
        ],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.current_system.name, "Mainani");
    assert_eq!(state.current_station, None);
}

#[test]
fn observed_at_follows_last_relevant_event() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[r#"{"timestamp":"2026-09-01T23:00:00Z","event":"Music","MusicTrack":"NoTrack"}"#],
    );
    assert_eq!(
        builder.build().unwrap().observed_at.to_rfc3339(),
        "2026-09-01T20:00:04+00:00"
    );
}

#[test]
fn unparsed_lines_do_not_corrupt_state() {
    let mut builder = baseline();
    let before = builder.build().unwrap();
    builder.apply_line(&ParsedLine::Unparsed {
        reason: "оборванный хвост".to_owned(),
    });
    assert_eq!(builder.build().unwrap(), before);
}
