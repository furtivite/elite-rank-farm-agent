use std::collections::BTreeSet;
use std::path::PathBuf;

use agent_core::journal::{observations_from, parse_line, Observation, ParsedLine};
use serde_json::Value;

fn schema(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("contracts")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).expect("схема должна быть валидным JSON")
}

fn keys(value: &Value) -> BTreeSet<String> {
    value.as_object().expect("объект").keys().cloned().collect()
}

fn observations(line: &str) -> Vec<Observation> {
    let ParsedLine::Event(parsed) = parse_line(line) else {
        panic!("ожидалось событие: {line}");
    };
    observations_from(&parsed)
}

const FSD_JUMP: &str = r#"{"timestamp":"2026-09-02T20:00:00Z","event":"FSDJump","Taxi":false,
    "StarSystem":"BD+29 2405","SystemAddress":4481899107026,"StarPos":[-6.28125,60.8125,5.5],
    "SystemAllegiance":"Alliance","SystemEconomy":"$economy_Industrial;","Population":40631702,
    "Body":"BD+29 2405","BodyID":1,"BodyType":"Star","JumpDist":6.742,"FuelUsed":0.71,
    "SystemFaction":{"Name":"Natural BD+29 2405 Front","FactionState":"Expansion"},
    "Factions":[{"Name":"Natural BD+29 2405 Front","FactionState":"Expansion","Government":"Dictatorship",
      "Influence":0.053892,"Allegiance":"Independent","Happiness":"$Faction_HappinessBand2;",
      "MyReputation":0.5,"ActiveStates":[{"State":"Expansion"}],
      "PendingStates":[{"State":"Boom","Trend":0}],"RecoveringStates":[{"State":"War","Trend":0}]}]}"#;

const DOCKED: &str = r#"{"timestamp":"2026-09-02T20:05:00Z","event":"Docked","StationName":"Gamow Orbital",
    "StationType":"Coriolis","StarSystem":"Chaxiraxi","SystemAddress":16063849047465,"MarketID":3229583104,
    "StationFaction":{"Name":"Alliance Office of Statistics","FactionState":"Expansion"},
    "StationGovernment":"$government_Corporate;","StationGovernment_Localised":"Корпоративная",
    "StationAllegiance":"Alliance","StationServices":["dock","missions","refuel"],
    "StationEconomies":[{"Name":"$economy_HighTech;","Proportion":1.0}],
    "DistFromStarLS":259.8,"LandingPads":{"Small":4,"Medium":8,"Large":5}}"#;

fn assert_matches_contract(schema_name: &str, payload: &Value) {
    let schema = schema(schema_name);
    let allowed = keys(&schema["properties"]);
    let present = keys(payload);

    let extra: Vec<_> = present.difference(&allowed).cloned().collect();
    assert!(extra.is_empty(), "{schema_name}: поля вне схемы: {extra:?}");

    for required in schema["required"].as_array().expect("required") {
        let name = required.as_str().unwrap();
        assert!(
            present.contains(name),
            "{schema_name}: нет обязательного {name}",
        );
    }
}

#[test]
fn system_observation_matches_its_contract() {
    let built = observations(FSD_JUMP);
    assert_eq!(built.len(), 1);

    let payload = serde_json::to_value(&built[0]).unwrap();
    assert_eq!(payload["type"], "system");
    assert_matches_contract("system-observation.schema.json", &payload["system"]);
}

#[test]
fn station_observation_matches_its_contract() {
    let built = observations(DOCKED);
    assert_eq!(built.len(), 1);

    let payload = serde_json::to_value(&built[0]).unwrap();
    assert_eq!(payload["type"], "station");
    assert_matches_contract("station-observation.schema.json", &payload["station"]);
}

#[test]
fn personal_fields_never_leave_the_machine() {
    let payload = serde_json::to_string(&observations(FSD_JUMP)[0]).unwrap();

    assert!(!payload.contains("eputation"), "{payload}");
    assert!(!payload.contains("appiness"), "{payload}");
}

#[test]
fn faction_values_are_normalised_to_the_contract_enum() {
    let observed = observations(FSD_JUMP);
    let Observation::System { system } = &observed[0] else {
        panic!("ожидалось наблюдение системы");
    };

    assert_eq!(system.system_allegiance.as_deref(), Some("Alliance"));
    assert_eq!(system.factions.len(), 1);

    let faction = &system.factions[0];
    assert_eq!(faction.allegiance, "Independent");
    assert_eq!(faction.government, "Dictatorship");
    assert_eq!(faction.state, "Expansion");
    assert_eq!(faction.active_states, vec!["Expansion".to_owned()]);
    assert_eq!(faction.pending_states[0].state, "Boom");
    assert_eq!(faction.recovering_states[0].state, "War");
}

#[test]
fn unknown_allegiance_becomes_other() {
    let line = r#"{"timestamp":"2026-09-02T20:00:00Z","event":"FSDJump","StarSystem":"X",
        "SystemAddress":1,"StarPos":[0,0,0],"SystemAllegiance":"PilotsFederation",
        "Factions":[{"Name":"F","Allegiance":"PilotsFederation","Government":"None",
        "Influence":0.1,"FactionState":"None"}]}"#;

    let Observation::System { system } = &observations(line)[0] else {
        panic!("ожидалось наблюдение системы");
    };
    assert_eq!(system.system_allegiance.as_deref(), Some("Other"));
    assert_eq!(system.factions[0].allegiance, "Other");
}

#[test]
fn station_fields_come_from_the_docked_event() {
    let Observation::Station { station } = &observations(DOCKED)[0] else {
        panic!("ожидалось наблюдение станции");
    };

    assert_eq!(station.market_id, 3229583104);
    assert_eq!(station.station_type, "Coriolis");
    assert_eq!(
        station.controlling_faction.name,
        "Alliance Office of Statistics"
    );
    assert_eq!(station.station_allegiance.as_deref(), Some("Alliance"));
    assert!(station.services.iter().any(|service| service == "missions"));
    assert_eq!(station.landing_pads.as_ref().unwrap().large, 5);
    assert_eq!(station.dist_from_star_ls, Some(259.8));
}

#[test]
fn docked_location_gives_system_first_then_station() {
    let line = r#"{"timestamp":"2026-09-02T20:00:00Z","event":"Location","StarSystem":"Chaxiraxi",
        "SystemAddress":16063849047465,"StarPos":[1,2,3],"Docked":true,"StationName":"Gamow Orbital",
        "StationType":"Coriolis","MarketID":3229583104,
        "StationFaction":{"Name":"Alliance Office of Statistics"},"StationServices":["dock"],
        "Factions":[]}"#;

    let built = observations(line);
    assert_eq!(built.len(), 2);
    assert!(matches!(built[0], Observation::System { .. }));
    assert!(matches!(built[1], Observation::Station { .. }));
}

#[test]
fn undocked_location_gives_only_the_system() {
    let line = r#"{"timestamp":"2026-09-02T20:00:00Z","event":"Location","StarSystem":"Chaxiraxi",
        "SystemAddress":16063849047465,"StarPos":[1,2,3],"Docked":false,"Factions":[]}"#;

    let built = observations(line);
    assert_eq!(built.len(), 1);
    assert!(matches!(built[0], Observation::System { .. }));
}

#[test]
fn events_without_a_system_address_produce_nothing() {
    let line = r#"{"timestamp":"2026-09-02T20:00:00Z","event":"FSDJump","StarSystem":"X",
        "StarPos":[0,0,0]}"#;

    assert!(observations(line).is_empty());
}
