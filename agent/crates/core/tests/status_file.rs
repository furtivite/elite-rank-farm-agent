use agent_core::journal::read_status;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-core-status-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("создать каталог");
    dir
}

fn dir_with(name: &str, contents: &str) -> PathBuf {
    let dir = temp_dir(name);
    fs::write(dir.join("Status.json"), contents).expect("запись Status.json");
    dir
}

#[test]
fn reads_fuel_and_cargo_in_flight() {
    let dir = dir_with(
        "flight",
        r#"{"timestamp":"2026-09-08T12:00:00Z","event":"Status","Flags":16777240,"Pips":[4,4,4],
            "FireGroup":0,"GuiFocus":0,"Fuel":{"FuelMain":42.815,"FuelReservoir":0.63},
            "Cargo":312.0,"LegalState":"Clean","Balance":1234567}"#,
    );

    let status = read_status(&dir, None);

    assert_eq!(status.fuel_main_t, Some(42.815));
    assert_eq!(status.cargo_t, Some(312));
}

#[test]
fn fractional_cargo_is_rounded_to_a_ton() {
    let dir = dir_with(
        "fraction",
        r#"{"event":"Status","Flags":0,"Fuel":{"FuelMain":10.0},"Cargo":311.6}"#,
    );

    assert_eq!(read_status(&dir, None).cargo_t, Some(312));
}

#[test]
fn closed_game_says_nothing_instead_of_zero() {
    let dir = dir_with(
        "closed",
        r#"{ "timestamp":"2026-09-06T16:25:12Z", "event":"Status", "Flags":0 }"#,
    );

    let status = read_status(&dir, None);

    assert_eq!(status.fuel_main_t, None);
    assert_eq!(status.cargo_t, None);
}

#[test]
fn missing_or_half_written_file_is_silence() {
    assert_eq!(read_status(&temp_dir("absent"), None), Default::default());
    assert_eq!(
        read_status(
            &dir_with("half", r#"{"event":"Status","Fuel":{"FuelMa"#),
            None
        ),
        Default::default()
    );
}

#[test]
fn nothing_but_fuel_and_cargo_is_taken() {
    let dir = dir_with(
        "private",
        r#"{"event":"Status","Flags":0,"Fuel":{"FuelMain":8.0},"Cargo":0.0,
            "Balance":9876543210,"LegalState":"Wanted","Latitude":12.34,"Longitude":56.78,
            "Health":0.42,"Oxygen":0.99,"BodyName":"Ngalinn 2 a"}"#,
    );

    let status = read_status(&dir, None);

    assert_eq!(status.fuel_main_t, Some(8.0));
    assert_eq!(status.cargo_t, Some(0));
}

#[test]
fn fuel_is_kept_exact() {
    use agent_core::CommanderStateBuilder;

    let snapshot = |tons: f64| {
        let mut builder = CommanderStateBuilder::new();
        builder.set_fuel_tons(Some(tons));
        format!("{builder:?}")
    };

    assert_ne!(snapshot(42.8151), snapshot(42.7906));
    assert!(snapshot(42.8151).contains("42.8151"));
}

fn moment(text: &str) -> DateTime<Utc> {
    text.parse().expect("разобрать момент")
}

#[test]
fn status_from_a_dead_session_is_silence() {
    let dir = dir_with(
        "stale",
        r#"{"timestamp":"2026-09-08T12:00:00Z","event":"Status","Flags":16777240,
            "Fuel":{"FuelMain":42.815,"FuelReservoir":0.63},"Cargo":312.0}"#,
    );

    let stale = read_status(&dir, Some(moment("2026-09-09T20:00:00Z")));
    assert_eq!(stale, Default::default());

    let live = read_status(&dir, Some(moment("2026-09-08T11:59:00Z")));
    assert_eq!(live.fuel_main_t, Some(42.815));
    assert_eq!(live.cargo_t, Some(312));
}

#[test]
fn unknown_session_takes_the_file_as_it_is() {
    let dir = dir_with(
        "no-session",
        r#"{"timestamp":"2020-01-01T00:00:00Z","event":"Status","Flags":0,
            "Fuel":{"FuelMain":8.0},"Cargo":0.0}"#,
    );

    assert_eq!(read_status(&dir, None).fuel_main_t, Some(8.0));
}
