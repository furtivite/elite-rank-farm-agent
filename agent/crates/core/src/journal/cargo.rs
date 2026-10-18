use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Deserialize;

pub const CARGO_FILE: &str = "Cargo.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CargoFile {
    #[serde(default, rename = "timestamp")]
    timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    vessel: Option<String>,
    #[serde(default)]
    count: Option<u32>,
    #[serde(default)]
    inventory: Vec<CargoItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CargoItem {
    name: String,
    #[serde(default)]
    count: u32,
    #[serde(default, rename = "MissionID")]
    mission_id: Option<u64>,
}

pub type FreeCargo = BTreeMap<String, u32>;

pub fn read_cargo_tons(
    journal_dir: &Path,
    session_started_at: Option<DateTime<Utc>>,
) -> Option<u32> {
    read_ship_cargo(journal_dir, session_started_at)?.count
}

pub fn read_free_cargo(
    journal_dir: &Path,
    session_started_at: Option<DateTime<Utc>>,
) -> Option<FreeCargo> {
    let mut free = FreeCargo::new();
    for item in read_ship_cargo(journal_dir, session_started_at)?.inventory {
        if item.mission_id.is_none() {
            *free.entry(commodity_key(&item.name)).or_default() += item.count;
        }
    }
    Some(free)
}

pub fn collected_by_hand(
    mission_name: &str,
    commodity: Option<&str>,
    total: u32,
    free: Option<&FreeCargo>,
) -> Option<u32> {
    let by_hand = ["Mission_Collect", "Mission_Mining"]
        .iter()
        .any(|prefix| mission_name.starts_with(prefix));
    if !by_hand {
        return None;
    }
    let tons = free?.get(&commodity_key(commodity?)).copied().unwrap_or(0);
    Some(tons.min(total))
}

pub fn commodity_key(symbol: &str) -> String {
    let bare = symbol.trim().trim_start_matches('$').trim_end_matches(';');
    let lower = bare.to_lowercase();
    lower.strip_suffix("_name").unwrap_or(&lower).to_owned()
}

fn read_ship_cargo(
    journal_dir: &Path,
    session_started_at: Option<DateTime<Utc>>,
) -> Option<CargoFile> {
    let raw = std::fs::read_to_string(journal_dir.join(CARGO_FILE)).ok()?;
    let parsed: CargoFile = serde_json::from_str(&raw).ok()?;

    if let (Some(started), Some(written)) = (session_started_at, parsed.timestamp) {
        if written < started {
            return None;
        }
    }

    match parsed.vessel.as_deref() {
        Some("Ship") | None => Some(parsed),
        _ => None,
    }
}
