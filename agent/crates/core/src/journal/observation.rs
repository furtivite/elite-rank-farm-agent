use std::collections::hash_map::Entry;
use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::event::{FactionPresence, FactionRef, JournalEvent, LandingPads};
pub use super::observation_dto::{
    Coordinates, EconomyDto, FactionObservation, LandingPadsDto, Observation, ObservationTarget,
    ObservedFaction, StationObservation, SystemObservation, TrendingStateDto,
};
use super::parse::JournalLine;

pub const OBSERVATION_SCHEMA_VERSION: u8 = 1;

const SOURCE: &str = "journal";

fn allegiance(raw: Option<&str>) -> Option<String> {
    let value = raw?.trim();
    if value.is_empty() {
        return None;
    }

    Some(
        match value {
            "Empire" | "Federation" | "Alliance" | "Independent" => value,
            _ => "Other",
        }
        .to_owned(),
    )
}

pub fn newest_per_target(observations: Vec<Observation>) -> Vec<Observation> {
    let mut newest: HashMap<ObservationTarget, Observation> = HashMap::new();

    for observation in observations {
        match newest.entry(observation.target()) {
            Entry::Occupied(mut slot) => {
                if observation.observed_at() > slot.get().observed_at() {
                    slot.insert(observation);
                }
            }
            Entry::Vacant(slot) => {
                slot.insert(observation);
            }
        }
    }

    let mut result: Vec<Observation> = newest.into_values().collect();
    result.sort_by_key(|observation| observation.observed_at());
    result
}

fn faction_ref(raw: &FactionRef) -> ObservedFaction {
    ObservedFaction {
        name: raw.name.clone(),
        state: raw.faction_state.clone(),
    }
}

fn faction(raw: &FactionPresence) -> FactionObservation {
    FactionObservation {
        name: raw.name.clone(),
        allegiance: allegiance(raw.allegiance.as_deref()).unwrap_or_else(|| "Other".to_owned()),
        government: raw
            .government
            .clone()
            .unwrap_or_else(|| "Unknown".to_owned()),
        influence: raw.influence.unwrap_or(0.0).clamp(0.0, 1.0),
        state: raw
            .faction_state
            .clone()
            .unwrap_or_else(|| "None".to_owned()),
        active_states: raw
            .active_states
            .iter()
            .map(|entry| entry.state.clone())
            .collect(),
        pending_states: raw
            .pending_states
            .iter()
            .map(|entry| TrendingStateDto {
                state: entry.state.clone(),
                trend: entry.trend,
            })
            .collect(),
        recovering_states: raw
            .recovering_states
            .iter()
            .map(|entry| TrendingStateDto {
                state: entry.state.clone(),
                trend: entry.trend,
            })
            .collect(),
    }
}

fn pads(raw: Option<LandingPads>) -> Option<LandingPadsDto> {
    raw.map(|value| LandingPadsDto {
        small: value.small,
        medium: value.medium,
        large: value.large,
    })
}

