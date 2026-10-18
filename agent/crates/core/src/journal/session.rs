use std::path::Path;

use super::event::JournalEvent;
use super::parse::{parse_line, ParsedLine};

const LIVE_GALAXY_MAJOR: u32 = 4;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameSession {
    version: Option<String>,
    build: Option<String>,
    cqc: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRejection {
    LegacyGalaxy,
    TestBuild,
    Arena,
}

impl std::fmt::Display for SessionRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LegacyGalaxy => f.write_str(&crate::tr!("the Legacy galaxy", "галактика Legacy")),
            Self::TestBuild => {
                f.write_str(&crate::tr!("a game alpha or beta", "альфа или бета игры"))
            }
            Self::Arena => f.write_str(&crate::tr!("the CQC arena", "арена CQC")),
        }
    }
}

impl GameSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(path: &Path) -> Self {
        let mut session = Self::new();
        if let Some(line) = first_line(path) {
            session.apply_line(&parse_line(&line));
        }
        session
    }

    pub fn apply_line(&mut self, line: &ParsedLine) {
        let ParsedLine::Event(line) = line else {
            return;
        };
        match &line.event {
            JournalEvent::Fileheader {
                gameversion, build, ..
            } => {
                self.version = gameversion.clone();
                self.build = build.clone();
                self.cqc = false;
            }
            JournalEvent::LoadGame {
                game_mode,
                ship,
                gameversion,
                ..
            } => {
                if self.version.is_none() {
                    self.version = gameversion.clone();
                }
                self.cqc = is_arena(game_mode.as_deref(), ship.as_deref());
            }
            _ => {}
        }
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn build(&self) -> Option<&str> {
        self.build.as_deref()
    }

    pub fn rejection(&self) -> Option<SessionRejection> {
        if self.cqc {
            return Some(SessionRejection::Arena);
        }
        let version = self.version.as_deref()?;
        if is_test_build(version) {
            return Some(SessionRejection::TestBuild);
        }
        if !is_live_galaxy(version) {
            return Some(SessionRejection::LegacyGalaxy);
        }
        None
    }

    pub fn accepts_observations(&self) -> bool {
        self.rejection().is_none()
    }
}

fn is_arena(game_mode: Option<&str>, ship: Option<&str>) -> bool {
    match game_mode {
        Some(mode) => mode.eq_ignore_ascii_case("cqc"),
        None => ship.is_none(),
    }
}

fn is_live_galaxy(version: &str) -> bool {
    major_of(version).is_some_and(|major| major >= LIVE_GALAXY_MAJOR)
}

fn major_of(version: &str) -> Option<u32> {
    let digits: String = version
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn is_test_build(version: &str) -> bool {
    let lowered = version.to_ascii_lowercase();
    lowered.contains("alpha") || lowered.contains("beta")
}

fn first_line(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(path).ok()?;
    BufReader::new(file).lines().next()?.ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_from(lines: &[&str]) -> GameSession {
        let mut session = GameSession::new();
        for line in lines {
            session.apply_line(&parse_line(line));
        }
        session
    }

    const LIVE_HEADER: &str = r#"{"timestamp":"2026-09-09T20:00:00Z","event":"Fileheader",
        "part":1,"language":"Russian/RU","Odyssey":true,"gameversion":"4.1.2.0","build":"r317667/r0 "}"#;

    #[test]
    fn live_session_is_taken() {
        let session = session_from(&[LIVE_HEADER]);

        assert_eq!(session.version(), Some("4.1.2.0"));
        assert_eq!(session.build(), Some("r317667/r0 "));
        assert!(session.accepts_observations());
    }

    #[test]
    fn header_of_a_live_journal_is_read() {
        let session = session_from(&[
            r#"{ "timestamp":"2026-09-06T15:48:07Z", "event":"Fileheader", "part":1, "language":"Russian/RU", "Odyssey":true, "gameversion":"4.4.1.1", "build":"r332841/r0 " }"#,
        ]);

        assert_eq!(session.version(), Some("4.4.1.1"));
        assert_eq!(session.build(), Some("r332841/r0 "));
        assert!(session.accepts_observations());
    }

    #[test]
    fn legacy_galaxy_is_refused() {
        let session = session_from(&[
            r#"{"timestamp":"2026-09-09T20:00:00Z","event":"Fileheader","part":1,
                "language":"English/UK","gameversion":"3.8.0.201","build":"r255812/r0 "}"#,
        ]);

        assert_eq!(session.rejection(), Some(SessionRejection::LegacyGalaxy));
    }

    #[test]
    fn alpha_build_is_refused_even_on_a_live_version() {
        let session = session_from(&[
            r#"{"timestamp":"2026-09-09T20:00:00Z","event":"Fileheader","part":1,
                "gameversion":"4.0.0.100 Alpha","build":"r280105/r0 "}"#,
        ]);

        assert_eq!(session.rejection(), Some(SessionRejection::TestBuild));
    }

    #[test]
    fn arena_is_refused_until_the_next_journal() {
        let arena = session_from(&[
            LIVE_HEADER,
            r#"{"timestamp":"2026-09-09T20:01:00Z","event":"LoadGame","Commander":"Jameson",
                "GameMode":"CQC","Credits":1,"Loan":0}"#,
        ]);
        assert_eq!(arena.rejection(), Some(SessionRejection::Arena));

        let mut back = arena;
        back.apply_line(&parse_line(
            r#"{"timestamp":"2026-09-09T21:00:00Z","event":"LoadGame","Commander":"Jameson",
                "Ship":"cutter","GameMode":"Solo","Credits":1,"Loan":0}"#,
        ));
        assert!(back.accepts_observations());
    }

    #[test]
    fn load_game_without_mode_and_ship_is_an_arena() {
        let session = session_from(&[
            LIVE_HEADER,
            r#"{"timestamp":"2026-09-09T20:01:00Z","event":"LoadGame","Commander":"Jameson",
                "Credits":1,"Loan":0}"#,
        ]);

        assert_eq!(session.rejection(), Some(SessionRejection::Arena));
    }

    #[test]
    fn unknown_version_is_not_a_reason_to_stay_silent() {
        let session = session_from(&[
            r#"{"timestamp":"2026-09-09T20:00:00Z","event":"Fileheader","part":1,"language":"English/UK"}"#,
        ]);

        assert_eq!(session.version(), None);
        assert!(session.accepts_observations());
    }
}
