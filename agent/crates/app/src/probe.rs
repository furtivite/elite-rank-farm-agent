use std::path::Path;

use agent_core::journal::{journal_files, read_cargo_tons, read_status, JournalTailer, ParsedLine};
use agent_core::{CommanderState, CommanderStateBuilder};

use crate::say;
use crate::sink::{self, Status};

pub fn collect(dir: &Path, verbose: bool) -> Result<CommanderState, String> {
    let files = journal_files(dir).map_err(|error| {
        sink::status(Status::Game(false));
        format!("{}: {error}", dir.display())
    })?;
    if files.is_empty() {
        sink::status(Status::Game(false));
        return Err(format!("{}: нет файлов Journal*.log", dir.display()));
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
                say!("{}: ошибка чтения: {error}", path.display());
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
        say!("Найдено журналов: {}\n", files.len());
        say!("Событий применено: {applied}");
        say!("Строк пропущено (события вне MVP): {ignored}");
        say!("Строк не разобрано: {}", unparsed.len());
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
            .map(|field| format!("{field:?} (событие {})", field.filled_by()))
            .collect();
        return Err(format!(
            "состояние неполное, не хватает: {}",
            listed.join(", ")
        ));
    }

    builder
        .build()
        .ok_or_else(|| "состояние не собралось при пустом списке недостающих полей".to_owned())
}

pub fn report(state: &CommanderState) {
    match serde_json::to_string_pretty(state) {
        Ok(json) => say!("Commander State (payload PUT /api/elite/state):\n{json}"),
        Err(error) => say!("не удалось сериализовать состояние: {error}"),
    }

    let violations = state.validate();
    if violations.is_empty() {
        say!("\nКонтракт: нарушений нет.");
    } else {
        say!("\nКонтракт нарушен:");
        for violation in &violations {
            say!("  - {violation}");
        }
    }
}
