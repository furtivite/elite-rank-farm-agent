use std::collections::BTreeSet;
use std::path::PathBuf;

use agent_core::contract::{
    CommanderState, SystemPosition, AGENT_VERSION, MAX_LEG_SECONDS, RECENT_SUPERCRUISE_LIMIT,
    RECENT_SYSTEMS_LIMIT, SCHEMA_VERSION,
};
use chrono::{TimeZone, Utc};
use serde_json::Value;

fn schema() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("contracts/commander-state.schema.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).expect("схема должна быть валидным JSON")
}

fn sample() -> CommanderState {
    CommanderState {
        schema_version: SCHEMA_VERSION,
        agent_version: AGENT_VERSION.to_owned(),
        fined_factions: Vec::new(),
        commander_name: "Jameson".to_owned(),
        current_system: SystemPosition {
            name: "Mainani".to_owned(),
            x: 10.40625,
            y: -219.8125,
            z: -64.21875,
        },
        recent_systems: Some(vec![agent_core::RecentSystem {
            name: "Mainani".to_owned(),
            at: "2026-09-01T20:00:00Z"
                .parse()
                .expect("время должно разбираться"),
        }]),
        recent_supercruise: Some(vec![agent_core::SupercruiseLeg {
            at: "2026-09-01T20:05:00Z"
                .parse()
                .expect("время должно разбираться"),
            distance_ls: 1234.5,
            seconds: 180,
            docking_seconds: Some(45),
        }]),
        current_station: Some("Mies van der Rohe's Claim".to_owned()),
        current_body: Some("Mainani".to_owned()),
        ship: "diamondbackxl".to_owned(),
        required_pad_size: None,
        jump_range_ly: 43.487682,
        imperial_rank: 7,
        imperial_progress: 71.0,
        imperial_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        federal_rank: 2,
        federal_progress: 100.0,
        federal_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        observed_at: Utc.with_ymd_and_hms(2026, 9, 1, 22, 16, 9).unwrap(),
        presence: None,
        loadout: None,
    }
}

fn keys(value: &Value) -> BTreeSet<String> {
    value.as_object().expect("объект").keys().cloned().collect()
}

#[test]
fn serialized_payload_uses_only_schema_properties() {
    let schema = schema();
    let allowed = keys(&schema["properties"]);
    let payload = serde_json::to_value(sample()).unwrap();

    let extra: Vec<_> = keys(&payload).difference(&allowed).cloned().collect();
    assert!(extra.is_empty(), "поля вне схемы: {extra:?}");
}

#[test]
fn serialized_payload_contains_every_required_property() {
    let schema = schema();
    let payload = serde_json::to_value(sample()).unwrap();
    let present = keys(&payload);

    for required in schema["required"].as_array().expect("required") {
        let name = required.as_str().unwrap();
        assert!(present.contains(name), "в payload нет обязательного {name}");
    }
}

#[test]
fn schema_version_matches_the_constant_in_schema() {
    assert_eq!(
        schema()["properties"]["schemaVersion"]["const"],
        SCHEMA_VERSION
    );
}

#[test]
fn optional_fields_are_serialized_as_null_not_omitted() {
    let mut state = sample();
    state.current_station = None;
    state.current_body = None;

    let payload = serde_json::to_value(&state).unwrap();
    assert_eq!(payload["currentStation"], Value::Null);
    assert_eq!(payload["currentBody"], Value::Null);
    assert_eq!(payload["requiredPadSize"], Value::Null);
}

fn bound(field: &str, key: &str) -> f64 {
    schema()["properties"][field][key]
        .as_f64()
        .unwrap_or_else(|| panic!("в схеме нет properties/{field}/{key}"))
}

