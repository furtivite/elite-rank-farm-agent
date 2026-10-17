mod common;

use common::{apply, baseline};

#[test]
fn promotion_updates_rank_and_zeroes_progress() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Promotion","Empire":8}"#],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 8);
    assert_eq!(state.imperial_progress, 0.0);
    assert_eq!(state.federal_rank, 2);
    assert_eq!(state.federal_progress, 100.0);
}

#[test]
fn progress_carries_the_moment_it_was_written() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T22:30:00Z","event":"Docked","StationName":"Archimedes City","StationType":"Coriolis","StarSystem":"Ngalinn"}"#,
        ],
    );

    let state = builder.build().unwrap();
    assert_eq!(
        state.imperial_progress_at.to_rfc3339(),
        "2026-09-01T20:00:03+00:00"
    );
    assert_eq!(state.federal_progress_at, state.imperial_progress_at);
    assert!(state.observed_at > state.imperial_progress_at);
}

#[test]
fn promotion_moves_the_progress_moment_to_the_promotion() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Promotion","Empire":8}"#],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_progress, 0.0);
    assert_eq!(
        state.imperial_progress_at.to_rfc3339(),
        "2026-09-02T10:00:00+00:00"
    );
    assert_eq!(
        state.federal_progress_at.to_rfc3339(),
        "2026-09-01T20:00:03+00:00"
    );
}

#[test]
fn rank_event_resets_progress_only_when_rank_changed() {
    let mut builder = baseline();

    apply(
        &mut builder,
        &[r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Rank","Empire":7,"Federation":2}"#],
    );
    assert_eq!(builder.build().unwrap().imperial_progress, 71.0);

    apply(
        &mut builder,
        &[r#"{"timestamp":"2026-09-02T11:00:00Z","event":"Rank","Empire":8,"Federation":2}"#],
    );
    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 8);
    assert_eq!(state.imperial_progress, 0.0);
}

#[test]
fn progress_right_after_rank_wins_over_the_reset() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Rank","Empire":8,"Federation":2}"#,
            r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Progress","Empire":12,"Federation":100}"#,
        ],
    );

    assert_eq!(builder.build().unwrap().imperial_progress, 12.0);
}

#[test]
fn older_events_do_not_overwrite_newer_ranks() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-02T12:00:00Z","event":"Promotion","Empire":8}"#,
            r#"{"timestamp":"2026-01-01T00:00:00Z","event":"Rank","Empire":3,"Federation":1}"#,
        ],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 8);
    assert_eq!(state.federal_rank, 2);
}

#[test]
fn later_session_reporting_the_old_rank_does_not_undo_a_promotion() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T18:13:57Z","event":"Rank","Empire":7,"Federation":2}"#,
            r#"{"timestamp":"2026-09-01T18:13:57Z","event":"Progress","Empire":71,"Federation":100}"#,
            r#"{"timestamp":"2026-09-01T20:32:19Z","event":"Promotion","Empire":8}"#,
            r#"{"timestamp":"2026-09-02T09:00:05Z","event":"Rank","Empire":7,"Federation":2}"#,
            r#"{"timestamp":"2026-09-02T09:00:05Z","event":"Progress","Empire":71,"Federation":100}"#,
        ],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 8);
    assert_eq!(state.imperial_progress, 0.0);
    assert_eq!(state.federal_rank, 2);
    assert_eq!(state.federal_progress, 100.0);
}

#[test]
fn progress_of_the_same_rank_is_still_accepted_after_a_promotion() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:32:19Z","event":"Promotion","Empire":8}"#,
            r#"{"timestamp":"2026-09-02T09:00:05Z","event":"Rank","Empire":8,"Federation":2}"#,
            r#"{"timestamp":"2026-09-02T09:00:05Z","event":"Progress","Empire":12,"Federation":100}"#,
        ],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 8);
    assert_eq!(state.imperial_progress, 12.0);
}

#[test]
fn promotion_never_lowers_a_known_rank() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T20:32:19Z","event":"Promotion","Empire":8}"#,
            r#"{"timestamp":"2026-09-02T09:00:05Z","event":"Promotion","Empire":6}"#,
        ],
    );

    assert_eq!(builder.build().unwrap().imperial_rank, 8);
}

const NEW_GAME: &[&str] = &[
    r#"{ "timestamp":"2026-09-03T06:16:37Z", "event":"NewCommander", "FID":"F1", "Name":"Jameson" }"#,
    r#"{ "timestamp":"2026-09-03T06:16:39Z", "event":"Commander", "FID":"F1", "Name":"Jameson" }"#,
    r#"{ "timestamp":"2026-09-03T06:16:39Z", "event":"Rank", "Combat":0, "Trade":0, "Explore":0, "Soldier":0, "Exobiologist":0, "Empire":0, "Federation":0, "CQC":0 }"#,
    r#"{ "timestamp":"2026-09-03T06:16:39Z", "event":"Progress", "Combat":0, "Trade":0, "Explore":0, "Soldier":0, "Exobiologist":0, "Empire":0, "Federation":0, "CQC":0 }"#,
];

#[test]
fn new_game_drops_ranks_of_the_previous_one() {
    let mut builder = baseline();
    apply(&mut builder, NEW_GAME);

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 0);
    assert_eq!(state.imperial_progress, 0.0);
    assert_eq!(state.federal_rank, 0);
    assert_eq!(state.federal_progress, 0.0);
}

#[test]
fn previous_game_read_after_the_reset_does_not_bring_its_ranks_back() {
    let mut builder = baseline();
    apply(&mut builder, NEW_GAME);
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Rank","Empire":12,"Federation":9}"#,
            r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Progress","Empire":40,"Federation":40}"#,
            r#"{"timestamp":"2026-09-02T11:00:00Z","event":"Promotion","Empire":13}"#,
        ],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 0);
    assert_eq!(state.imperial_progress, 0.0);
    assert_eq!(state.federal_rank, 0);
}

#[test]
fn ranks_still_do_not_drop_within_the_new_game() {
    let mut builder = baseline();
    apply(&mut builder, NEW_GAME);
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-04T10:00:00Z","event":"Promotion","Federation":1}"#,
            r#"{"timestamp":"2026-09-05T10:00:00Z","event":"Rank","Empire":0,"Federation":0}"#,
        ],
    );

    assert_eq!(builder.build().unwrap().federal_rank, 1);
}

#[test]
fn cleared_save_resets_ranks_and_fines() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-02T09:00:00Z","event":"Docked","StationName":"A","StationFaction":{"Name":"Faction A"},"ActiveFine":true}"#,
            r#"{"timestamp":"2026-09-02T10:00:00Z","event":"ClearSavedGame","FID":"F1","Name":"Jameson"}"#,
        ],
    );

    let state = builder.build().unwrap();
    assert_eq!(state.imperial_rank, 0);
    assert_eq!(state.federal_rank, 0);
    assert!(state.fined_factions.is_empty());
}
