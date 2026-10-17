use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FuelCapacity {
    #[serde(default)]
    pub main: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LoadoutModule {
    pub slot: String,
    pub item: String,
    #[serde(default)]
    pub engineering: Option<ModuleEngineering>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModuleEngineering {
    #[serde(default)]
    pub modifiers: Vec<EngineeringModifier>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EngineeringModifier {
    pub label: String,
    #[serde(default)]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FactionPresence {
    pub name: String,
    #[serde(default)]
    pub allegiance: Option<String>,
    #[serde(default)]
    pub government: Option<String>,
    #[serde(default)]
    pub influence: Option<f64>,
    #[serde(default)]
    pub faction_state: Option<String>,
    #[serde(default)]
    pub active_states: Vec<StateEntry>,
    #[serde(default)]
    pub pending_states: Vec<TrendingState>,
    #[serde(default)]
    pub recovering_states: Vec<TrendingState>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StateEntry {
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrendingState {
    pub state: String,
    #[serde(default)]
    pub trend: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FactionRef {
    pub name: String,
    #[serde(default)]
    pub faction_state: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LandingPads {
    pub small: u32,
    pub medium: u32,
    pub large: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StationEconomy {
    pub name: String,
    pub proportion: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MissionSnapshotEntry {
    #[serde(rename = "MissionID")]
    pub mission_id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MissionAcceptedFields {
    #[serde(rename = "MissionID")]
    pub mission_id: u64,
    pub name: String,
    #[serde(default)]
    pub localised_name: Option<String>,
    #[serde(default)]
    pub faction: Option<String>,
    #[serde(default)]
    pub destination_system: Option<String>,
    #[serde(default)]
    pub destination_station: Option<String>,
    #[serde(default)]
    pub expiry: Option<DateTime<Utc>>,
    #[serde(default)]
    pub commodity: Option<String>,
    #[serde(default, rename = "Commodity_Localised")]
    pub commodity_localised: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
}
