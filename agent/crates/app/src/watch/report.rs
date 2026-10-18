use std::time::{Duration, Instant};

use agent_core::journal::missions::MissionLog;
use agent_core::{CommanderCard, CommanderStateBuilder};
use agent_sync::token_hint;

use crate::sink::{self, Status};
use crate::{say, say_tr};

const ALIVE_INTERVAL: Duration = Duration::from_secs(300);

pub(super) fn status_summary(builder: &CommanderStateBuilder, missions: &MissionLog) -> String {
    let Some(state) = builder.build() else {
        return agent_core::tr!("waiting for journal data", "жду данных из журнала");
    };

    let where_is = match &state.current_station {
        Some(station) => format!("{} · {station}", state.current_system.name),
        None => agent_core::tr!("{} · in space", "{} · в космосе", state.current_system.name),
    };

    let active = missions.active().len();

    agent_core::tr!(
        "{where_is} | {} | missions: {active} | Empire {}%, Federation {}%",
        "{where_is} | {} | заданий: {active} | Империя {}%, Федерация {}%",
        state.ship,
        state.imperial_progress,
        state.federal_progress
    )
}

pub(super) fn token_rejected(error: &str) -> String {
    let hint = token_hint();
    agent_core::tr!(
        "The server did not accept the agent key: {error}\n\
         {hint}\n\
         Nothing was sent. The collected observations are saved and will go out once \
         the key is fixed.\n\
         The agent is stopped.",
        "Сервер не принял ключ агента: {error}\n\
         {hint}\n\
         Ничего не отправлено. Накопленные наблюдения сохранены и уйдут, когда \
         ключ будет исправлен.\n\
         Агент остановлен."
    )
}

pub(super) fn report_status(
    builder: &CommanderStateBuilder,
    missions: &MissionLog,
    last_line: &mut Option<String>,
    last_at: &mut Instant,
    last_card: &mut Option<CommanderCard>,
) {
    let card = builder.card();
    if last_card.as_ref() != Some(&card) {
        sink::status(Status::Commander(card.clone()));
        *last_card = Some(card);
    }

    let status_line = status_summary(builder, missions);
    if last_line.as_ref() != Some(&status_line) {
        say!("{status_line}");
        *last_line = Some(status_line);
        *last_at = Instant::now();
    } else if last_at.elapsed() >= ALIVE_INTERVAL {
        say_tr!("working, no changes", "работаю, изменений нет");
        *last_at = Instant::now();
    }
}
