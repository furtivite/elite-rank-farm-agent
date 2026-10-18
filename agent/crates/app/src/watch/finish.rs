use std::collections::BTreeSet;

use agent_core::journal::Observation;
use agent_core::CommanderStateBuilder;
use agent_sync::{ObservationQueue, PendingState, SendPace, StateClient};
use chrono::Utc;

use super::observations::{flush_observations, FlushOutcome};
use super::rebuild::report_rebuild;
use super::state::send;
use crate::say_tr;

pub(super) struct Leftovers<'a> {
    pub builder: &'a CommanderStateBuilder,
    pub queued: &'a mut Vec<Observation>,
    pub observations_pace: &'a mut SendPace,
    pub rebuild_pending: bool,
    pub said_warnings: &'a mut BTreeSet<String>,
}

pub(super) fn finish(
    client: &StateClient,
    pending: &PendingState,
    observations: &ObservationQueue,
    pace: &mut SendPace,
    leftovers: Leftovers<'_>,
) {
    let reached = match leftovers.builder.build_stopping(Utc::now()) {
        Some(state) => match send(client, pending, &state, pace) {
            Ok(accepted) => {
                say_tr!(
                    "The agent is shutting down, the server is notified (updatedAt = {})",
                    "Агент выключается, сервер уведомлён (updatedAt = {})",
                    accepted.updated_at,
                );
                true
            }
            Err(error) => {
                say_tr!(
                    "could not report the shutdown: {error}",
                    "не удалось сообщить о выключении: {error}"
                );
                false
            }
        },
        None => false,
    };

    let mut applied = false;
    if should_flush(reached, leftovers.queued.len()) {
        match flush_observations(
            client,
            observations,
            leftovers.queued,
            leftovers.observations_pace,
        ) {
            FlushOutcome::Applied => applied = true,
            FlushOutcome::Nothing => {}
            FlushOutcome::TokenRejected(message) => say_tr!(
                "observations not accepted: {message}",
                "наблюдения не приняты: {message}"
            ),
        }
    }

    if should_rebuild(reached, leftovers.rebuild_pending, applied) {
        report_rebuild(client, leftovers.said_warnings);
    }
}

fn should_flush(server_reached: bool, queued: usize) -> bool {
    server_reached && queued > 0
}

fn should_rebuild(server_reached: bool, rebuild_pending: bool, applied_now: bool) -> bool {
    server_reached && (rebuild_pending || applied_now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unreachable_server_gets_no_more_requests() {
        assert!(!should_flush(false, 120));
        assert!(!should_rebuild(false, true, false));
    }

    #[test]
    fn leftover_observations_go_out_before_exit() {
        assert!(should_flush(true, 3));
        assert!(!should_flush(true, 0));
    }

    #[test]
    fn rebuild_only_when_the_data_changed() {
        assert!(!should_rebuild(true, false, false));
        assert!(should_rebuild(true, true, false));
        assert!(should_rebuild(true, false, true));
    }
}
