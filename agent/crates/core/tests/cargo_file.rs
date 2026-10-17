use agent_core::journal::read_cargo_tons;
use std::fs;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-core-cargo-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("создать каталог");
    dir
}

fn dir_with(name: &str, contents: &str) -> PathBuf {
    let dir = temp_dir(name);
    fs::write(dir.join("Cargo.json"), contents).expect("запись Cargo.json");
    dir
}

#[test]
fn reads_tons_from_ship_cargo() {
    let dir = dir_with(
        "ship",
        r#"{"timestamp":"2026-09-08T10:00:00Z","event":"Cargo","Vessel":"Ship","Count":312,"Inventory":[{"Name":"gold","Count":312,"Stolen":0}]}"#,
    );

    assert_eq!(read_cargo_tons(&dir, None), Some(312));
}

#[test]
fn empty_hold_is_zero_not_unknown() {
    let dir = dir_with(
        "empty",
        r#"{"event":"Cargo","Vessel":"Ship","Count":0,"Inventory":[]}"#,
    );

    assert_eq!(read_cargo_tons(&dir, None), Some(0));
}

#[test]
fn srv_cargo_is_not_ship_cargo() {
    let dir = dir_with(
        "srv",
        r#"{"event":"Cargo","Vessel":"SRV","Count":4,"Inventory":[]}"#,
    );

    assert_eq!(read_cargo_tons(&dir, None), None);
}

#[test]
fn missing_or_broken_file_is_silence_not_zero() {
    let absent = temp_dir("absent");
    assert_eq!(read_cargo_tons(&absent, None), None);

    let broken = dir_with("broken", "{ это не json");
    assert_eq!(read_cargo_tons(&broken, None), None);

    let half_written = dir_with("half", r#"{"event":"Cargo","Vessel":"Ship""#);
    assert_eq!(read_cargo_tons(&half_written, None), None);
}

#[test]
fn cargo_from_a_dead_session_is_not_taken() {
    let dir = dir_with(
        "stale",
        r#"{"timestamp":"2026-09-08T12:00:00Z","event":"Cargo","Vessel":"Ship","Count":312}"#,
    );
    let started: chrono::DateTime<chrono::Utc> =
        "2026-09-09T20:00:00Z".parse().expect("разобрать момент");

    assert_eq!(read_cargo_tons(&dir, Some(started)), None);
}
