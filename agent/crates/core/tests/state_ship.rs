mod common;

use common::{apply, baseline};

#[test]
fn required_pad_size_comes_from_ship_type() {
    let state = baseline().build().unwrap();
    assert_eq!(state.ship, "diamondbackxl");
    assert_eq!(state.required_pad_size, Some(agent_core::PadSize::S));
    assert!(state.validate().is_empty(), "{:?}", state.validate());
}

#[test]
fn swapping_ships_updates_required_pad_size() {
    let mut builder = baseline();
    assert_eq!(
        builder.build().unwrap().required_pad_size,
        Some(agent_core::PadSize::S)
    );

    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T22:10:00Z","event":"Loadout","Ship":"Anaconda","MaxJumpRange":61.2}"#,
        ],
    );
    let state = builder.build().unwrap();
    assert_eq!(state.ship, "Anaconda");
    assert_eq!(state.required_pad_size, Some(agent_core::PadSize::L));
    assert_eq!(state.jump_range_ly, 61.2);

    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T22:40:00Z","event":"Loadout","Ship":"Python_NX","MaxJumpRange":38.9}"#,
        ],
    );
    assert_eq!(
        builder.build().unwrap().required_pad_size,
        Some(agent_core::PadSize::M)
    );
}

#[test]
fn unknown_ship_leaves_pad_size_null_and_state_still_valid() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T23:00:00Z","event":"Loadout","Ship":"SomeShipFromNextUpdate","MaxJumpRange":30.0}"#,
        ],
    );
    let state = builder.build().unwrap();
    assert_eq!(state.required_pad_size, None);
    assert!(state.validate().is_empty(), "{:?}", state.validate());
}

#[test]
fn active_fine_marks_faction_and_all_fines_clears_it() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T16:47:04Z","event":"Docked","StationName":"Zenbei Enterprise","StationFaction":{"Name":"Nationals of HIP 6108"},"ActiveFine":true}"#,
        ],
    );

    let state = builder.build().expect("состояние должно собраться");
    assert_eq!(
        state.fined_factions,
        vec!["Nationals of HIP 6108".to_owned()]
    );

    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T16:47:26Z","event":"PayFines","Amount":1365610,"AllFines":true}"#,
        ],
    );

    let state = builder.build().expect("состояние должно собраться");
    assert!(
        state.fined_factions.is_empty(),
        "AllFines гасит все штрафы разом: фракции в такой записи игра не пишет"
    );
}

#[test]
fn paying_one_faction_keeps_the_others() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T10:00:00Z","event":"Docked","StationName":"A","StationFaction":{"Name":"Faction A"},"ActiveFine":true}"#,
            r#"{"timestamp":"2026-09-01T11:00:00Z","event":"Docked","StationName":"B","StationFaction":{"Name":"Faction B"},"ActiveFine":true}"#,
            r#"{"timestamp":"2026-09-01T12:00:00Z","event":"PayFines","Amount":100,"Faction":"Faction A","AllFines":false}"#,
        ],
    );

    let state = builder.build().expect("состояние должно собраться");
    assert_eq!(state.fined_factions, vec!["Faction B".to_owned()]);
}

#[test]
fn docking_without_active_fine_marks_nothing() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T10:00:00Z","event":"Docked","StationName":"A","StationFaction":{"Name":"Faction A"}}"#,
        ],
    );

    assert!(builder
        .build()
        .expect("состояние должно собраться")
        .fined_factions
        .is_empty());
}
