use chrono::{DateTime, Utc};

use std::collections::BTreeSet;

pub use card::CommanderCard;
use ranks::NavyRank;
use supercruise::LegInProgress;

mod apply;
mod card;
mod ranks;
mod supercruise;

use crate::contract::{
    CommanderState, LoadoutFactsPayload, Presence, PresenceStatus, RecentSystem, SupercruiseLeg,
    SystemPosition, AGENT_VERSION, RECENT_SYSTEMS_LIMIT, SCHEMA_VERSION,
};
use crate::ships::pad_size_for_ship;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingField {
    CommanderName,
    CurrentSystem,
    Ship,
    JumpRange,
    Ranks,
    Progress,
}

impl MissingField {
    pub fn filled_by(self) -> &'static str {
        match self {
            Self::CommanderName => "Commander",
            Self::CurrentSystem => "Location / FSDJump",
            Self::Ship | Self::JumpRange => "Loadout",
            Self::Ranks => "Rank",
            Self::Progress => "Progress",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommanderStateBuilder {
    commander_name: Option<String>,
    current_system: Option<SystemPosition>,
    recent_systems: Vec<RecentSystem>,
    recent_supercruise: Vec<SupercruiseLeg>,
    leg_in_progress: Option<LegInProgress>,
    current_station: Option<String>,
    current_body: Option<String>,
    ship: Option<String>,
    fined_factions: BTreeSet<String>,
    jump_range_ly: Option<f64>,
    loadout: LoadoutFacts,
    empire: NavyRank,
    federal: NavyRank,
    observed_at: Option<DateTime<Utc>>,
    presence: Option<Presence>,
    session_started_at: Option<DateTime<Utc>>,
    ship_label: Option<(String, String)>,
    suit: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoadoutFacts {
    pub unladen_mass_t: Option<f64>,
    pub cargo_capacity_t: Option<u32>,
    pub fuel_capacity_t: Option<f64>,
    pub fuel_level_t: Option<f64>,
    pub cargo_t: Option<u32>,
    pub fsd_symbol: Option<String>,
    pub fsd_optimal_mass_t: Option<f64>,
    pub fsd_booster_symbol: Option<String>,
}

impl CommanderStateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn session_started_at(&self) -> Option<DateTime<Utc>> {
        self.session_started_at
    }

    pub fn build_stopping(&self, at: DateTime<Utc>) -> Option<CommanderState> {
        let mut state = self.build()?;
        state.presence = Some(Presence {
            status: PresenceStatus::AgentStopped,
            at,
        });
        Some(state)
    }

    pub fn set_cargo_tons(&mut self, tons: Option<u32>) {
        if let Some(tons) = tons {
            self.loadout.cargo_t = Some(tons);
        }
    }

    pub fn set_fuel_tons(&mut self, tons: Option<f64>) {
        if let Some(tons) = tons {
            self.loadout.fuel_level_t = Some(tons);
        }
    }

    fn note_system(&mut self, star_system: &str, at: DateTime<Utc>) {
        if let Some(first) = self.recent_systems.first_mut() {
            if first.name == star_system {
                first.at = at;
                return;
            }
        }

        self.recent_systems.insert(
            0,
            RecentSystem {
                name: star_system.to_owned(),
                at,
            },
        );
        self.recent_systems.truncate(RECENT_SYSTEMS_LIMIT);
    }

    pub fn missing(&self) -> Vec<MissingField> {
        let mut missing = Vec::new();
        if self.commander_name.is_none() {
            missing.push(MissingField::CommanderName);
        }
        if self.current_system.is_none() {
            missing.push(MissingField::CurrentSystem);
        }
        if self.ship.is_none() {
            missing.push(MissingField::Ship);
        }
        if self.jump_range_ly.is_none() {
            missing.push(MissingField::JumpRange);
        }
        if self.empire.rank.is_none() || self.federal.rank.is_none() {
            missing.push(MissingField::Ranks);
        }
        if self.empire.progress.is_none() || self.federal.progress.is_none() {
            missing.push(MissingField::Progress);
        }
        missing
    }

    pub fn build(&self) -> Option<CommanderState> {
        let (imperial_rank, _) = self.empire.rank?;
        let (federal_rank, _) = self.federal.rank?;
        let (imperial_progress, imperial_progress_at) = self.empire.progress?;
        let (federal_progress, federal_progress_at) = self.federal.progress?;
        let ship = self.ship.clone()?;

        Some(CommanderState {
            schema_version: SCHEMA_VERSION,
            agent_version: AGENT_VERSION.to_string(),
            fined_factions: self.fined_factions.iter().cloned().collect(),
            commander_name: self.commander_name.clone()?,
            current_system: self.current_system.clone()?,
            current_station: self.current_station.clone(),
            current_body: self.current_body.clone(),
            ship: ship.clone(),
            required_pad_size: pad_size_for_ship(&ship),
            jump_range_ly: self.jump_range_ly?,
            imperial_rank,
            imperial_progress,
            imperial_progress_at,
            federal_rank,
            federal_progress,
            federal_progress_at,
            observed_at: self.observed_at?,
            presence: self.presence.clone(),
            recent_supercruise: if self.recent_supercruise.is_empty() {
                None
            } else {
                Some(self.recent_supercruise.clone())
            },
            recent_systems: if self.recent_systems.is_empty() {
                None
            } else {
                Some(self.recent_systems.clone())
            },
            loadout: match (self.loadout.unladen_mass_t, self.loadout.fuel_capacity_t) {
                (Some(unladen_mass_t), Some(fuel_capacity_t)) => Some(LoadoutFactsPayload {
                    unladen_mass_t,
                    fuel_capacity_t,
                    cargo_capacity_t: self.loadout.cargo_capacity_t,
                    fuel_level_t: self.loadout.fuel_level_t,
                    cargo_t: self.loadout.cargo_t,
                    fsd_symbol: self.loadout.fsd_symbol.clone(),
                    fsd_optimal_mass_t: self.loadout.fsd_optimal_mass_t,
                    fsd_booster_symbol: self.loadout.fsd_booster_symbol.clone(),
                }),
                _ => None,
            },
        })
    }
}

fn system_position(name: &str, star_pos: &[f64; 3]) -> SystemPosition {
    SystemPosition {
        name: name.to_owned(),
        x: star_pos[0],
        y: star_pos[1],
        z: star_pos[2],
    }
}
