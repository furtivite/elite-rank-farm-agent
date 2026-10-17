use agent_core::journal::{parse_line, JournalEvent, ParsedLine};

fn event(line: &str) -> JournalEvent {
    match parse_line(line) {
        ParsedLine::Event(parsed) => parsed.event,
        other => panic!("ожидалось событие, получено {other:?}"),
    }
}

#[test]
fn fsd_jump_gives_system_and_coordinates() {
    let parsed = parse_line(
        r#"{"timestamp":"2026-09-02T10:00:00Z","event":"FSDJump","StarSystem":"Ngalinn","StarPos":[-33.65,72.46,-20.65],"JumpDist":12.3}"#,
    );
    let ParsedLine::Event(line) = parsed else {
        panic!("ожидалось событие");
    };
    assert_eq!(line.timestamp.to_rfc3339(), "2026-09-02T10:00:00+00:00");
    let JournalEvent::FsdJump {
        star_system,
        star_pos,
        body,
        ..
    } = line.event
    else {
        panic!("ожидался FSDJump");
    };
    assert_eq!(star_system, "Ngalinn");
    assert_eq!(star_pos, [-33.65, 72.46, -20.65]);
    assert_eq!(body, None);
}

#[test]
fn location_keeps_station_only_when_docked() {
    let docked = event(
        r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Location","StarSystem":"Ngalinn","StarPos":[0.0,0.0,0.0],"Docked":true,"StationName":"Miller Depot","Body":"Ngalinn A 1"}"#,
    );
    let JournalEvent::Location {
        star_system,
        star_pos,
        body,
        docked: is_docked,
        station_name,
        ..
    } = docked
    else {
        panic!("ожидался Location");
    };
    assert_eq!(star_system, "Ngalinn");
    assert_eq!(star_pos, [0.0, 0.0, 0.0]);
    assert_eq!(body.as_deref(), Some("Ngalinn A 1"));
    assert!(is_docked);
    assert_eq!(station_name.as_deref(), Some("Miller Depot"));

    let undocked = event(
        r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Location","StarSystem":"Ngalinn","StarPos":[0.0,0.0,0.0],"Docked":false}"#,
    );
    let JournalEvent::Location {
        docked,
        station_name,
        ..
    } = undocked
    else {
        panic!("ожидался Location");
    };
    assert!(!docked);
    assert_eq!(station_name, None);
}

#[test]
fn rank_and_progress_read_only_empire_and_federation() {
    assert_eq!(
        event(
            r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Rank","Combat":3,"Trade":4,"Explore":2,"Empire":7,"Federation":5,"CQC":0}"#
        ),
        JournalEvent::Rank {
            empire: 7,
            federation: 5
        }
    );
    assert_eq!(
        event(
            r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Progress","Combat":10,"Trade":20,"Explore":30,"Empire":64,"Federation":12,"CQC":0}"#
        ),
        JournalEvent::Progress {
            empire: 64.0,
            federation: 12.0
        }
    );
}

#[test]
fn loadout_gives_ship_type_and_jump_range() {
    let parsed = event(
        r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Loadout","Ship":"empire_courier","ShipID":12,"ShipName":"blue","MaxJumpRange":38.42,"Modules":[]}"#,
    );

    let JournalEvent::Loadout {
        ship,
        max_jump_range,
        unladen_mass,
        ..
    } = parsed
    else {
        panic!("ожидался Loadout");
    };

    assert_eq!(ship, "empire_courier");
    assert_eq!(max_jump_range, 38.42);
    assert_eq!(unladen_mass, None);
}

