use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;

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
}

pub fn read_cargo_tons(
    journal_dir: &Path,
    session_started_at: Option<DateTime<Utc>>,
) -> Option<u32> {
    let raw = std::fs::read_to_string(journal_dir.join(CARGO_FILE)).ok()?;
    let parsed: CargoFile = serde_json::from_str(&raw).ok()?;

    if let (Some(started), Some(written)) = (session_started_at, parsed.timestamp) {
        if written < started {
            return None;
        }
    }

    match parsed.vessel.as_deref() {
        Some("Ship") | None => parsed.count,
        _ => None,
    }
}
