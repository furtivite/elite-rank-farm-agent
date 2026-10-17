use std::path::Path;
use std::time::Duration;

use agent_core::journal::{read_cargo_tons, read_status};
use agent_core::{CommanderState, CommanderStateBuilder};
use agent_sync::{
    send_with_retry, PendingState, RetryPolicy, SendAccepted, SendError, SendPace, ShipContext,
    StateClient,
};

use super::pace::learn_pace;
use crate::say;

pub(super) fn update_line(current_version: Option<&str>) -> String {
    match current_version {
        Some(version) => format!(
            "Вышла новая версия агента: {version} (у вас {}). Скачайте её перед продолжением.",
            agent_core::AGENT_VERSION,
        ),
        None => "Вышла новая версия агента — скачайте её перед продолжением.".to_owned(),
    }
}

pub(super) fn send(
    client: &StateClient,
    pending: &PendingState,
    state: &CommanderState,
    pace: &mut SendPace,
) -> Result<SendAccepted, SendError> {
    if let Err(error) = pending.store(state) {
        say!("не удалось записать локальную очередь: {error}");
    }

    let mut on_retry = |error: &SendError, delay: Duration| {
        say!("  попытка не прошла: {error}");
        say!("  повтор через {} с", delay.as_secs());
        learn_pace(pace, error, "отправки");
    };

    let accepted = send_with_retry(client, state, &RetryPolicy::default(), &mut on_retry)?;

    if let Err(error) = pending.clear() {
        say!("состояние отправлено, но очередь не очистилась: {error}");
    }

    Ok(accepted)
}

pub(super) fn read_ship(journal_dir: &Path, builder: &mut CommanderStateBuilder) -> ShipContext {
    let status = read_status(journal_dir, builder.session_started_at());
    builder.set_fuel_tons(status.fuel_main_t);
    builder.set_cargo_tons(
        status
            .cargo_t
            .or_else(|| read_cargo_tons(journal_dir, builder.session_started_at())),
    );

    ShipContext {
        parked: status.is_parked(),
        fsd_charging: status.is_fsd_charging(),
        low_fuel: status.is_low_fuel(),
    }
}
