use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::contract::{
    ActiveMissionsPayload, MissionCargo, MissionEntry, AGENT_VERSION, MISSIONS_SCHEMA_VERSION,
};
use crate::journal::event::{JournalEvent, MissionAcceptedFields};
use crate::journal::parse::ParsedLine;

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveMission {
    pub mission_id: u64,
    pub name: String,
    pub localised_name: Option<String>,
    pub faction: Option<String>,
    pub destination_system: Option<String>,
    pub destination_station: Option<String>,
    pub origin: Option<(String, String)>,
    pub accepted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub cargo: Option<MissionCargoState>,
    pub details_known: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionCargoState {
    pub commodity: Option<String>,
    pub commodity_localised: Option<String>,
    pub total: u32,
    pub delivered: Option<u32>,
    pub collected: Option<u32>,
    pub source_market_id: Option<u64>,
    pub lost: u32,
}

impl MissionCargoState {
    pub fn in_hold(&self) -> u32 {
        let (Some(collected), Some(delivered)) = (self.collected, self.delivered) else {
            return 0;
        };

        collected
            .saturating_sub(delivered)
            .saturating_sub(self.lost)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MissionLog {
    missions: BTreeMap<u64, ActiveMission>,
    docked_at: Option<(String, String)>,
    observed_at: Option<DateTime<Utc>>,
}

impl MissionLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_line(&mut self, line: &ParsedLine) {
        if let ParsedLine::Event(parsed) = line {
            self.apply(parsed.timestamp, &parsed.event);
        }
    }

    fn apply(&mut self, timestamp: DateTime<Utc>, event: &JournalEvent) {
        match event {
            JournalEvent::Docked {
                station_name,
                star_system: Some(system),
                ..
            } => {
                self.docked_at = Some((system.clone(), station_name.clone()));
            }
            JournalEvent::Undocked => self.docked_at = None,

            JournalEvent::MissionAccepted(MissionAcceptedFields {
                mission_id,
                name,
                localised_name,
                faction,
                destination_system,
                destination_station,
                expiry,
                commodity,
                commodity_localised,
                count,
            }) => {
                let (system, station) = match (destination_system, destination_station) {
                    (Some(system), Some(station)) => (Some(system.clone()), Some(station.clone())),
                    _ => match &self.docked_at {
                        Some((system, station)) => (Some(system.clone()), Some(station.clone())),
                        None => (destination_system.clone(), destination_station.clone()),
                    },
                };

                self.missions.insert(
                    *mission_id,
                    ActiveMission {
                        mission_id: *mission_id,
                        name: name.clone(),
                        localised_name: localised_name.clone(),
                        faction: faction.clone(),
                        destination_system: system,
                        destination_station: station,
                        origin: self.docked_at.clone(),
                        accepted_at: timestamp,
                        expires_at: *expiry,
                        cargo: count.map(|total| MissionCargoState {
                            commodity: commodity.clone(),
                            commodity_localised: commodity_localised.clone(),
                            total,
                            delivered: None,
                            collected: None,
                            source_market_id: None,
                            lost: 0,
                        }),
                        details_known: true,
                    },
                );
                self.observed_at = Some(timestamp);
            }

            JournalEvent::CargoDepot {
                mission_id,
                cargo_type,
                cargo_type_localised,
                start_market_id,
                items_collected,
                items_delivered,
                total_items_to_deliver,
            } => {
                let Some(mission) = self.missions.get_mut(mission_id) else {
                    return;
                };

                let total =
                    total_items_to_deliver.or(mission.cargo.as_ref().map(|cargo| cargo.total));
                let Some(total) = total else {
                    return;
                };

                let previous = mission.cargo.take();
                mission.cargo = Some(MissionCargoState {
                    commodity: cargo_type
                        .clone()
                        .or_else(|| previous.as_ref().and_then(|cargo| cargo.commodity.clone())),
                    commodity_localised: cargo_type_localised.clone().or_else(|| {
                        previous
                            .as_ref()
                            .and_then(|cargo| cargo.commodity_localised.clone())
                    }),
                    total,
                    delivered: *items_delivered,
                    collected: *items_collected,
                    source_market_id: start_market_id
                        .or_else(|| previous.as_ref().and_then(|cargo| cargo.source_market_id)),
                    lost: previous.as_ref().map_or(0, |cargo| cargo.lost),
                });
                self.observed_at = Some(timestamp);
            }

            JournalEvent::Died {} => {
                let mut changed = false;
                for mission in self.missions.values_mut() {
                    let Some(cargo) = mission.cargo.as_mut() else {
                        continue;
                    };
                    let in_hold = cargo.in_hold();
                    if in_hold > 0 {
                        cargo.lost += in_hold;
                        changed = true;
                    }
                }
                if changed {
                    self.observed_at = Some(timestamp);
                }
            }

            JournalEvent::MissionCompleted { mission_id }
            | JournalEvent::MissionAbandoned { mission_id }
            | JournalEvent::MissionFailed { mission_id } => {
                self.missions.remove(mission_id);
                self.observed_at = Some(timestamp);
            }

            JournalEvent::MissionRedirected {
                mission_id,
                new_destination_system,
                new_destination_station,
            } => {
                if let Some(mission) = self.missions.get_mut(mission_id) {
                    mission.destination_system = new_destination_system.clone();
                    mission.destination_station = new_destination_station.clone();
                    self.observed_at = Some(timestamp);
                }
            }

            JournalEvent::Missions { active } => {
                let active_ids: Vec<u64> = active.iter().map(|entry| entry.mission_id).collect();

                self.missions.retain(|id, _| active_ids.contains(id));

                for entry in active {
                    self.missions
                        .entry(entry.mission_id)
                        .or_insert(ActiveMission {
                            mission_id: entry.mission_id,
                            name: entry.name.clone(),
                            localised_name: None,
                            faction: None,
                            origin: None,
                            destination_system: None,
                            destination_station: None,
                            accepted_at: timestamp,
                            expires_at: None,
                            cargo: None,
                            details_known: false,
                        });
                }
                self.observed_at = Some(timestamp);
            }

            _ => {}
        }
    }

    pub fn active(&self) -> Vec<ActiveMission> {
        self.missions.values().cloned().collect()
    }

    pub fn observed_at(&self) -> Option<DateTime<Utc>> {
        self.observed_at
    }
}

impl MissionLog {
    pub fn payload(&self) -> Option<ActiveMissionsPayload> {
        let observed_at = self.observed_at?;

        Some(ActiveMissionsPayload {
            schema_version: MISSIONS_SCHEMA_VERSION,
            agent_version: AGENT_VERSION.to_string(),
            observed_at,
            missions: self
                .missions
                .values()
                .map(|mission| MissionEntry {
                    mission_id: mission.mission_id,
                    name: mission.name.clone(),
                    localised_name: mission.localised_name.clone(),
                    faction: mission.faction.clone(),
                    destination_system: mission.destination_system.clone(),
                    destination_station: mission.destination_station.clone(),
                    origin_system: mission.origin.as_ref().map(|(system, _)| system.clone()),
                    origin_station: mission.origin.as_ref().map(|(_, station)| station.clone()),
                    accepted_at: mission.accepted_at,
                    expires_at: mission.expires_at,
                    cargo: mission.cargo.as_ref().map(|cargo| MissionCargo {
                        commodity: cargo.commodity.clone(),
                        commodity_localised: cargo.commodity_localised.clone(),
                        total: cargo.total,
                        delivered: cargo.delivered,
                        collected: cargo.collected,
                        source_market_id: cargo.source_market_id.map(|id| id.to_string()),
                        lost: (cargo.lost > 0).then_some(cargo.lost),
                    }),
                    details_known: mission.details_known,
                })
                .collect(),
        })
    }
}
