use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::event::JournalEvent;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct JournalLine {
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub event: JournalEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedLine {
    Event(Box<JournalLine>),
    Ignored,
    Unparsed { reason: String },
}

pub fn parse_line(line: &str) -> ParsedLine {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return ParsedLine::Ignored;
    }
    match serde_json::from_str::<JournalLine>(trimmed) {
        Ok(parsed) => {
            if parsed.event == JournalEvent::Other {
                ParsedLine::Ignored
            } else {
                ParsedLine::Event(Box::new(parsed))
            }
        }
        Err(error) => ParsedLine::Unparsed {
            reason: error.to_string(),
        },
    }
}
