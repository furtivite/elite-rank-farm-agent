use std::fs;
use std::path::PathBuf;

use agent_core::journal::missions::MissionLog;
use agent_core::journal::{collected_by_hand, commodity_key, parse_line, read_free_cargo};

const COLLECT: &str = r#"{ "timestamp":"2026-09-12T20:16:10Z", "event":"MissionAccepted", "Faction":"Ngalinn Jet Natural Incorporated", "Name":"Mission_Collect_Industrial", "LocalisedName":"Для производства требуется 490 шт. товара Тритий", "Commodity":"$Tritium_Name;", "Commodity_Localised":"Тритий", "Count":490, "DestinationSystem":"Col 285 Sector VD-A b14-1", "DestinationStation":"Bede Sanctuary", "Expiry":"2026-09-13T20:02:17Z", "Wing":true, "Influence":"++", "Reputation":"++", "Reward":50000000, "MissionID":1065842158 }"#;

fn dir_with(name: &str, cargo: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-core-collect-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("создать каталог");
    fs::write(dir.join("Cargo.json"), cargo).expect("запись Cargo.json");
    dir
}

#[test]
fn commodity_keys_match_between_mission_and_hold() {
    assert_eq!(commodity_key("$Tritium_Name;"), "tritium");
    assert_eq!(commodity_key("$BasicMedicines_Name;"), "basicmedicines");
    assert_eq!(commodity_key("foodcartridges"), "foodcartridges");
}

#[test]
fn mission_cargo_is_not_free_cargo() {
    let dir = dir_with(
        "mixed",
        r#"{"timestamp":"2026-09-12T20:20:00Z","event":"Cargo","Vessel":"Ship","Count":314,"Inventory":[{"Name":"tritium","Name_Localised":"Тритий","Count":300,"Stolen":0},{"Name":"tritium","Count":10,"Stolen":0},{"Name":"foodcartridges","MissionID":1041227249,"Count":4,"Stolen":0}]}"#,
    );

    let free = read_free_cargo(&dir, None).expect("трюм прочитан");
    assert_eq!(free.get("tritium"), Some(&310));
    assert_eq!(free.get("foodcartridges"), None);
}

#[test]
fn only_collect_and_mining_get_a_count() {
    let free = [("tritium".to_owned(), 600)].into_iter().collect();

    assert_eq!(
        collected_by_hand(
            "Mission_Collect_Industrial",
            Some("$Tritium_Name;"),
            490,
            Some(&free)
        ),
        Some(490),
        "больше нужного не показываем"
    );
    assert_eq!(
        collected_by_hand(
            "Mission_Mining_Expansion",
            Some("$Gold_Name;"),
            20,
            Some(&free)
        ),
        Some(0),
        "товара нет — ноль, а не «не знаем»"
    );
    assert_eq!(
        collected_by_hand("Mission_Delivery", Some("$Tritium_Name;"), 10, Some(&free)),
        None
    );
    assert_eq!(
        collected_by_hand(
            "Mission_Collect_Industrial",
            Some("$Tritium_Name;"),
            490,
            None
        ),
        None,
        "трюм не прочитан — не знаем"
    );
}

#[test]
fn the_payload_carries_the_count_and_nothing_else_from_the_hold() {
    let mut log = MissionLog::new();
    log.apply_line(&parse_line(COLLECT));
    log.set_free_cargo(Some(
        [("tritium".to_owned(), 120), ("gold".to_owned(), 8)]
            .into_iter()
            .collect(),
    ));

    let payload = log.payload().expect("есть задание");
    let cargo = payload.missions[0].cargo.as_ref().expect("груз задания");
    assert_eq!(cargo.in_hold, Some(120));

    let json = serde_json::to_string(&payload).expect("сериализуется");
    assert!(json.contains("\"inHold\":120"), "{json}");
    assert!(!json.contains("gold"), "чужой товар не уходит: {json}");
}
