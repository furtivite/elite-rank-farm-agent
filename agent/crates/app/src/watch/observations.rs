use agent_core::journal::Observation;
use agent_sync::{ObservationQueue, SendPace, StateClient, MAX_BATCH};

use super::pace::learn_pace;
use crate::say;

pub(super) fn flush_observations(
    client: &StateClient,
    queue: &ObservationQueue,
    queued: &mut Vec<Observation>,
    pace: &mut SendPace,
) -> FlushOutcome {
    let batch_size = queued.len().min(MAX_BATCH);
    let batch: Vec<Observation> = queued[..batch_size].to_vec();

    match client.post_observations(&batch) {
        Ok(accepted) => {
            queued.drain(..batch_size);
            if let Err(error) = queue.replace(queued) {
                say!("очередь наблюдений не перезаписалась: {error}");
            }

            say!(
                "  наблюдения: отправлено {}, применено {}, без изменений {}{}",
                accepted.accepted,
                accepted.applied,
                accepted.unchanged,
                if accepted.skipped.is_empty() {
                    String::new()
                } else {
                    format!(
                        ", пропущено сервером {} ({})",
                        accepted.skipped.len(),
                        skip_reasons(&accepted.skipped),
                    )
                },
            );

            if !queued.is_empty() {
                say!("  осталось в очереди: {}", queued.len());
            }

            return if accepted.applied > 0 {
                FlushOutcome::Applied
            } else {
                FlushOutcome::Nothing
            };
        }
        Err(error) if error.is_retryable() => {
            say!("  наблюдения не отправлены: {error}");
            learn_pace(pace, &error, "наблюдений");
        }
        Err(error) if error.is_token_rejected() => {
            return FlushOutcome::TokenRejected(error.to_string());
        }
        Err(error) => {
            say!("  наблюдения отклонены и выброшены ({batch_size} шт.): {error}");
            queued.drain(..batch_size);
            if let Err(error) = queue.replace(queued) {
                say!("очередь наблюдений не перезаписалась: {error}");
            }
        }
    }

    FlushOutcome::Nothing
}

pub(super) enum FlushOutcome {
    Applied,
    Nothing,
    TokenRejected(String),
}

pub(super) fn skip_reasons(skipped: &[agent_sync::SkippedObservation]) -> String {
    let mut counts: Vec<(String, usize)> = Vec::new();

    for item in skipped {
        match counts.iter_mut().find(|(reason, _)| *reason == item.reason) {
            Some((_, count)) => *count += 1,
            None => counts.push((item.reason.clone(), 1)),
        }
    }

    counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    counts
        .iter()
        .map(|(reason, count)| format!("{reason}: {count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::skip_reasons;
    use agent_sync::SkippedObservation;

    fn skipped(reasons: &[&str]) -> Vec<SkippedObservation> {
        reasons
            .iter()
            .enumerate()
            .map(|(index, reason)| SkippedObservation {
                index: index as u32,
                reason: (*reason).to_owned(),
            })
            .collect()
    }

    #[test]
    fn reasons_are_counted_not_listed_one_by_one() {
        let line = skip_reasons(&skipped(&["outside-regions"; 200]));

        assert_eq!(line, "outside-regions: 200");
    }

    #[test]
    fn the_most_common_reason_goes_first() {
        let line = skip_reasons(&skipped(&[
            "unknown-system",
            "outside-regions",
            "outside-regions",
        ]));

        assert_eq!(line, "outside-regions: 2, unknown-system: 1");
    }

    #[test]
    fn nothing_skipped_gives_an_empty_line() {
        assert_eq!(skip_reasons(&[]), "");
    }
}
