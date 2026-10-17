use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;

pub const STATUS_FILE: &str = "Status.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StatusFile {
    #[serde(default, rename = "timestamp")]
    timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    flags: Option<u32>,
    #[serde(default)]
    fuel: Option<StatusFuel>,
    #[serde(default)]
    cargo: Option<f64>,
}

mod flag {
    pub const DOCKED: u32 = 1 << 0;
    pub const LANDED: u32 = 1 << 1;
    pub const FSD_CHARGING: u32 = 1 << 17;
    pub const LOW_FUEL: u32 = 1 << 19;
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StatusFuel {
    #[serde(default)]
    fuel_main: Option<f64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StatusFacts {
    pub flags: Option<u32>,
    pub fuel_main_t: Option<f64>,
    pub cargo_t: Option<u32>,
}

pub fn read_status(journal_dir: &Path, session_started_at: Option<DateTime<Utc>>) -> StatusFacts {
    let Ok(raw) = std::fs::read_to_string(journal_dir.join(STATUS_FILE)) else {
        return StatusFacts::default();
    };
    let Ok(parsed) = serde_json::from_str::<StatusFile>(&raw) else {
        return StatusFacts::default();
    };
    if let (Some(started), Some(written)) = (session_started_at, parsed.timestamp) {
        if written < started {
            return StatusFacts::default();
        }
    }

    StatusFacts {
        flags: parsed.flags,
        fuel_main_t: parsed.fuel.and_then(|fuel| fuel.fuel_main),
        cargo_t: parsed
            .cargo
            .filter(|tons| tons.is_finite() && *tons >= 0.0)
            .map(|tons| tons.round() as u32),
    }
}

impl StatusFacts {
    fn has(&self, bit: u32) -> bool {
        self.flags.is_some_and(|flags| flags & bit != 0)
    }

    pub fn is_docked(&self) -> bool {
        self.has(flag::DOCKED)
    }

    pub fn is_landed(&self) -> bool {
        self.has(flag::LANDED)
    }

    pub fn is_fsd_charging(&self) -> bool {
        self.has(flag::FSD_CHARGING)
    }

    pub fn is_low_fuel(&self) -> bool {
        self.has(flag::LOW_FUEL)
    }

    pub fn is_parked(&self) -> bool {
        self.is_docked() || self.is_landed()
    }
}
