use std::path::Path;

use agent_core::CommanderState;
use agent_sync::{send_with_retry, PendingState, RetryPolicy, StateClient, SyncConfig};

use crate::dir::data_dir;
use crate::probe;

pub fn run(journal_dir: &Path) -> Result<(), String> {
    let config = SyncConfig::load().map_err(|error| error.to_string())?;
    println!("Отправка на {}", config.state_url());

    let pending = PendingState::in_dir(&data_dir());
    let state = choose_state(journal_dir, &pending)?;

    pending
        .store(&state)
        .map_err(|error| format!("не удалось записать локальную очередь: {error}"))?;

    let client = StateClient::new(&config);
    let mut on_retry = |error: &agent_sync::SendError, delay: std::time::Duration| {
        println!("  попытка не прошла: {error}");
        println!("  повтор через {} с", delay.as_secs());
        std::thread::sleep(delay);
    };

    match send_with_retry(&client, &state, &RetryPolicy::default(), &mut on_retry) {
        Ok(accepted) => {
            pending.clear().map_err(|error| {
                format!("состояние отправлено, но очередь не очистилась: {error}")
            })?;
            println!("Принято сервером, updatedAt = {}", accepted.updated_at);
            Ok(())
        }
        Err(error) if error.is_retryable() => Err(format!(
            "{error}\nСостояние сохранено в {} и будет отправлено позже.",
            pending.path().display()
        )),
        Err(error) => Err(match error.hint() {
            Some(hint) => format!("{error}\n{hint}"),
            None => format!("{error}\nПовтор не поможет: проверьте baseUrl и device token."),
        }),
    }
}

fn choose_state(journal_dir: &Path, pending: &PendingState) -> Result<CommanderState, String> {
    match probe::collect(journal_dir, false) {
        Ok(state) => Ok(state),
        Err(reason) => {
            let queued = pending
                .load()
                .map_err(|error| format!("локальная очередь повреждена: {error}"))?;
            match queued {
                Some(state) => {
                    println!("Журналы состояние не дали ({reason}); отправляю отложенное.");
                    Ok(state)
                }
                None => Err(reason),
            }
        }
    }
}
