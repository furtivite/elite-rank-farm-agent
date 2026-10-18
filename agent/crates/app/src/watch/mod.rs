use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use agent_core::journal::{
    read_free_cargo, GameSession, HistoryStore, JournalHistory, PositionStore,
};
use agent_core::CommanderState;
use agent_sync::{
    decide_for_ship, ObservationQueue, PendingState, SendDecision, SendPace, StateClient,
    SyncConfig,
};
use finish::{finish, Leftovers};
use journal::{
    apply_lines, file_name_of, save_position, seed_from_history, store_history, SeededHistory,
};
use missions::send_missions;
use observations::{flush_observations, load_queue, FlushOutcome};
use rebuild::report_rebuild;
use report::{report_status, token_rejected};
use state::{read_ship, send, update_line};

use crate::dir::data_dir;
use crate::sink::{self, Status};
use crate::{say, say_tr};

mod finish;
mod journal;
mod missions;
mod observations;
mod pace;
mod rebuild;
mod report;
mod resync;
mod state;

pub use rebuild::rebuild_only;
pub use resync::resync_until;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

const OBSERVATIONS_INTERVAL: Duration = Duration::from_secs(5);

const MISSIONS_INTERVAL: Duration = Duration::from_secs(5);

pub fn run(journal_dir: &Path) -> Result<(), String> {
    crate::shutdown::install();
    let result = run_until(journal_dir, crate::shutdown::running_flag());
    crate::shutdown::mark_finished();
    result
}

