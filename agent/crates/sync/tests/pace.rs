use std::time::Duration;

use agent_core::{CommanderState, SystemPosition, AGENT_VERSION, SCHEMA_VERSION};
use agent_sync::{decide, SendDecision, SendPace};
use chrono::{TimeZone, Utc};

const MIN: Duration = Duration::from_secs(5);

fn state(system: &str) -> CommanderState {
    CommanderState {
        schema_version: SCHEMA_VERSION,
        agent_version: AGENT_VERSION.to_owned(),
        fined_factions: Vec::new(),
        commander_name: "Jameson".to_owned(),
        current_system: SystemPosition {
            name: system.to_owned(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        current_station: None,
        current_body: None,
        ship: "diamondbackxl".to_owned(),
        required_pad_size: None,
        jump_range_ly: 43.0,
        imperial_rank: 7,
        imperial_progress: 71.0,
        imperial_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        federal_rank: 2,
        federal_progress: 100.0,
        federal_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        observed_at: Utc.with_ymd_and_hms(2026, 9, 1, 22, 16, 9).unwrap(),
        presence: None,
        loadout: None,
        recent_systems: None,
        recent_supercruise: None,
    }
}

#[test]
fn первая_отправка_идёт_сразу() {
    let current = state("Ngalinn");
    assert_eq!(decide(Some(&current), None, None, MIN), SendDecision::Send);
}

#[test]
fn без_состояния_отправлять_нечего() {
    assert_eq!(decide(None, None, None, MIN), SendDecision::NotReady);
}

#[test]
fn неизменное_состояние_не_отправляется_повторно() {
    let current = state("Ngalinn");
    let sent = state("Ngalinn");
    assert_eq!(
        decide(
            Some(&current),
            Some(&sent),
            Some(Duration::from_secs(60)),
            MIN
        ),
        SendDecision::Unchanged
    );
}

#[test]
fn изменение_раньше_интервала_откладывается() {
    let current = state("Mainani");
    let sent = state("Ngalinn");
    assert_eq!(
        decide(
            Some(&current),
            Some(&sent),
            Some(Duration::from_secs(2)),
            MIN
        ),
        SendDecision::TooSoon(Duration::from_secs(3))
    );
}

#[test]
fn изменение_после_интервала_отправляется() {
    let current = state("Mainani");
    let sent = state("Ngalinn");
    assert_eq!(
        decide(
            Some(&current),
            Some(&sent),
            Some(Duration::from_secs(5)),
            MIN
        ),
        SendDecision::Send
    );
}

#[test]
fn изменение_только_станции_тоже_отправляется() {
    let mut current = state("Ngalinn");
    current.current_station = Some("Archimedes City".to_owned());
    let sent = state("Ngalinn");

    assert_eq!(
        decide(
            Some(&current),
            Some(&sent),
            Some(Duration::from_secs(10)),
            MIN
        ),
        SendDecision::Send
    );
}

#[test]
fn pace_starts_at_the_client_guess() {
    let pace = SendPace::default();

    assert_eq!(pace.interval(), Duration::from_secs(5));
}

#[test]
fn server_suggestion_becomes_the_new_interval() {
    let mut pace = SendPace::default();

    pace.learn(Some(Duration::from_secs(10)));

    assert_eq!(pace.interval(), Duration::from_secs(11));
}

#[test]
fn refusal_without_a_hint_doubles_the_interval() {
    let mut pace = SendPace::default();

    pace.learn(None);

    assert_eq!(pace.interval(), Duration::from_secs(10));
}

#[test]
fn learned_interval_never_drops_below_the_client_minimum() {
    let mut pace = SendPace::default();

    pace.learn(Some(Duration::from_secs(1)));

    assert_eq!(pace.interval(), Duration::from_secs(5));
}

#[test]
fn learned_interval_has_a_ceiling() {
    let mut pace = SendPace::default();

    pace.learn(Some(Duration::from_secs(3600)));

    assert_eq!(pace.interval(), Duration::from_secs(60));
}

#[test]
fn observations_keep_their_own_pace() {
    let mut pace = SendPace::new(Duration::from_secs(5));
    pace.learn(Some(Duration::from_secs(30)));

    assert_eq!(pace.interval(), Duration::from_secs(31));
    assert_eq!(SendPace::default().interval(), Duration::from_secs(5));
}

use agent_core::LoadoutFactsPayload;
use agent_sync::{decide_for_ship, ShipContext};

fn with_loadout(fuel: Option<f64>, cargo: Option<u32>) -> CommanderState {
    CommanderState {
        loadout: Some(LoadoutFactsPayload {
            unladen_mass_t: 1256.2,
            fuel_capacity_t: 64.0,
            cargo_capacity_t: Some(724),
            fuel_level_t: fuel,
            cargo_t: cargo,
            fsd_symbol: Some("int_hyperdrive_overcharge_size6_class5".to_owned()),
            fsd_optimal_mass_t: Some(3224.0),
            fsd_booster_symbol: None,
        }),
        ..state("Ngalinn")
    }
}

fn verdict(current: &CommanderState, last: &CommanderState, ship: ShipContext) -> SendDecision {
    decide_for_ship(
        Some(current),
        Some(last),
        Some(Duration::from_secs(60)),
        MIN,
        ship,
    )
}

#[test]
fn slow_fuel_burn_is_not_worth_a_request() {
    let before = with_loadout(Some(42.8), None);
    let after = with_loadout(Some(42.3), None);

    assert_eq!(
        verdict(&after, &before, ShipContext::default()),
        SendDecision::Unchanged
    );
}

#[test]
fn a_jump_worth_of_fuel_is() {
    let before = with_loadout(Some(42.8), None);
    let after = with_loadout(Some(35.2), None);

    assert_eq!(
        verdict(&after, &before, ShipContext::default()),
        SendDecision::Send
    );
}

#[test]
fn low_fuel_makes_the_step_finer() {
    let before = with_loadout(Some(12.0), None);
    let after = with_loadout(Some(10.6), None);

    assert_eq!(
        verdict(&after, &before, ShipContext::default()),
        SendDecision::Unchanged,
        "полтора процента бака при полном баке — шум"
    );
    assert_eq!(
        verdict(
            &after,
            &before,
            ShipContext {
                low_fuel: true,
                ..ShipContext::default()
            }
        ),
        SendDecision::Send,
        "при малом запасе тот же расход — уже новость"
    );
}

#[test]
fn charging_drive_sends_immediately() {
    let before = with_loadout(Some(42.8), None);
    let after = with_loadout(Some(42.7), None);

    assert_eq!(
        verdict(
            &after,
            &before,
            ShipContext {
                fsd_charging: true,
                ..ShipContext::default()
            }
        ),
        SendDecision::Send
    );
}

#[test]
fn cargo_is_ignored_on_the_pad_but_not_in_flight() {
    let before = with_loadout(Some(42.8), Some(0));
    let after = with_loadout(Some(42.8), Some(300));

    assert_eq!(
        verdict(
            &after,
            &before,
            ShipContext {
                parked: true,
                ..ShipContext::default()
            }
        ),
        SendDecision::Unchanged
    );
    assert_eq!(
        verdict(&after, &before, ShipContext::default()),
        SendDecision::Send
    );
}

#[test]
fn rank_goes_through_even_on_the_pad() {
    let before = with_loadout(Some(42.8), Some(0));
    let after = CommanderState {
        imperial_rank: 8,
        ..with_loadout(Some(42.8), Some(0))
    };

    assert_eq!(
        verdict(
            &after,
            &before,
            ShipContext {
                parked: true,
                ..ShipContext::default()
            }
        ),
        SendDecision::Send
    );
}

#[test]
fn a_new_system_always_goes_through() {
    let before = with_loadout(Some(42.8), None);
    let after = CommanderState {
        ..with_loadout(Some(42.8), None)
    };
    let moved = CommanderState {
        current_system: agent_core::SystemPosition {
            name: "Mainani".to_owned(),
            x: 10.0,
            y: 2.0,
            z: 3.0,
        },
        ..after
    };

    assert_eq!(
        verdict(
            &moved,
            &before,
            ShipContext {
                parked: true,
                ..ShipContext::default()
            }
        ),
        SendDecision::Send
    );
}
