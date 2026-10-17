use agent_core::journal::missions::MissionLog;
use agent_core::journal::parse_line;

fn log(lines: &[&str]) -> MissionLog {
    let mut log = MissionLog::new();
    for line in lines {
        log.apply_line(&parse_line(line));
    }
    log
}

const ACCEPTED: &str = r#"{"timestamp":"2026-09-05T18:00:00Z","event":"MissionAccepted","Faction":"Society of Ngalinn","Name":"Mission_Courier","LocalisedName":"Курьерское задание","Expiry":"2026-09-06T18:00:00Z","Wing":false,"Influence":"++","Reputation":"++","MissionID":1041136691,"DestinationSystem":"HIP 8060","DestinationStation":"Archimedes City","Reward":50000,"TargetFaction":"Nationals of HIP 8060"}"#;

#[test]
fn accepted_mission_lands_in_the_list_with_its_target() {
    let missions = log(&[ACCEPTED]).active();

    assert_eq!(missions.len(), 1);
    assert_eq!(missions[0].mission_id, 1041136691);
    assert_eq!(missions[0].name, "Mission_Courier");
    assert_eq!(missions[0].destination_system.as_deref(), Some("HIP 8060"));
    assert_eq!(
        missions[0].destination_station.as_deref(),
        Some("Archimedes City")
    );
    assert!(missions[0].details_known);
}

#[test]
fn finished_missions_leave_the_list() {
    for closing in [
        r#"{"timestamp":"2026-09-05T19:00:00Z","event":"MissionCompleted","Faction":"Society of Ngalinn","Name":"Mission_Courier","MissionID":1041136691,"FactionEffects":[]}"#,
        r#"{"timestamp":"2026-09-05T19:00:00Z","event":"MissionAbandoned","Name":"Mission_Courier","MissionID":1041136691}"#,
        r#"{"timestamp":"2026-09-05T19:00:00Z","event":"MissionFailed","Name":"Mission_Courier","MissionID":1041136691}"#,
    ] {
        assert!(log(&[ACCEPTED, closing]).active().is_empty(), "{closing}");
    }
}

#[test]
fn donation_mission_gets_the_station_it_was_taken_at() {
    let missions = log(&[
        r#"{"timestamp":"2026-09-05T17:59:00Z","event":"Docked","StationName":"Hickam Survey","StarSystem":"Ngalinn"}"#,
        r#"{"timestamp":"2026-09-05T18:00:00Z","event":"MissionAccepted","Faction":"Society of Ngalinn","Name":"Mission_AltruismCredits","LocalisedName":"Пожертвование","Expiry":"2026-09-06T18:00:00Z","Wing":false,"Influence":"++","Reputation":"++","MissionID":42,"Donation":1000000}"#,
    ])
    .active();

    assert_eq!(missions[0].destination_system.as_deref(), Some("Ngalinn"));
    assert_eq!(
        missions[0].destination_station.as_deref(),
        Some("Hickam Survey"),
        "иначе треть корзины оказалась бы «без цели»"
    );
}

#[test]
fn redirect_moves_the_target() {
    let missions = log(&[
        ACCEPTED,
        r#"{"timestamp":"2026-09-05T18:30:00Z","event":"MissionRedirected","MissionID":1041136691,"Name":"Mission_Courier","NewDestinationStation":"Gloss Gateway","NewDestinationSystem":"Col 285 Sector LH-M c7-10","OldDestinationStation":"Archimedes City","OldDestinationSystem":"HIP 8060"}"#,
    ])
    .active();

    assert_eq!(
        missions[0].destination_station.as_deref(),
        Some("Gloss Gateway")
    );
    assert_eq!(
        missions[0].destination_system.as_deref(),
        Some("Col 285 Sector LH-M c7-10")
    );
}

#[test]
fn snapshot_closes_what_ended_while_the_agent_was_off() {
    let missions = log(&[
        ACCEPTED,
        r#"{"timestamp":"2026-09-06T10:00:00Z","event":"Missions","Active":[],"Failed":[],"Complete":[]}"#,
    ])
    .active();

    assert!(
        missions.is_empty(),
        "задание закончилось, пока агент был выключен"
    );
}

#[test]
fn snapshot_declares_unknown_missions_without_inventing_details() {
    let missions = log(&[
        r#"{"timestamp":"2026-09-06T10:00:00Z","event":"Missions","Active":[{"MissionID":777,"Name":"Mission_Courier_name","PassengerMission":false,"Expires":84333}],"Failed":[],"Complete":[]}"#,
    ])
    .active();

    assert_eq!(missions.len(), 1);
    assert_eq!(missions[0].mission_id, 777);
    assert!(
        !missions[0].details_known,
        "о взятии мы не слышали — подробностей нет и выдумывать их нельзя"
    );
    assert_eq!(missions[0].destination_system, None);
}

