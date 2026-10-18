use std::path::Path;
use std::time::Duration;

use agent_core::journal::{read_cargo_tons, read_status};
use agent_core::{CommanderState, CommanderStateBuilder};
use agent_sync::{
    send_with_retry, PendingState, RetryPolicy, SendAccepted, SendError, SendPace, ShipContext,
    StateClient, RELEASES_URL,
};

use super::pace::learn_pace;
use crate::say_tr;

pub(super) fn update_line(current_version: Option<&str>) -> String {
    let notice = match current_version {
        Some(version) => agent_core::tr!(
            "A new version of the agent is out: {version} (you have {}). Download it before you continue.",
            "Вышла новая версия агента: {version} (у вас {}). Скачайте её перед продолжением.",
            agent_core::AGENT_VERSION,
        ),
        None => agent_core::tr!(
            "A new version of the agent is out — download it before you continue.",
            "Вышла новая версия агента — скачайте её перед продолжением."
        ),
    };
    format!("{notice}\n{RELEASES_URL}")
}

pub(super) fn send(
    client: &StateClient,
    pending: &PendingState,
    state: &CommanderState,
    pace: &mut SendPace,
) -> Result<SendAccepted, SendError> {
    if let Err(error) = pending.store(state) {
        say_tr!(
            "could not write the local queue: {error}",
            "не удалось записать локальную очередь: {error}"
        );
    }

    let mut on_retry = |error: &SendError, delay: Duration| {
        say_tr!("  attempt failed: {error}", "  попытка не прошла: {error}");
        say_tr!("  retry in {} s", "  повтор через {} с", delay.as_secs());
        learn_pace(pace, error, &agent_core::tr!("sending", "отправки"));
    };

    let accepted = send_with_retry(client, state, &RetryPolicy::default(), &mut on_retry)?;

    if let Err(error) = pending.clear() {
        say_tr!(
            "the state was sent, but the queue was not cleared: {error}",
            "состояние отправлено, но очередь не очистилась: {error}"
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_update_line_says_where_to_download() {
        for version in [Some("9.9.9"), None] {
            let line = update_line(version);
            assert!(line.ends_with(RELEASES_URL), "{line}");
        }
        assert!(update_line(Some("9.9.9")).contains("9.9.9"));
    }
}
