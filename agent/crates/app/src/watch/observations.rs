use agent_core::journal::{newest_per_target, Observation};
use agent_sync::{ObservationQueue, SendPace, StateClient, MAX_BATCH};

use super::pace::learn_pace;
use crate::say_tr;

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
                say_tr!(
                    "the observation queue was not rewritten: {error}",
                    "очередь наблюдений не перезаписалась: {error}"
                );
            }

            say_tr!(
                "  observations: sent {}, applied {}, unchanged {}{}",
                "  наблюдения: отправлено {}, применено {}, без изменений {}{}",
                accepted.accepted,
                accepted.applied,
                accepted.unchanged,
                if accepted.skipped.is_empty() {
                    String::new()
                } else {
                    agent_core::tr!(
                        ", skipped by the server {} ({})",
                        ", пропущено сервером {} ({})",
                        accepted.skipped.len(),
                        skip_reasons(&accepted.skipped),
                    )
                },
            );

            if !queued.is_empty() {
                say_tr!(
                    "  left in the queue: {}",
                    "  осталось в очереди: {}",
                    queued.len()
                );
            }

            return if accepted.applied > 0 {
                FlushOutcome::Applied
            } else {
                FlushOutcome::Nothing
            };
        }
        Err(error) if error.is_retryable() => {
            say_tr!(
                "  observations not sent: {error}",
                "  наблюдения не отправлены: {error}"
            );
            learn_pace(pace, &error, &agent_core::tr!("observations", "наблюдений"));
        }
        Err(error) if error.is_token_rejected() => {
            return FlushOutcome::TokenRejected(error.to_string());
        }
        Err(error) => {
            say_tr!(
                "  observations rejected and dropped ({batch_size}): {error}",
                "  наблюдения отклонены и выброшены ({batch_size} шт.): {error}"
            );
            queued.drain(..batch_size);
            if let Err(error) = queue.replace(queued) {
                say_tr!(
                    "the observation queue was not rewritten: {error}",
                    "очередь наблюдений не перезаписалась: {error}"
                );
            }
        }
    }

    FlushOutcome::Nothing
}

pub(super) fn load_queue(
    observations: &ObservationQueue,
    history_observations: Vec<Observation>,
    first_run: bool,
) -> Vec<Observation> {
    let mut queued: Vec<Observation> = match observations.load() {
        Ok(loaded) => loaded,
        Err(error) => {
            say_tr!(
                "the observation queue could not be read: {error}",
                "очередь наблюдений не прочиталась: {error}"
            );
            Vec::new()
        }
    };
    if !queued.is_empty() {
        say_tr!(
            "Observations queued since the last run: {}",
            "В очереди наблюдений с прошлого запуска: {}",
            queued.len()
        );
    }

    if !history_observations.is_empty() {
        let before = queued.len();
        queued.extend(history_observations);
        queued = newest_per_target(queued);
        if let Err(error) = observations.replace(&queued) {
            say_tr!(
                "could not write the observation queue: {error}",
                "не удалось записать очередь наблюдений: {error}"
            );
        }
        if first_run {
            say_tr!(
                "First run: sending the observation history — {} entries queued.",
                "Первый запуск: досылаю историю наблюдений — {} записей в очереди.",
                queued.len()
            );
        } else {
            say_tr!(
                "Observations added from history: {}",
                "Из истории добавлено наблюдений: {}",
                queued.len() - before
            );
        }
    }

    queued
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