#[test]
fn snapshot_keeps_details_of_missions_we_saw_accepted() {
    let missions = log(&[
        ACCEPTED,
        r#"{"timestamp":"2026-09-06T10:00:00Z","event":"Missions","Active":[{"MissionID":1041136691,"Name":"Mission_Courier","PassengerMission":false,"Expires":1000}],"Failed":[],"Complete":[]}"#,
    ])
    .active();

    assert!(missions[0].details_known);
    assert_eq!(missions[0].destination_system.as_deref(), Some("HIP 8060"));
}

const WING_DELIVERY: &str = r#"{ "timestamp":"2026-09-08T11:12:55Z", "event":"MissionAccepted", "Faction":"Ngalinn Jet Natural Incorporated", "Name":"Mission_DeliveryWing_Boom", "LocalisedName":"Срочно доставьте 6004 шт. товара Снаряжение для выживания", "Commodity":"$SurvivalEquipment_Name;", "Commodity_Localised":"Снаряжение для выживания", "Count":6004, "TargetFaction":"Ngalinn Jet Natural Incorporated", "DestinationSystem":"Col 285 Sector SX-B b13-2", "DestinationStation":"Tull Reach", "Expiry":"2026-09-09T11:12:15Z", "Wing":true, "Influence":"+", "Reputation":"+", "Reward":10699766, "MissionID":1065480444 }"#;

const DEPOT_DELIVERED_3600: &str = r#"{ "timestamp":"2026-09-08T16:12:12Z", "event":"CargoDepot", "MissionID":1065480444, "UpdateType":"Deliver", "CargoType":"SurvivalEquipment", "CargoType_Localised":"Снаряжение для выживания", "Count":720, "StartMarketID":4224570115, "EndMarketID":4270463491, "ItemsCollected":3600, "ItemsDelivered":3600, "TotalItemsToDeliver":6004, "Progress":0.000000 }"#;

const DEPOT_COLLECTED_3600: &str = r#"{ "timestamp":"2026-09-08T16:04:21Z", "event":"CargoDepot", "MissionID":1065480444, "UpdateType":"Collect", "CargoType":"SurvivalEquipment", "CargoType_Localised":"Снаряжение для выживания", "Count":720, "StartMarketID":4224570115, "EndMarketID":4270463491, "ItemsCollected":3600, "ItemsDelivered":2880, "TotalItemsToDeliver":6004, "Progress":0.119920 }"#;

#[test]
fn cargo_mission_carries_its_commodity_and_count() {
    let missions = log(&[WING_DELIVERY]).active();
    let cargo = missions[0]
        .cargo
        .as_ref()
        .expect("у грузового задания есть груз");

    assert_eq!(cargo.total, 6004);
    assert_eq!(cargo.commodity.as_deref(), Some("$SurvivalEquipment_Name;"));
    assert_eq!(
        cargo.commodity_localised.as_deref(),
        Some("Снаряжение для выживания")
    );
    assert_eq!(cargo.delivered, None);
}

#[test]
fn courier_mission_has_no_cargo_at_all() {
    let missions = log(&[ACCEPTED]).active();

    assert_eq!(missions[0].cargo, None);
}

#[test]
fn depot_says_how_much_is_delivered_and_in_the_hold() {
    let missions = log(&[WING_DELIVERY, DEPOT_COLLECTED_3600]).active();
    let cargo = missions[0].cargo.as_ref().unwrap();

    assert_eq!(cargo.total, 6004);
    assert_eq!(cargo.delivered, Some(2880));
    assert_eq!(cargo.collected, Some(3600));
}

#[test]
fn depot_names_the_warehouse_by_market_id() {
    let missions = log(&[WING_DELIVERY, DEPOT_DELIVERED_3600]).active();

    assert_eq!(
        missions[0].cargo.as_ref().unwrap().source_market_id,
        Some(4224570115)
    );
}

#[test]
fn a_later_depot_line_wins() {
    let missions = log(&[WING_DELIVERY, DEPOT_COLLECTED_3600, DEPOT_DELIVERED_3600]).active();

    assert_eq!(missions[0].cargo.as_ref().unwrap().delivered, Some(3600));
}

#[test]
fn depot_without_its_mission_is_ignored() {
    let missions = log(&[DEPOT_DELIVERED_3600]).active();

    assert!(missions.is_empty());
}