#[test]
fn loadout_gives_mass_fuel_and_drive() {
    let parsed = event(
        r#"{"timestamp":"2026-09-06T15:57:31Z","event":"Loadout","Ship":"type9","MaxJumpRange":39.825539,"UnladenMass":1256.199951,"CargoCapacity":724,"FuelCapacity":{"Main":64.0,"Reserve":0.77},"Modules":[{"Slot":"FrameShiftDrive","Item":"int_hyperdrive_overcharge_size6_class5","On":true,"Priority":2,"Engineering":{"Engineer":"Felicity Farseer","BlueprintName":"FSD_LongRange","Level":5,"Modifiers":[{"Label":"Mass","Value":52.0,"OriginalValue":40.0},{"Label":"FSDOptimalMass","Value":3224.0,"OriginalValue":2000.0}]}},{"Slot":"Slot05_Size4","Item":"int_guardianfsdbooster_size4","On":true,"Priority":2}]}"#,
    );

    let JournalEvent::Loadout {
        unladen_mass,
        cargo_capacity,
        fuel_capacity,
        modules,
        ..
    } = parsed
    else {
        panic!("ожидался Loadout");
    };

    assert_eq!(unladen_mass, Some(1256.199951));
    assert_eq!(cargo_capacity, Some(724));
    assert_eq!(fuel_capacity.and_then(|fuel| fuel.main), Some(64.0));
    assert_eq!(modules.len(), 2);

    let drive = &modules[0];
    assert_eq!(drive.slot, "FrameShiftDrive");
    assert_eq!(drive.item, "int_hyperdrive_overcharge_size6_class5");

    let optimal = drive
        .engineering
        .as_ref()
        .and_then(|engineering| {
            engineering
                .modifiers
                .iter()
                .find(|modifier| modifier.label == "FSDOptimalMass")
        })
        .and_then(|modifier| modifier.value);
    assert_eq!(optimal, Some(3224.0));
}

#[test]
fn cargo_and_fuel_are_parsed() {
    assert_eq!(
        event(
            r#"{"timestamp":"2026-09-06T15:52:32Z","event":"Cargo","Vessel":"Ship","Count":312,"Inventory":[]}"#
        ),
        JournalEvent::Cargo {
            vessel: Some("Ship".to_owned()),
            count: Some(312),
        }
    );

    assert_eq!(
        event(
            r#"{"timestamp":"2026-09-06T16:18:25Z","event":"FuelScoop","Scooped":1.445575,"Total":58.099022}"#
        ),
        JournalEvent::FuelScoop {
            total: Some(58.099022),
        }
    );
}

#[test]
fn unknown_events_are_ignored_not_failed() {
    assert_eq!(
        parse_line(r#"{"timestamp":"2026-09-02T10:00:00Z","event":"ShipLocker","Items":[]}"#),
        ParsedLine::Ignored
    );
    assert_eq!(parse_line("   "), ParsedLine::Ignored);
}

#[test]
fn unknown_fields_in_known_events_do_not_break_parsing() {
    let JournalEvent::Docked {
        station_name,
        star_system,
        station_type,
        ..
    } = event(
        r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Docked","StationName":"Miller Depot","StationType":"Coriolis","SomeFieldFromFutureUpdate":{"a":1}}"#,
    )
    else {
        panic!("ожидался Docked");
    };
    assert_eq!(station_name, "Miller Depot");
    assert_eq!(star_system, None);
    assert_eq!(station_type.as_deref(), Some("Coriolis"));
}

#[test]
fn truncated_line_is_reported_not_dropped_silently() {
    let parsed = parse_line(r#"{"timestamp":"2026-09-02T10:00:00Z","event":"FSDJump","Star"#);
    assert!(matches!(parsed, ParsedLine::Unparsed { .. }));
}

#[test]
fn promotion_carries_only_the_changed_rank() {
    assert_eq!(
        event(r#"{"timestamp":"2026-01-07T13:52:11Z","event":"Promotion","Federation":2}"#),
        JournalEvent::Promotion {
            empire: None,
            federation: Some(2),
        }
    );
    assert_eq!(
        event(r#"{"timestamp":"2026-09-02T10:00:00Z","event":"Promotion","Empire":8}"#),
        JournalEvent::Promotion {
            empire: Some(8),
            federation: None,
        }
    );
    assert_eq!(
        event(r#"{"timestamp":"2026-01-07T11:44:27Z","event":"Promotion","Trade":5}"#),
        JournalEvent::Promotion {
            empire: None,
            federation: None,
        }
    );
}
