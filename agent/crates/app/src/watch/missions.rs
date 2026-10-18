use std::time::{Duration, Instant};

use agent_core::contract::ActiveMissionsPayload;
use agent_core::journal::missions::MissionLog;
use agent_sync::{SendPace, StateClient};

use super::pace::learn_pace;
use super::report::token_rejected;
use crate::say_tr;

pub(super) fn missions_due(
    payload: &agent_core::contract::ActiveMissionsPayload,
    last_sent: Option<&agent_core::contract::ActiveMissionsPayload>,
    waited: Option<Duration>,
    interval: Duration,
) -> bool {
    last_sent != Some(payload) && due(waited, interval)
}

pub(super) fn due(since_last: Option<Duration>, interval: Duration) -> bool {
    since_last.is_none_or(|elapsed| elapsed >= interval)
}

pub(super) fn send_missions(
    client: &StateClient,
    missions: &MissionLog,
    last_sent: &mut Option<ActiveMissionsPayload>,
    last_at: &mut Option<Instant>,
    pace: &mut SendPace,
) -> Result<(), String> {
    if let Some(payload) = missions.payload() {
        let waited = last_at.map(|at| at.elapsed());
        if missions_due(&payload, last_sent.as_ref(), waited, pace.interval()) {
            *last_at = Some(Instant::now());
            match client.put_missions(&payload) {
                Ok(()) => {
                    say_tr!(
                        "Missions sent: {}",
                        "Задания отправлены: {}",
                        payload.missions.len()
                    );
                    *last_sent = Some(payload);
                }
                Err(error) if error.is_token_rejected() => {
                    return Err(token_rejected(&error.to_string()))
                }
                Err(error) => {
                    say_tr!(
                        "could not send missions: {error}",
                        "не удалось отправить задания: {error}"
                    );
                    learn_pace(pace, &error, &agent_core::tr!("missions", "заданий"));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{due, missions_due};
    use agent_core::contract::{ActiveMissionsPayload, MissionEntry};
    use std::time::Duration;

    fn payload(ids: &[u64]) -> ActiveMissionsPayload {
        ActiveMissionsPayload {
            schema_version: 1,
            agent_version: "0.0.0-test".to_owned(),
            observed_at: "2026-09-15T12:00:00Z".parse().expect("время разбирается"),
            missions: ids
                .iter()
                .map(|id| MissionEntry {
                    mission_id: *id,
                    name: "Mission_Delivery".to_owned(),
                    localised_name: None,
                    faction: None,
                    destination_system: None,
                    destination_station: None,
                    origin_system: None,
                    origin_station: None,
                    accepted_at: "2026-09-15T11:00:00Z".parse().expect("время разбирается"),
                    expires_at: None,
                    cargo: None,
                    details_known: true,
                })
                .collect(),
        }
    }

    const PACE: Duration = Duration::from_secs(5);

    #[test]
    fn unsent_list_goes_out() {
        assert!(missions_due(&payload(&[1]), None, None, PACE));
    }

    #[test]
    fn the_same_list_is_not_sent_twice() {
        let sent = payload(&[1]);
        assert!(!missions_due(&sent, Some(&sent), Some(PACE), PACE));
    }

    #[test]
    fn a_failed_send_is_retried_without_waiting_for_a_change() {
        let previously_sent = payload(&[1]);
        let now = payload(&[1, 2]);

        assert!(!missions_due(
            &now,
            Some(&previously_sent),
            Some(Duration::from_secs(2)),
            PACE
        ));
        assert!(missions_due(&now, Some(&previously_sent), Some(PACE), PACE));
    }

    #[test]
    fn nothing_ever_sent_retries_too() {
        assert!(missions_due(&payload(&[1]), None, Some(PACE), PACE));
    }

    #[test]
    fn first_attempt_needs_no_waiting() {
        assert!(due(None, Duration::from_secs(5)));
    }

    #[test]
    fn next_attempt_waits_out_the_server_interval() {
        assert!(!due(Some(Duration::from_secs(2)), Duration::from_secs(5)));
        assert!(!due(Some(Duration::from_secs(4)), Duration::from_secs(5)));
        assert!(due(Some(Duration::from_secs(5)), Duration::from_secs(5)));
        assert!(due(Some(Duration::from_secs(9)), Duration::from_secs(5)));
    }
}