#[test]
fn depot_restores_the_total_for_a_mission_from_the_snapshot() {
    let snapshot = r#"{"timestamp":"2026-09-08T16:00:00Z","event":"Missions","Active":[{"MissionID":1065480444,"Name":"Mission_DeliveryWing_Boom","PassengerMission":false,"Expires":86000}],"Failed":[],"Complete":[]}"#;
    let missions = log(&[snapshot, DEPOT_DELIVERED_3600]).active();
    let cargo = missions[0].cargo.as_ref().expect("склад принёс количество");

    assert_eq!(cargo.total, 6004);
    assert_eq!(cargo.delivered, Some(3600));
    assert!(!missions[0].details_known, "подробностей всё ещё нет");
}

#[test]
fn dying_with_cargo_marks_it_lost() {
    let died = r#"{ "timestamp":"2026-09-06T11:27:34Z", "event":"Died", "KillerName":"Ian \"Aussie\" Rose", "KillerShip":"ferdelance", "KillerRank":"Master" }"#;
    let missions = log(&[WING_DELIVERY, DEPOT_COLLECTED_3600, died]).active();
    let cargo = missions[0].cargo.as_ref().unwrap();

    assert_eq!(cargo.lost, 720);
    assert_eq!(cargo.in_hold(), 0);
    assert_eq!(cargo.delivered, Some(2880));
    assert_eq!(cargo.collected, Some(3600));
}

#[test]
fn dying_with_an_empty_hold_loses_nothing() {
    let died = r#"{"timestamp":"2026-09-08T16:30:00Z","event":"Died"}"#;
    let missions = log(&[WING_DELIVERY, DEPOT_DELIVERED_3600, died]).active();

    assert_eq!(missions[0].cargo.as_ref().unwrap().lost, 0);
}

#[test]
fn losses_add_up_across_deaths() {
    let died = r#"{"timestamp":"2026-09-08T16:30:00Z","event":"Died"}"#;
    let collect_again = r#"{ "timestamp":"2026-09-08T17:00:00Z", "event":"CargoDepot", "MissionID":1065480444, "UpdateType":"Collect", "CargoType":"SurvivalEquipment", "Count":720, "ItemsCollected":4320, "ItemsDelivered":2880, "TotalItemsToDeliver":6004, "Progress":0.1 }"#;
    let missions = log(&[
        WING_DELIVERY,
        DEPOT_COLLECTED_3600,
        died,
        collect_again,
        died,
    ])
    .active();
    let cargo = missions[0].cargo.as_ref().unwrap();

    assert_eq!(cargo.lost, 1440);
    assert_eq!(cargo.in_hold(), 0);
}

#[test]
fn mission_remembers_where_it_was_taken() {
    let missions = log(&[
        r#"{"timestamp":"2026-09-05T17:59:00Z","event":"Docked","StationName":"Hickam Survey","StarSystem":"Ngalinn"}"#,
        r#"{"timestamp":"2026-09-05T18:00:00Z","event":"MissionAccepted","Faction":"Society of Ngalinn","Name":"Mission_Delivery","MissionID":77,"DestinationSystem":"Mainani","DestinationStation":"Kirk Dock","Commodity":"$Gold_Name;","Count":60}"#,
    ])
    .active();

    assert_eq!(
        missions[0]
            .origin
            .as_ref()
            .map(|(system, station)| (system.as_str(), station.as_str())),
        Some(("Ngalinn", "Hickam Survey")),
    );
    assert_eq!(
        missions[0].destination_station.as_deref(),
        Some("Kirk Dock")
    );
}

#[test]
fn mission_taken_in_flight_has_no_origin() {
    let missions = log(&[
        r#"{"timestamp":"2026-09-05T17:59:00Z","event":"Docked","StationName":"Hickam Survey","StarSystem":"Ngalinn"}"#,
        r#"{"timestamp":"2026-09-05T17:59:30Z","event":"Undocked","StationName":"Hickam Survey"}"#,
        r#"{"timestamp":"2026-09-05T18:00:00Z","event":"MissionAccepted","Faction":"Society of Ngalinn","Name":"Mission_Delivery","MissionID":78,"DestinationSystem":"Mainani","DestinationStation":"Kirk Dock"}"#,
    ])
    .active();

    assert_eq!(missions[0].origin, None);
}

#[test]
fn mission_from_the_snapshot_has_no_origin() {
    let missions = log(&[
        r#"{"timestamp":"2026-09-05T17:59:00Z","event":"Docked","StationName":"Hickam Survey","StarSystem":"Ngalinn"}"#,
        r#"{"timestamp":"2026-09-05T18:00:00Z","event":"Missions","Active":[{"MissionID":91,"Name":"Mission_Delivery","PassengerMission":false,"Expires":86400}],"Failed":[],"Complete":[]}"#,
    ])
    .active();

    assert_eq!(missions[0].origin, None);
    assert!(!missions[0].details_known);
}
