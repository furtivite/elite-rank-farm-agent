use agent_core::journal::parse_line;
use agent_core::CommanderStateBuilder;

const BASELINE: &[&str] = &[
    r#"{"timestamp":"2026-09-01T20:00:00Z","event":"Commander","FID":"F1","Name":"Jameson"}"#,
    r#"{"timestamp":"2026-09-01T20:00:01Z","event":"Loadout","Ship":"diamondbackxl","MaxJumpRange":43.487682}"#,
    r#"{"timestamp":"2026-09-01T20:00:02Z","event":"Rank","Empire":7,"Federation":2}"#,
    r#"{"timestamp":"2026-09-01T20:00:03Z","event":"Progress","Empire":71,"Federation":100}"#,
    r#"{"timestamp":"2026-09-01T20:00:04Z","event":"Location","StarSystem":"Ngalinn","StarPos":[-33.65,72.46,-20.65],"Body":"Ngalinn","BodyType":"Star","Docked":false}"#,
];

fn apply(builder: &mut CommanderStateBuilder, lines: &[&str]) {
    for line in lines {
        builder.apply_line(&parse_line(line));
    }
}

fn baseline() -> CommanderStateBuilder {
    let mut builder = CommanderStateBuilder::new();
    apply(&mut builder, BASELINE);
    builder
}

const CLEAN_LEG: &[&str] = &[
    r#"{"timestamp":"2026-09-01T21:00:00Z","event":"FSDJump","StarSystem":"Canopus","StarPos":[1.0,2.0,3.0]}"#,
    r#"{"timestamp":"2026-09-01T21:03:00Z","event":"SupercruiseExit","StarSystem":"Canopus","Body":"Canopus A 1"}"#,
    r#"{"timestamp":"2026-09-01T21:03:45Z","event":"Docked","StationName":"Hammel Terminal","DistFromStarLS":1234.5}"#,
];

#[test]
fn чистая_дорога_до_станции_записывается() {
    let mut builder = baseline();
    apply(&mut builder, CLEAN_LEG);

    let state = builder.build().expect("состояние должно собраться");
    let legs = state
        .recent_supercruise
        .as_ref()
        .expect("дорога должна была записаться");

    assert_eq!(legs.len(), 1);
    assert_eq!(legs[0].distance_ls, 1234.5);
    assert_eq!(legs[0].seconds, 180);
    assert_eq!(legs[0].docking_seconds, Some(45));
    assert!(state.validate().is_empty(), "{:?}", state.validate());
}

#[test]
fn уход_обратно_в_суперкруиз_рвёт_цепочку() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            CLEAN_LEG[0],
            CLEAN_LEG[1],
            r#"{"timestamp":"2026-09-01T21:04:00Z","event":"SupercruiseEntry","StarSystem":"Canopus"}"#,
            r#"{"timestamp":"2026-09-01T21:20:00Z","event":"SupercruiseExit","StarSystem":"Canopus","Body":"Canopus A 2"}"#,
            r#"{"timestamp":"2026-09-01T21:21:00Z","event":"Docked","StationName":"Другая","DistFromStarLS":900.0}"#,
        ],
    );

    let state = builder.build().expect("состояние должно собраться");

    assert_eq!(state.recent_supercruise, None);
}

#[test]
fn стыковка_без_расстояния_замера_не_даёт() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            CLEAN_LEG[0],
            CLEAN_LEG[1],
            r#"{"timestamp":"2026-09-01T21:03:45Z","event":"Docked","StationName":"Hammel Terminal"}"#,
        ],
    );

    assert_eq!(
        builder
            .build()
            .expect("состояние должно собраться")
            .recent_supercruise,
        None,
    );
}

#[test]
fn стыковка_без_прыжка_замера_не_даёт() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            r#"{"timestamp":"2026-09-01T21:03:00Z","event":"SupercruiseExit","StarSystem":"Ngalinn","Body":"Ngalinn 1"}"#,
            r#"{"timestamp":"2026-09-01T21:03:45Z","event":"Docked","StationName":"Соседняя","DistFromStarLS":50.0}"#,
        ],
    );

    assert_eq!(
        builder
            .build()
            .expect("состояние должно собраться")
            .recent_supercruise,
        None,
    );
}

#[test]
fn долгая_посадка_не_портит_перелёт() {
    let mut builder = baseline();
    apply(
        &mut builder,
        &[
            CLEAN_LEG[0],
            CLEAN_LEG[1],
            r#"{"timestamp":"2026-09-01T23:30:00Z","event":"Docked","StationName":"Hammel Terminal","DistFromStarLS":1234.5}"#,
        ],
    );

    let legs = builder
        .build()
        .expect("состояние должно собраться")
        .recent_supercruise
        .expect("перелёт должен был записаться");

    assert_eq!(legs[0].seconds, 180);
    assert_eq!(legs[0].docking_seconds, None);
}
