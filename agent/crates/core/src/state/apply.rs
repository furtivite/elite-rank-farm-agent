use chrono::{DateTime, Utc};

use super::supercruise::{finish_leg, LegInProgress};
use super::{system_position, CommanderStateBuilder};
use crate::contract::{Presence, PresenceStatus, RECENT_SUPERCRUISE_LIMIT};
use crate::journal::{JournalEvent, JournalLine, ParsedLine};

impl CommanderStateBuilder {
    pub fn apply_line(&mut self, line: &ParsedLine) {
        if let ParsedLine::Event(event) = line {
            self.apply(event);
        }
    }

    fn start_session(&mut self, at: DateTime<Utc>) {
        self.presence = Some(Presence {
            status: PresenceStatus::Playing,
            at,
        });
        self.session_started_at = Some(at);
    }

    pub fn apply(&mut self, line: &JournalLine) {
        match &line.event {
            JournalEvent::Commander { name } => {
                self.commander_name = Some(name.clone());
            }
            JournalEvent::Location {
                star_system,
                star_pos,
                body,
                docked,
                station_name,
                ..
            } => {
                self.note_system(star_system, line.timestamp);
                self.current_system = Some(system_position(star_system, star_pos));
                self.current_body = body.clone();
                self.current_station = if *docked { station_name.clone() } else { None };
            }
            JournalEvent::ApproachBody { body } => {
                self.current_body = Some(body.clone());
            }
            JournalEvent::LeaveBody { .. } => {
                self.current_body = None;
            }
            JournalEvent::Touchdown { body } => {
                if let Some(body) = body {
                    self.current_body = Some(body.clone());
                }
            }
            JournalEvent::FsdJump {
                star_system,
                star_pos,
                body,
                fuel_level,
                ..
            } => {
                self.note_system(star_system, line.timestamp);
                self.leg_in_progress = Some(LegInProgress {
                    arrived_at: line.timestamp,
                    exited_at: None,
                });
                self.current_system = Some(system_position(star_system, star_pos));
                self.current_body = body.clone();
                self.current_station = None;
                if let Some(fuel_level) = fuel_level {
                    self.loadout.fuel_level_t = Some(*fuel_level);
                }
            }
            JournalEvent::SupercruiseEntry { .. } => {
                self.leg_in_progress = None;
            }
            JournalEvent::SupercruiseExit { .. } => {
                if let Some(leg) = self.leg_in_progress.as_mut() {
                    if leg.exited_at.is_none() {
                        leg.exited_at = Some(line.timestamp);
                    }
                }
            }
            JournalEvent::Docked {
                station_name,
                station_faction,
                active_fine,
                dist_from_star_ls,
                ..
            } => {
                self.current_station = Some(station_name.clone());
                if let Some(leg) = finish_leg(
                    self.leg_in_progress.take(),
                    *dist_from_star_ls,
                    line.timestamp,
                ) {
                    self.recent_supercruise.insert(0, leg);
                    self.recent_supercruise.truncate(RECENT_SUPERCRUISE_LIMIT);
                }

                if *active_fine == Some(true) {
                    if let Some(faction) = station_faction {
                        self.fined_factions.insert(faction.name.clone());
                    }
                }
            }
            JournalEvent::Undocked => {
                self.current_station = None;
            }
            JournalEvent::Rank { empire, federation } => {
                self.empire.observe_rank(*empire, line.timestamp);
                self.federal.observe_rank(*federation, line.timestamp);
            }
            JournalEvent::Promotion { empire, federation } => {
                if let Some(empire) = empire {
                    self.empire.observe_promotion(*empire, line.timestamp);
                }
                if let Some(federation) = federation {
                    self.federal.observe_promotion(*federation, line.timestamp);
                }
            }
            JournalEvent::NewCommander {} => {
                self.empire.restart(line.timestamp);
                self.federal.restart(line.timestamp);
                self.fined_factions.clear();
            }
            JournalEvent::Progress { empire, federation } => {
                self.empire.observe_progress(*empire, line.timestamp);
                self.federal.observe_progress(*federation, line.timestamp);
            }
            JournalEvent::Loadout {
                ship,
                max_jump_range,
                unladen_mass,
                cargo_capacity,
                fuel_capacity,
                modules,
            } => {
                self.ship = Some(ship.clone());
                self.jump_range_ly = Some(*max_jump_range);

                self.loadout.unladen_mass_t = *unladen_mass;
                self.loadout.cargo_capacity_t = *cargo_capacity;
                self.loadout.fuel_capacity_t = fuel_capacity.as_ref().and_then(|fuel| fuel.main);
                self.loadout.fsd_symbol = None;
                self.loadout.fsd_optimal_mass_t = None;
                self.loadout.fsd_booster_symbol = None;

                for module in modules {
                    if module.slot.eq_ignore_ascii_case("FrameShiftDrive") {
                        self.loadout.fsd_symbol = Some(module.item.clone());
                        self.loadout.fsd_optimal_mass_t = module
                            .engineering
                            .as_ref()
                            .and_then(|engineering| {
                                engineering
                                    .modifiers
                                    .iter()
                                    .find(|modifier| modifier.label == "FSDOptimalMass")
                            })
                            .and_then(|modifier| modifier.value);
                    } else if module
                        .item
                        .to_ascii_lowercase()
                        .contains("guardianfsdbooster")
                    {
                        self.loadout.fsd_booster_symbol = Some(module.item.clone());
                    }
                }
            }

            JournalEvent::Cargo { vessel, count } => {
                if vessel.as_deref().unwrap_or("Ship") == "Ship" {
                    self.loadout.cargo_t = *count;
                }
            }
            JournalEvent::FuelScoop { total } => {
                if let Some(total) = total {
                    self.loadout.fuel_level_t = Some(*total);
                }
            }
            JournalEvent::PayFines { faction, all_fines } => {
                if *all_fines || faction.is_none() {
                    self.fined_factions.clear();
                } else if let Some(name) = faction {
                    self.fined_factions.remove(name);
                }
            }

            JournalEvent::MissionAccepted(_)
            | JournalEvent::CargoDepot { .. }
            | JournalEvent::Died {}
            | JournalEvent::MissionCompleted { .. }
            | JournalEvent::MissionAbandoned { .. }
            | JournalEvent::MissionFailed { .. }
            | JournalEvent::MissionRedirected { .. }
            | JournalEvent::Missions { .. } => return,

            JournalEvent::LoadGame {
                ship: Some(ship),
                ship_localised: Some(label),
                ..
            } => {
                self.ship_label = Some((ship.to_lowercase(), label.clone()));
                self.start_session(line.timestamp);
            }
            JournalEvent::LoadGame { .. } => self.start_session(line.timestamp),
            JournalEvent::SuitLoadout {
                suit_name,
                suit_name_localised,
            } => {
                self.suit = Some((suit_name.clone(), suit_name_localised.clone()));
                return;
            }
            JournalEvent::Shutdown {} => {
                self.presence = Some(Presence {
                    status: PresenceStatus::GameExited,
                    at: line.timestamp,
                });
            }

            JournalEvent::Fileheader { .. } | JournalEvent::Other => return,
        }

        self.observed_at = Some(line.timestamp);
    }
}