pub fn observations_from(line: &JournalLine) -> Vec<Observation> {
    let at = line.timestamp;
    let mut result = Vec::new();

    match &line.event {
        JournalEvent::FsdJump {
            star_system,
            star_pos,
            system_address: Some(address),
            system_allegiance,
            population,
            system_faction,
            factions,
            ..
        } => {
            result.push(system_observation(
                "FSDJump",
                at,
                *address,
                star_system,
                star_pos,
                system_allegiance.as_deref(),
                *population,
                system_faction.as_ref(),
                factions,
            ));
        }
        JournalEvent::Location {
            star_system,
            star_pos,
            system_address,
            system_allegiance,
            population,
            system_faction,
            factions,
            docked,
            station_name,
            market_id,
            station_type,
            station_faction,
            station_allegiance,
            station_government,
            station_services,
            station_economies,
            dist_from_star_ls,
            ..
        } => {
            let Some(address) = system_address else {
                return result;
            };

            result.push(system_observation(
                "Location",
                at,
                *address,
                star_system,
                star_pos,
                system_allegiance.as_deref(),
                *population,
                system_faction.as_ref(),
                factions,
            ));

            if *docked {
                if let (Some(market), Some(name), Some(kind), Some(controller)) = (
                    market_id,
                    station_name.as_ref(),
                    station_type.as_ref(),
                    station_faction.as_ref(),
                ) {
                    result.push(Observation::Station {
                        station: StationObservation {
                            schema_version: OBSERVATION_SCHEMA_VERSION,
                            game_version: None,
                            game_build: None,
                            source: SOURCE.to_owned(),
                            event: "Location".to_owned(),
                            observed_at: at,
                            market_id: *market,
                            station_name: name.clone(),
                            station_type: kind.clone(),
                            system_address: *address,
                            star_system: star_system.clone(),
                            controlling_faction: faction_ref(controller),
                            station_allegiance: allegiance(station_allegiance.as_deref()),
                            station_government: station_government.clone(),
                            services: station_services.clone(),
                            economies: economies(station_economies),
                            landing_pads: None,
                            dist_from_star_ls: *dist_from_star_ls,
                        },
                    });
                }
            }
        }
        JournalEvent::Docked {
            station_name,
            star_system,
            system_address,
            market_id,
            station_type,
            station_faction,
            station_allegiance,
            station_government,
            station_services,
            station_economies,
            dist_from_star_ls,
            landing_pads,
            ..
        } => {
            if let (Some(address), Some(market), Some(system), Some(kind), Some(controller)) = (
                system_address,
                market_id,
                star_system.as_ref(),
                station_type.as_ref(),
                station_faction.as_ref(),
            ) {
                result.push(Observation::Station {
                    station: StationObservation {
                        schema_version: OBSERVATION_SCHEMA_VERSION,
                        game_version: None,
                        game_build: None,
                        source: SOURCE.to_owned(),
                        event: "Docked".to_owned(),
                        observed_at: at,
                        market_id: *market,
                        station_name: station_name.clone(),
                        station_type: kind.clone(),
                        system_address: *address,
                        star_system: system.clone(),
                        controlling_faction: faction_ref(controller),
                        station_allegiance: allegiance(station_allegiance.as_deref()),
                        station_government: station_government.clone(),
                        services: station_services.clone(),
                        economies: economies(station_economies),
                        landing_pads: pads(*landing_pads),
                        dist_from_star_ls: *dist_from_star_ls,
                    },
                });
            }
        }
        _ => {}
    }

    result
}

fn economies(raw: &[super::event::StationEconomy]) -> Option<Vec<EconomyDto>> {
    if raw.is_empty() {
        return None;
    }

    Some(
        raw.iter()
            .map(|entry| EconomyDto {
                name: entry.name.clone(),
                proportion: entry.proportion,
            })
            .collect(),
    )
}

#[allow(clippy::too_many_arguments)]
fn system_observation(
    event: &str,
    at: DateTime<Utc>,
    address: u64,
    star_system: &str,
    star_pos: &[f64; 3],
    system_allegiance: Option<&str>,
    population: Option<u64>,
    system_faction: Option<&FactionRef>,
    factions: &[FactionPresence],
) -> Observation {
    Observation::System {
        system: SystemObservation {
            schema_version: OBSERVATION_SCHEMA_VERSION,
            game_version: None,
            game_build: None,
            source: SOURCE.to_owned(),
            event: event.to_owned(),
            observed_at: at,
            system_address: address,
            star_system: star_system.to_owned(),
            star_pos: Coordinates {
                x: star_pos[0],
                y: star_pos[1],
                z: star_pos[2],
            },
            system_allegiance: allegiance(system_allegiance),
            population,
            controlling_faction: system_faction.map(faction_ref),
            factions: factions.iter().map(faction).collect(),
        },
    }
}
