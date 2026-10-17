use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const RECENT_SYSTEMS_LIMIT: usize = 10;

pub const RECENT_SUPERCRUISE_LIMIT: usize = 10;

pub const MAX_LEG_SECONDS: i64 = 3600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecentSystem {
    pub name: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SupercruiseLeg {
    pub at: DateTime<Utc>,
    pub distance_ls: f64,
    pub seconds: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docking_seconds: Option<u32>,
}
