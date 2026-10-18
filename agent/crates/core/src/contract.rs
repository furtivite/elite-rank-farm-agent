use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use history::{
    RecentSystem, SupercruiseLeg, MAX_LEG_SECONDS, RECENT_SUPERCRUISE_LIMIT, RECENT_SYSTEMS_LIMIT,
};
pub use presence::{Presence, PresenceStatus};

mod history;
mod presence;

pub const SCHEMA_VERSION: u8 = 4;

pub const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const COORDINATE_LIMIT: f64 = 100_000.0;
const MAX_JUMP_RANGE_LY: f64 = 500.0;
const MAX_RANK_INDEX: u8 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadSize {
    S,
    M,
    L,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemPosition {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommanderState {
    pub schema_version: u8,
    pub agent_version: String,
    pub fined_factions: Vec<String>,
    pub commander_name: String,
    pub current_system: SystemPosition,
    pub current_station: Option<String>,
    pub current_body: Option<String>,
    pub ship: String,
    pub required_pad_size: Option<PadSize>,
    pub jump_range_ly: f64,
    pub imperial_rank: u8,
    pub imperial_progress: f64,
    pub imperial_progress_at: DateTime<Utc>,
    pub federal_rank: u8,
    pub federal_progress: f64,
    pub federal_progress_at: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loadout: Option<LoadoutFactsPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_systems: Option<Vec<RecentSystem>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_supercruise: Option<Vec<SupercruiseLeg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoadoutFactsPayload {
    pub unladen_mass_t: f64,
    pub fuel_capacity_t: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_capacity_t: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_level_t: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_t: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsd_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsd_optimal_mass_t: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsd_booster_symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractViolation {
    pub field: &'static str,
    pub reason: String,
}

impl std::fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

impl CommanderState {
    pub fn validate(&self) -> Vec<ContractViolation> {
        let mut violations = Vec::new();

        if self.schema_version != SCHEMA_VERSION {
            violations.push(ContractViolation {
                field: "schemaVersion",
                reason: expected(SCHEMA_VERSION, self.schema_version),
            });
        }
        check_string("commanderName", &self.commander_name, 64, &mut violations);
        check_string(
            "currentSystem.name",
            &self.current_system.name,
            128,
            &mut violations,
        );
        check_coordinate("currentSystem.x", self.current_system.x, &mut violations);
        check_coordinate("currentSystem.y", self.current_system.y, &mut violations);
        check_coordinate("currentSystem.z", self.current_system.z, &mut violations);

        if let Some(station) = &self.current_station {
            check_max_len("currentStation", station, 128, &mut violations);
        }
        if let Some(body) = &self.current_body {
            check_max_len("currentBody", body, 128, &mut violations);
        }
        check_string("ship", &self.ship, 64, &mut violations);

        if !(self.jump_range_ly > 0.0 && self.jump_range_ly <= MAX_JUMP_RANGE_LY) {
            violations.push(ContractViolation {
                field: "jumpRangeLy",
                reason: expected(format!("0 < x <= {MAX_JUMP_RANGE_LY}"), self.jump_range_ly),
            });
        }
        check_rank("imperialRank", self.imperial_rank, &mut violations);
        check_rank("federalRank", self.federal_rank, &mut violations);
        check_progress("imperialProgress", self.imperial_progress, &mut violations);
        check_progress("federalProgress", self.federal_progress, &mut violations);

        violations
    }
}

fn expected(range: impl std::fmt::Display, got: impl std::fmt::Display) -> String {
    crate::tr!(
        "expected {range}, got {got}",
        "ожидается {range}, получено {got}"
    )
}

fn check_string(
    field: &'static str,
    value: &str,
    max_len: usize,
    violations: &mut Vec<ContractViolation>,
) {
    if value.is_empty() {
        violations.push(ContractViolation {
            field,
            reason: crate::tr!(
                "minLength 1, got an empty string",
                "minLength 1, получена пустая строка"
            ),
        });
        return;
    }
    check_max_len(field, value, max_len, violations);
}

fn check_max_len(
    field: &'static str,
    value: &str,
    max_len: usize,
    violations: &mut Vec<ContractViolation>,
) {
    let len = value.chars().count();
    if len > max_len {
        violations.push(ContractViolation {
            field,
            reason: crate::tr!(
                "maxLength {max_len}, got {len}",
                "maxLength {max_len}, получено {len}"
            ),
        });
    }
}

fn check_coordinate(field: &'static str, value: f64, violations: &mut Vec<ContractViolation>) {
    if !value.is_finite() || value.abs() > COORDINATE_LIMIT {
        violations.push(ContractViolation {
            field,
            reason: expected(format!("+-{COORDINATE_LIMIT} ly"), value),
        });
    }
}

fn check_rank(field: &'static str, value: u8, violations: &mut Vec<ContractViolation>) {
    if value > MAX_RANK_INDEX {
        violations.push(ContractViolation {
            field,
            reason: expected(format!("0..={MAX_RANK_INDEX}"), value),
        });
    }
}

fn check_progress(field: &'static str, value: f64, violations: &mut Vec<ContractViolation>) {
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        violations.push(ContractViolation {
            field,
            reason: expected("0..=100", value),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMissionsPayload {
    pub schema_version: u8,
    pub agent_version: String,
    pub observed_at: DateTime<Utc>,
    pub missions: Vec<MissionEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionEntry {
    pub mission_id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localised_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_station: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_station: Option<String>,
    pub accepted_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo: Option<MissionCargo>,
    pub details_known: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionCargo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commodity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commodity_localised: Option<String>,
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collected: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_hold: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lost: Option<u32>,
}

pub const MISSIONS_SCHEMA_VERSION: u8 = 1;
