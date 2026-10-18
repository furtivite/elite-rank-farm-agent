use std::path::Path;

use agent_core::journal::{journal_files, read_cargo_tons, read_status, JournalTailer, ParsedLine};
use agent_core::{CommanderState, CommanderStateBuilder};

use crate::sink::{self, Status};
use crate::{say, say_tr};

pub fn collect(dir: &Path, verbose: bool) -> Result<CommanderState, String> {
    let files = journal_files(dir).map_err(|error| {
        sink::status(Status::Game(false));
        format!("{}: {error}", dir.display())
    })?;
    if files.is_empty() {
        sink::status(Status::Game(false));
        return Err(agent_core::tr!(
            "{}: no Journal*.log files",
            "{}: нет файлов Journal*.log",
            dir.display()
        ));
    }
    sink::status(Status::Game(true));

    let mut builder = CommanderStateBuilder::new();
    let mut applied = 0_usize;
    let mut ignored = 0_usize;
    let mut unparsed: Vec<String> = Vec::new();

    for path in &files {
        let mut tailer = JournalTailer::new(path.clone());
        let lines = match tailer.read_new() {
            Ok(lines) => lines,
            Err(error) => {
                say_tr!(
                    "{}: read error: {error}",
                    "{}: ошибка чтения: {error}",
                    path.display()
                );
                continue;
            }
        };
        for line in &lines {
            match line {
                ParsedLine::Event(_) => applied += 1,
                ParsedLine::Ignored => ignored += 1,
                ParsedLine::Unparsed { reason } => {
                    if unparsed.len() < 10 {
                        unparsed.push(format!("{}: {reason}", path.display()));
                    }
                }
            }
            builder.apply_line(line);
        }
    }

    if verbose {
        say_tr!(
            "Journals found: {}\n",
            "Найдено журналов: {}\n",
            files.len()
        );
        say_tr!("Events applied: {applied}", "Событий применено: {applied}");
        say_tr!(
            "Lines skipped (events outside MVP): {ignored}",
            "Строк пропущено (события вне MVP): {ignored}"
        );
        say_tr!(
            "Lines not parsed: {}",
            "Строк не разобрано: {}",
            unparsed.len()
        );
        for reason in &unparsed {
            say!("  - {reason}");
        }
        say!();
    }

    let status = read_status(dir, builder.session_started_at());
    builder.set_fuel_tons(status.fuel_main_t);
    builder.set_cargo_tons(
        status
            .cargo_t
            .or_else(|| read_cargo_tons(dir, builder.session_started_at())),
    );

    sink::status(Status::Commander(builder.card()));

    let missing = builder.missing();
    if !missing.is_empty() {
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
        return Err(agent_core::tr!(
            "the state is incomplete, missing: {}",
            "состояние неполное, не хватает: {}",
            listed.join(", ")
        ));
    }

    builder.build().ok_or_else(|| {
        agent_core::tr!(
            "the state did not build although no fields are missing",
            "состояние не собралось при пустом списке недостающих полей"
        )
    })
}

pub fn report(state: &CommanderState) {
    match serde_json::to_string_pretty(state) {
        Ok(json) => say!("Commander State (payload PUT /api/elite/state):\n{json}"),
        Err(error) => say_tr!(
            "could not serialize the state: {error}",
            "не удалось сериализовать состояние: {error}"
        ),
    }

    let violations = state.validate();
    if violations.is_empty() {
        say_tr!("\nContract: no violations.", "\nКонтракт: нарушений нет.");
    } else {
        say_tr!("\nContract violated:", "\nКонтракт нарушен:");
        for violation in &violations {
            say!("  - {violation}");
        }
    }
}
