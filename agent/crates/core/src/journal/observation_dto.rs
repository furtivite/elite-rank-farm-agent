use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservedFaction {
    pub name: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingStateDto {
    pub state: String,
    pub trend: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactionObservation {
    pub name: String,
    pub allegiance: String,
    pub government: String,
    pub influence: f64,
    pub state: String,
    pub active_states: Vec<String>,
    pub pending_states: Vec<TrendingStateDto>,
    pub recovering_states: Vec<TrendingStateDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemObservation {
    pub schema_version: u8,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_build: Option<String>,
    pub event: String,
    pub observed_at: DateTime<Utc>,
    pub system_address: u64,
    pub star_system: String,
    pub star_pos: Coordinates,
    pub system_allegiance: Option<String>,
    pub population: Option<u64>,
    pub controlling_faction: Option<ObservedFaction>,
    pub factions: Vec<FactionObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandingPadsDto {
    pub small: u32,
    pub medium: u32,
    pub large: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EconomyDto {
    pub name: String,
    pub proportion: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationObservation {
    pub schema_version: u8,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_build: Option<String>,
    pub event: String,
    pub observed_at: DateTime<Utc>,
    pub market_id: u64,
    pub station_name: String,
    pub station_type: String,
    pub system_address: u64,
    pub star_system: String,
    pub controlling_faction: ObservedFaction,
    pub station_allegiance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_government: Option<String>,
    pub services: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub economies: Option<Vec<EconomyDto>>,
    pub landing_pads: Option<LandingPadsDto>,
    pub dist_from_star_ls: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Observation {
    System { system: SystemObservation },
    Station { station: StationObservation },
}

impl Observation {
    pub fn stamp_game(&mut self, session: &super::session::GameSession) {
        let (version, build) = match self {
            Self::System { system } => (&mut system.game_version, &mut system.game_build),
            Self::Station { station } => (&mut station.game_version, &mut station.game_build),
        };
        *version = session.version().map(ToOwned::to_owned);
        *build = session.build().map(ToOwned::to_owned);
    }

    pub fn observed_at(&self) -> DateTime<Utc> {
        match self {
            Self::System { system } => system.observed_at,
            Self::Station { station } => station.observed_at,
        }
    }

    pub fn target(&self) -> ObservationTarget {
        match self {
            Self::System { system } => ObservationTarget::System(system.system_address),
            Self::Station { station } => ObservationTarget::Station(station.market_id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationTarget {
    System(u64),
    Station(u64),
}