pub fn run_until(journal_dir: &Path, running: &AtomicBool) -> Result<(), String> {
    let config = SyncConfig::load().map_err(|error| error.to_string())?;
    let client = StateClient::new(&config);
    let pending = PendingState::in_dir(&data_dir());
    let positions = PositionStore::in_dir(&data_dir());
    let observations = ObservationQueue::in_dir(&data_dir());

    say_tr!("Sending to {}", "Отправка на {}", config.state_url());

    let histories = HistoryStore::in_dir(&data_dir());
    let mut history = match histories.load() {
        Ok(loaded) => loaded,
        Err(error) => {
            say_tr!(
                "the journal history could not be read ({error}), parsing again",
                "история журналов не прочиталась ({error}), разбираю заново"
            );
            JournalHistory::default()
        }
    };
    let first_run = history.is_empty();

    let SeededHistory {
        mut builder,
        mut missions,
        mut watcher,
        mut session,
        history_observations,
    } = seed_from_history(journal_dir, &mut history)?;
    let mut saved_position = save_position(&positions, &watcher, None);

    let mut last_sent: Option<CommanderState> = None;
    let mut last_send_at: Option<Instant> = None;
    let mut pace = SendPace::default();
    let mut rebuild_requested = false;
    let mut said_warnings: BTreeSet<String> = BTreeSet::new();
    let mut session_file = watcher.position().map(|(path, _)| path.to_path_buf());
    let mut rebuild_pending = false;
    let mut update_reported = false;
    let mut last_observations_at: Option<Instant> = None;
    let mut observations_pace = SendPace::new(OBSERVATIONS_INTERVAL);
    let mut last_missions_sent: Option<agent_core::contract::ActiveMissionsPayload> = None;
    let mut last_missions_at: Option<Instant> = None;
    let mut missions_pace = SendPace::new(MISSIONS_INTERVAL);
    let mut last_status_line: Option<String> = None;
    let mut last_status_at = Instant::now();
    let mut last_card = None;
    let mut queued = load_queue(&observations, history_observations, first_run);

    store_history(&histories, &history);

    say_tr!(
        "Watching the journal. Ctrl+C to quit.\n",
        "Слежу за журналом. Ctrl+C — выход.\n"
    );

    while running.load(Ordering::SeqCst) {
        match watcher.poll() {
            Ok(lines) => {
                if let Some((path, _)) = watcher.position() {
                    if session_file.as_deref() != Some(path) {
                        session = GameSession::from_file(path);
                        session_file = Some(path.to_path_buf());
                    }
                }

                let fresh = apply_lines(
                    &lines,
                    &mut session,
                    &mut builder,
                    &mut missions,
                    &mut said_warnings,
                );

                if !fresh.is_empty() {
                    if let Err(error) = observations.append(&fresh) {
                        say_tr!(
                            "could not write the observation queue: {error}",
                            "не удалось записать очередь наблюдений: {error}"
                        );
                    }
                    queued.extend(fresh);
                }
            }
            Err(error) => {
                say_tr!(
                    "journal read error: {error}",
                    "ошибка чтения журнала: {error}"
                );
            }
        }
        let moved = save_position(&positions, &watcher, saved_position.clone());
        if moved != saved_position {
            if let Some(position) = moved.as_ref() {
                if let Some(name) = file_name_of(&position.path) {
                    history.record(&name, position.offset);
                    store_history(&histories, &history);
                }
            }
            saved_position = moved;
        }

        if !queued.is_empty()
            && last_observations_at.is_none_or(|at| at.elapsed() >= observations_pace.interval())
        {
            last_observations_at = Some(Instant::now());

            match flush_observations(&client, &observations, &mut queued, &mut observations_pace) {
                FlushOutcome::Applied => rebuild_pending = true,
                FlushOutcome::Nothing => {}
                FlushOutcome::TokenRejected(message) => return Err(token_rejected(&message)),
            }
        }

        if rebuild_pending && queued.is_empty() {
            rebuild_pending = false;
            rebuild_requested = true;
            report_rebuild(&client, &mut said_warnings);
        }

        missions.set_free_cargo(read_free_cargo(journal_dir, builder.session_started_at()));
        send_missions(
            &client,
            &missions,
            &mut last_missions_sent,
            &mut last_missions_at,
            &mut missions_pace,
        )?;

        report_status(
            &builder,
            &missions,
            &mut last_status_line,
            &mut last_status_at,
            &mut last_card,
        );

        let ship = read_ship(journal_dir, &mut builder);

        let current = builder.build();
        let elapsed = last_send_at.map(|instant| instant.elapsed());

        match decide_for_ship(
            current.as_ref(),
            last_sent.as_ref(),
            elapsed,
            pace.interval(),
            ship,
        ) {
            SendDecision::Send => {
                let state = current.expect("Send выдаётся только при собранном состоянии");
                match send(&client, &pending, &state, &mut pace) {
                    Ok(accepted) => {
                        let updated_at = accepted.updated_at;
                        sink::status(Status::Connection(true));
                        sink::status(Status::LastSync(updated_at.clone()));
                        say_tr!(
                            "{} · {} → sent, updatedAt = {updated_at}",
                            "{} · {} → отправлено, updatedAt = {updated_at}",
                            state.current_system.name,
                            state
                                .current_station
                                .clone()
                                .unwrap_or_else(|| agent_core::tr!("in space", "в космосе")),
                        );

                        if !update_reported {
                            update_reported = true;
                            if let Some(update) = accepted.agent_update {
                                say!("{}", update_line(update.current_version.as_deref()));
                            }
                        }
                        last_sent = Some(state);
                        last_send_at = Some(Instant::now());

                        if !rebuild_requested && queued.is_empty() {
                            rebuild_requested = true;
                            report_rebuild(&client, &mut said_warnings);
                        }
                    }
                    Err(error) if error.is_retryable() => {
                        sink::status(Status::Connection(false));
                        say_tr!("not sent: {error}", "не отправлено: {error}");
                        last_send_at = Some(Instant::now());
                    }
                    Err(error) if error.is_token_rejected() => {
                        sink::status(Status::Connection(false));
                        return Err(token_rejected(&error.to_string()));
                    }
                    Err(error) => {
                        sink::status(Status::Connection(false));
                        return Err(agent_core::tr!(
                            "{error}\nThe loop is stopped.",
                            "{error}\nЦикл остановлен."
                        ));
                    }
                }
            }
            SendDecision::Unchanged | SendDecision::TooSoon(_) | SendDecision::NotReady => {}
        }

        std::thread::sleep(POLL_INTERVAL);
    }

    finish(
        &client,
        &pending,
        &observations,
        &mut pace,
        Leftovers {
            builder: &builder,
            queued: &mut queued,
            observations_pace: &mut observations_pace,
            rebuild_pending,
            said_warnings: &mut said_warnings,
        },
    );

    Ok(())
}
