mod common;

use common::{apply, baseline};

#[test]
fn card_names_commander_ship_suit_system_and_station() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{ "timestamp":"2026-09-01T20:01:00Z", "event":"LoadGame", "Commander":"Jameson", "Ship":"DiamondbackXL", "Ship_Localised":"Diamondback Explorer", "GameMode":"Solo" }"#,
            r#"{ "timestamp":"2026-09-01T20:01:01Z", "event":"SuitLoadout", "SuitID":1853548891076818, "SuitName":"utilitysuit_class1", "SuitName_Localised":"Комбинезон Maverick", "LoadoutName":"Maveric Base" }"#,
            r#"{"timestamp":"2026-09-01T20:02:00Z","event":"Docked","StationName":"Hickam Survey","StationType":"Coriolis","StarSystem":"Ngalinn"}"#,
        ],
    );

    let card = builder.card();
    assert_eq!(card.cmdr.as_deref(), Some("Jameson"));
    assert_eq!(card.ship.as_deref(), Some("Diamondback Explorer"));
    assert_eq!(card.suit.as_deref(), Some("Комбинезон Maverick"));
    assert_eq!(card.system.as_deref(), Some("Ngalinn"));
    assert_eq!(card.station.as_deref(), Some("Hickam Survey"));
}

#[test]
fn untranslated_suit_name_falls_back_to_the_internal_one() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{ "timestamp":"2026-09-01T20:01:00Z", "event":"SwitchSuitLoadout", "SuitName":"explorationsuit_class3", "SuitName_Localised":"$ExplorationSuit_Class1_Name;" }"#,
        ],
    );

    assert_eq!(
        builder.card().suit.as_deref(),
        Some("explorationsuit_class3")
    );
}

#[test]
fn another_ship_is_not_called_by_the_previous_name() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{ "timestamp":"2026-09-01T20:01:00Z", "event":"LoadGame", "Ship":"DiamondbackXL", "Ship_Localised":"Diamondback Explorer", "GameMode":"Solo" }"#,
            r#"{"timestamp":"2026-09-01T21:00:00Z","event":"Loadout","Ship":"panthermkii","MaxJumpRange":20.5}"#,
        ],
    );

    assert_eq!(builder.card().ship.as_deref(), Some("panthermkii"));
}

#[test]
fn card_does_not_move_the_state_moment() {
    let mut builder = baseline();
    let before = builder.build().unwrap().observed_at;
    apply(
        &mut builder,
        &[
            r#"{ "timestamp":"2026-09-02T10:00:00Z", "event":"SuitLoadout", "SuitName":"flightsuit", "SuitName_Localised":"Летный комбинезон" }"#,
        ],
    );

    assert_eq!(builder.build().unwrap().observed_at, before);
}