#[test]
fn validate_rejects_values_outside_schema_bounds() {
    let mut state = sample();
    state.imperial_rank = bound("imperialRank", "maximum") as u8 + 1;
    state.jump_range_ly = 0.0;
    state.federal_progress = bound("federalProgress", "maximum") + 1.0;
    state.commander_name = String::new();

    let fields: BTreeSet<_> = state
        .validate()
        .into_iter()
        .map(|violation| violation.field)
        .collect();
    assert!(fields.contains("imperialRank"));
    assert!(fields.contains("jumpRangeLy"));
    assert!(fields.contains("federalProgress"));
    assert!(fields.contains("commanderName"));
}

#[test]
fn validate_accepts_values_exactly_on_schema_bounds() {
    let mut state = sample();
    state.imperial_rank = bound("imperialRank", "maximum") as u8;
    state.federal_progress = bound("federalProgress", "maximum");
    state.jump_range_ly = bound("jumpRangeLy", "maximum");
    state.commander_name = "a".repeat(bound("commanderName", "maxLength") as usize);

    assert_eq!(state.validate(), Vec::new(), "{state:?}");
}

#[test]
fn string_limits_match_the_schema() {
    for field in ["commanderName", "ship", "currentStation", "currentBody"] {
        let limit = bound(field, "maxLength") as usize;
        let mut state = sample();
        let over = "a".repeat(limit + 1);
        match field {
            "commanderName" => state.commander_name = over,
            "ship" => state.ship = over,
            "currentStation" => state.current_station = Some(over),
            _ => state.current_body = Some(over),
        }

        let fields: BTreeSet<_> = state
            .validate()
            .into_iter()
            .map(|violation| violation.field)
            .collect();
        assert!(fields.contains(field), "{field}: длиннее maxLength прошло");
    }
}

#[test]
fn history_limits_match_the_schema() {
    assert_eq!(
        RECENT_SYSTEMS_LIMIT,
        bound("recentSystems", "maxItems") as usize,
    );
    assert_eq!(
        RECENT_SUPERCRUISE_LIMIT,
        bound("recentSupercruise", "maxItems") as usize,
    );

    let leg = &schema()["properties"]["recentSupercruise"]["items"]["properties"];
    assert_eq!(
        MAX_LEG_SECONDS,
        leg["seconds"]["maximum"].as_i64().expect("maximum"),
    );
    assert_eq!(
        MAX_LEG_SECONDS,
        leg["dockingSeconds"]["maximum"].as_i64().expect("maximum"),
    );
}

fn looks_like_semver(version: &str) -> bool {
    let core = version.split('-').next().unwrap_or("");
    if version.starts_with('-')
        || (version.contains('-')
            && version
                .split_once('-')
                .is_none_or(|(_, rest)| rest.is_empty()))
    {
        return false;
    }

    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| {
        !part.is_empty()
            && part.chars().all(|c| c.is_ascii_digit())
            && (part.len() == 1 || !part.starts_with('0'))
    })
}

#[test]
fn agent_version_is_semver() {
    assert!(
        looks_like_semver(AGENT_VERSION),
        "версия сборки {AGENT_VERSION} не по SemVer — сервер её отвергнет"
    );
}

#[test]
fn semver_check_rejects_what_it_should() {
    assert!(looks_like_semver("0.2.0"));
    assert!(looks_like_semver("0.2.0-beta.1"));
    assert!(looks_like_semver("10.0.1-rc.2"));
    assert!(!looks_like_semver("v0.2.0"));
    assert!(!looks_like_semver("0.2"));
    assert!(!looks_like_semver("0.01.0"));
    assert!(!looks_like_semver("0.2.0-"));
    assert!(!looks_like_semver(""));
}

#[test]
fn version_carries_no_prerelease_before_1_0() {
    let major = AGENT_VERSION.split('.').next().unwrap_or("");

    assert_eq!(
        major, "0",
        "правило про суффиксы держится до 1.0; сейчас {AGENT_VERSION}"
    );
    assert!(
        !AGENT_VERSION.contains('-'),
        "до 1.0 номер идёт без предрелизного суффикса, а тут {AGENT_VERSION}"
    );
}
