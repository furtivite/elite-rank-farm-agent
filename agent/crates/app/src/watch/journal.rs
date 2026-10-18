use std::collections::BTreeSet;
use std::path::Path;

use agent_core::journal::{
    journal_files, missions::MissionLog, newest_per_target, observations_from, GameSession,
    HistoryStore, JournalHistory, JournalPosition, JournalTailer, JournalWatcher, Observation,
    ParsedLine, PositionStore,
};
use agent_core::CommanderStateBuilder;

use crate::sink::{self, Status};
use crate::{say, say_tr};

pub(super) fn seed_from_history(
    dir: &Path,
    history: &mut JournalHistory,
) -> Result<SeededHistory, String> {
    let files = journal_files(dir).map_err(|error| {
        sink::status(Status::Game(false));
        format!("{}: {error}", dir.display())
    })?;
    let Some(latest) = files.last().cloned() else {
        sink::status(Status::Game(false));
        return Err(agent_core::tr!(
            "{}: no Journal*.log files",
            "{}: нет файлов Journal*.log",
            dir.display()
        ));
    };
    sink::status(Status::Game(true));

    let mut builder = CommanderStateBuilder::new();
    let mut missions = MissionLog::new();
    let mut latest_offset = 0;
    let mut history_observations = Vec::new();
    let mut files_backfilled = 0;
    let mut latest_session = GameSession::new();
    let mut refused: BTreeSet<String> = BTreeSet::new();

    for path in &files {
        let mut tailer = JournalTailer::new(path.clone());
        let is_latest = *path == latest;
        match tailer.read_new() {
            Ok(lines) => {
                for line in &lines {
                    builder.apply_line(line);
                    missions.apply_line(line);
                    if is_latest {
                        latest_session.apply_line(line);
                    }
                }
            }
            Err(error) => say_tr!(
                "{}: read error: {error}",
                "{}: ошибка чтения: {error}",
                path.display()
            ),
        }
        if *path == latest {
            latest_offset = tailer.offset();
        }

        let Some(file_name) = file_name_of(path) else {
            continue;
        };
        let read_from = history.offset_of(&file_name);
        let size = tailer.offset();
        if read_from >= size {
            continue;
        }

        let mut file_session = GameSession::from_file(path);

        let mut unread = JournalTailer::resume_at(path.clone(), read_from);
        match unread.read_new() {
            Ok(lines) => {
                let before = history_observations.len();
                for line in &lines {
                    file_session.apply_line(line);
                    if let (ParsedLine::Event(event), true) =
                        (line, file_session.accepts_observations())
                    {
                        let mut from_line = observations_from(event);
                        for observation in &mut from_line {
                            observation.stamp_game(&file_session);
                        }
                        history_observations.extend(from_line);
                    }
                }
                if let Some(reason) = file_session.rejection() {
                    refused.insert(reason.to_string());
                }
                if history_observations.len() > before {
                    files_backfilled += 1;
                }
                history.record(&file_name, unread.offset());
            }
            Err(error) => say_tr!(
                "{}: history read error: {error}",
                "{}: ошибка чтения истории: {error}",
                path.display()
            ),
        }
    }

    let existing: Vec<String> = files.iter().filter_map(|path| file_name_of(path)).collect();
    history.retain_existing(&existing);

    let missing = builder.missing();
    if missing.is_empty() {
        say_tr!(
            "The state is built from the journal history.",
            "Состояние собрано по истории журналов."
        );
    } else {
        let listed: Vec<String> = missing
            .iter()
            .map(|field| {
                agent_core::tr!(
                    "{field:?} (event {})",
                    "{field:?} (событие {})",
                    field.filled_by()
                )
            })
            .collect();
        say_tr!(
            "The state is still incomplete, waiting for events: {}",
            "Состояние пока неполное, ждём события: {}",
            listed.join(", ")
        );
    }

    let before_collapse = history_observations.len();
    let history_observations = newest_per_target(history_observations);
    if !history_observations.is_empty() {
        say_tr!(
            "From history ({files_backfilled} journals): {} observations after collapsing repeats, was {before_collapse}.",
            "Из истории ({files_backfilled} журн.): {} наблюдений после схлопывания повторов, было {before_collapse}.",
            history_observations.len()
        );
    }

    if !refused.is_empty() {
        let listed: Vec<String> = refused.into_iter().collect();
        say_tr!(
            "Some journals gave no observations: {}. This is not an error — their data is not about our galaxy (115.1).",
            "Часть журналов не дала наблюдений: {}. Это не ошибка — данные оттуда не про нашу галактику (115.1).",
            listed.join(", ")
        );
    }

    Ok(SeededHistory {
        builder,
        missions,
        watcher: JournalWatcher::resume(dir.to_path_buf(), latest, latest_offset),
        session: latest_session,
        history_observations,
    })
}

pub(super) fn file_name_of(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_owned())
}

pub(super) struct SeededHistory {
    pub(super) builder: CommanderStateBuilder,
    pub(super) missions: MissionLog,
    pub(super) watcher: JournalWatcher,
    pub(super) session: GameSession,
    pub(super) history_observations: Vec<Observation>,
}

pub(super) fn save_position(
    store: &PositionStore,
    watcher: &JournalWatcher,
    previous: Option<JournalPosition>,
) -> Option<JournalPosition> {
    let Some((path, offset)) = watcher.position() else {
        return previous;
    };
    let current = JournalPosition {
        path: path.to_path_buf(),
        offset,
    };
    if previous.as_ref() == Some(&current) {
        return previous;
    }
    if let Err(error) = store.store(&current) {
        say_tr!(
            "could not save the journal position: {error}",
            "не удалось сохранить позицию журнала: {error}"
        );
    }
    Some(current)
}

pub(super) fn store_history(store: &HistoryStore, history: &JournalHistory) {
    if let Err(error) = store.store(history) {
        say_tr!(
            "could not save the journal history: {error}",
            "не удалось сохранить историю журналов: {error}"
        );
    }
}

pub(super) fn apply_lines(
    lines: &[ParsedLine],
    session: &mut GameSession,
    builder: &mut CommanderStateBuilder,
    missions: &mut MissionLog,
    said: &mut BTreeSet<String>,
) -> Vec<Observation> {
    let mut fresh = Vec::new();
    for line in lines {
        session.apply_line(line);
        match line {
            ParsedLine::Unparsed { reason } => {
                say_tr!("line not parsed: {reason}", "строка не разобрана: {reason}");
            }
            ParsedLine::Event(event) if session.accepts_observations() => {
                fresh.extend(observations_from(event))
            }
            ParsedLine::Event(_) | ParsedLine::Ignored => {}
        }
        builder.apply_line(line);
        missions.apply_line(line);
    }

    if let Some(reason) = session.rejection() {
        let notice = agent_core::tr!(
            "observations are not collected: {reason}",
            "наблюдения не собираются: {reason}"
        );
        if said.insert(notice.clone()) {
            say!("{notice}");
        }
    }

    for observation in &mut fresh {
        observation.stamp_game(session);
    }

    fresh
}
