use serde::Deserialize;

pub use super::event_parts::{
    EngineeringModifier, FactionPresence, FactionRef, FuelCapacity, LandingPads, LoadoutModule,
    MissionAcceptedFields, MissionSnapshotEntry, ModuleEngineering, StateEntry, StationEconomy,
    TrendingState,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "event")]
pub enum JournalEvent {
    #[serde(rename_all = "PascalCase")]
    Commander {
        name: String,
    },

    #[serde(rename_all = "PascalCase")]
    Location {
        star_system: String,
        star_pos: [f64; 3],
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        docked: bool,
        #[serde(default)]
        station_name: Option<String>,
        #[serde(default)]
        system_address: Option<u64>,
        #[serde(default)]
        system_allegiance: Option<String>,
        #[serde(default)]
        population: Option<u64>,
        #[serde(default)]
        system_faction: Option<FactionRef>,
        #[serde(default)]
        factions: Vec<FactionPresence>,
        #[serde(default, rename = "MarketID")]
        market_id: Option<u64>,
        #[serde(default)]
        station_type: Option<String>,
        #[serde(default)]
        station_faction: Option<FactionRef>,
        #[serde(default)]
        station_allegiance: Option<String>,
        #[serde(default)]
        station_government: Option<String>,
        #[serde(default)]
        station_services: Vec<String>,
        #[serde(default)]
        station_economies: Vec<StationEconomy>,
        #[serde(default, rename = "DistFromStarLS")]
        dist_from_star_ls: Option<f64>,
    },

    #[serde(rename = "FSDJump", rename_all = "PascalCase")]
    FsdJump {
        star_system: String,
        star_pos: [f64; 3],
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        fuel_level: Option<f64>,
        #[serde(default)]
        system_address: Option<u64>,
        #[serde(default)]
        system_allegiance: Option<String>,
        #[serde(default)]
        population: Option<u64>,
        #[serde(default)]
        system_faction: Option<FactionRef>,
        #[serde(default)]
        factions: Vec<FactionPresence>,
    },

    #[serde(rename_all = "PascalCase")]
    SupercruiseEntry {
        #[serde(default)]
        star_system: Option<String>,
    },

    #[serde(rename_all = "PascalCase")]
    SupercruiseExit {
        #[serde(default)]
        star_system: Option<String>,
        #[serde(default)]
        body: Option<String>,
    },

    #[serde(rename_all = "PascalCase")]
    ApproachBody {
        body: String,
    },

    #[serde(rename_all = "PascalCase")]
    Touchdown {
        #[serde(default)]
        body: Option<String>,
    },

    #[serde(rename_all = "PascalCase")]
    LeaveBody {
        #[serde(default)]
        body: Option<String>,
    },

    #[serde(rename_all = "PascalCase")]
    Docked {
        station_name: String,
        #[serde(default)]
        star_system: Option<String>,
        #[serde(default)]
        system_address: Option<u64>,
        #[serde(default, rename = "MarketID")]
        market_id: Option<u64>,
        #[serde(default)]
        station_type: Option<String>,
        #[serde(default)]
        station_faction: Option<FactionRef>,
        #[serde(default)]
        station_allegiance: Option<String>,
        #[serde(default)]
        station_government: Option<String>,
        #[serde(default)]
        station_services: Vec<String>,
        #[serde(default)]
        station_economies: Vec<StationEconomy>,
        #[serde(default, rename = "DistFromStarLS")]
        dist_from_star_ls: Option<f64>,
        #[serde(default)]
        landing_pads: Option<LandingPads>,
        #[serde(default, rename = "ActiveFine")]
        active_fine: Option<bool>,
    },

    Undocked,

    MissionAccepted(MissionAcceptedFields),

    #[serde(rename_all = "PascalCase")]
    CargoDepot {
        #[serde(rename = "MissionID")]
        mission_id: u64,
        #[serde(default)]
        cargo_type: Option<String>,
        #[serde(default, rename = "CargoType_Localised")]
        cargo_type_localised: Option<String>,
        #[serde(default, rename = "StartMarketID")]
        start_market_id: Option<u64>,
        #[serde(default)]
        items_collected: Option<u32>,
        #[serde(default)]
        items_delivered: Option<u32>,
        #[serde(default)]
        total_items_to_deliver: Option<u32>,
    },

    Died {},

    #[serde(rename_all = "PascalCase")]
    MissionCompleted {
        #[serde(rename = "MissionID")]
        mission_id: u64,
    },

    #[serde(rename_all = "PascalCase")]
    MissionAbandoned {
        #[serde(rename = "MissionID")]
        mission_id: u64,
    },

    #[serde(rename_all = "PascalCase")]
    MissionFailed {
        #[serde(rename = "MissionID")]
        mission_id: u64,
    },

    #[serde(rename_all = "PascalCase")]
    MissionRedirected {
        #[serde(rename = "MissionID")]
        mission_id: u64,
        #[serde(default)]
        new_destination_system: Option<String>,
        #[serde(default)]
        new_destination_station: Option<String>,
    },

    Missions {
        #[serde(default, rename = "Active")]
        active: Vec<MissionSnapshotEntry>,
    },

    #[serde(rename_all = "PascalCase")]
    PayFines {
        #[serde(default)]
        faction: Option<String>,
        #[serde(default)]
        all_fines: bool,
    },

    #[serde(rename_all = "PascalCase")]
    Rank {
        empire: u8,
        federation: u8,
    },

    #[serde(rename_all = "PascalCase")]
    Progress {
        empire: f64,
        federation: f64,
    },

    #[serde(rename_all = "PascalCase")]
    Promotion {
        #[serde(default)]
        empire: Option<u8>,
        #[serde(default)]
        federation: Option<u8>,
    },

    #[serde(alias = "ClearSavedGame")]
    NewCommander {},

    #[serde(rename_all = "PascalCase")]
    Loadout {
        ship: String,
        max_jump_range: f64,
        #[serde(default)]
        unladen_mass: Option<f64>,
        #[serde(default)]
        cargo_capacity: Option<u32>,
        #[serde(default)]
        fuel_capacity: Option<FuelCapacity>,
        #[serde(default)]
        modules: Vec<LoadoutModule>,
    },

    #[serde(rename_all = "PascalCase")]
    Cargo {
        #[serde(default)]
        vessel: Option<String>,
        #[serde(default)]
        count: Option<u32>,
    },

    #[serde(rename_all = "PascalCase")]
    FuelScoop {
        #[serde(default)]
        total: Option<f64>,
    },

    Fileheader {
        #[serde(default)]
        gameversion: Option<String>,
        #[serde(default)]
        build: Option<String>,
        #[serde(default, rename = "Odyssey")]
        odyssey: Option<bool>,
    },

    #[serde(rename_all = "PascalCase")]
    LoadGame {
        #[serde(default)]
        game_mode: Option<String>,
        #[serde(default)]
        ship: Option<String>,
        #[serde(default)]
        gameversion: Option<String>,
        #[serde(default, rename = "Ship_Localised")]
        ship_localised: Option<String>,
    },

    #[serde(rename_all = "PascalCase", alias = "SwitchSuitLoadout")]
    SuitLoadout {
        suit_name: String,
        #[serde(default, rename = "SuitName_Localised")]
        suit_name_localised: Option<String>,
    },

    Shutdown {},

    #[serde(other)]
    Other,
}
