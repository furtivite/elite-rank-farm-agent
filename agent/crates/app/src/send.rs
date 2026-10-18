use std::path::Path;

use agent_core::CommanderState;
use agent_sync::{send_with_retry, PendingState, RetryPolicy, StateClient, SyncConfig};

use crate::dir::data_dir;
use crate::probe;

pub fn run(journal_dir: &Path) -> Result<(), String> {
    let config = SyncConfig::load().map_err(|error| error.to_string())?;
    println!(
        "{}",
        agent_core::tr!("Sending to {}", "Отправка на {}", config.state_url())
    );

    let pending = PendingState::in_dir(&data_dir());
    let state = choose_state(journal_dir, &pending)?;

    pending.store(&state).map_err(|error| {
        agent_core::tr!(
            "could not write the local queue: {error}",
            "не удалось записать локальную очередь: {error}"
        )
    })?;

    let client = StateClient::new(&config);
    let mut on_retry = |error: &agent_sync::SendError, delay: std::time::Duration| {
        println!(
            "{}",
            agent_core::tr!("  attempt failed: {error}", "  попытка не прошла: {error}")
        );
        println!(
            "{}",
            agent_core::tr!("  retry in {} s", "  повтор через {} с", delay.as_secs())
        );
        std::thread::sleep(delay);
    };

    match send_with_retry(&client, &state, &RetryPolicy::default(), &mut on_retry) {
        Ok(accepted) => {
            pending.clear().map_err(|error| {
                agent_core::tr!(
                    "the state was sent, but the queue was not cleared: {error}",
                    "состояние отправлено, но очередь не очистилась: {error}"
                )
            })?;
            println!(
                "{}",
                agent_core::tr!(
                    "Accepted by the server, updatedAt = {}",
                    "Принято сервером, updatedAt = {}",
                    accepted.updated_at
                )
            );
            Ok(())
        }
        Err(error) if error.is_retryable() => Err(agent_core::tr!(
            "{error}\nThe state is saved to {} and will be sent later.",
            "{error}\nСостояние сохранено в {} и будет отправлено позже.",
            pending.path().display()
        )),
        Err(error) => Err(match error.hint() {
            Some(hint) => format!("{error}\n{hint}"),
            None => agent_core::tr!(
                "{error}\nRetrying will not help: check baseUrl and the device token.",
                "{error}\nПовтор не поможет: проверьте baseUrl и device token."
            ),
        }),
    }
}

fn choose_state(journal_dir: &Path, pending: &PendingState) -> Result<CommanderState, String> {
    match probe::collect(journal_dir, false) {
        Ok(state) => Ok(state),
        Err(reason) => {
            let queued = pending.load().map_err(|error| {
                agent_core::tr!(
                    "the local queue is damaged: {error}",
                    "локальная очередь повреждена: {error}"
                )
            })?;
            match queued {
                Some(state) => {
                    println!(
                        "{}",
                        agent_core::tr!(
                            "The journals gave no state ({reason}); sending the saved one.",
                            "Журналы состояние не дали ({reason}); отправляю отложенное."
                        )
                    );
                    Ok(state)
                }
                None => Err(reason),
            }
        }
    }
}
