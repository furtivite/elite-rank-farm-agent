#![allow(dead_code)]

use agent_core::journal::parse_line;
use agent_core::CommanderStateBuilder;

pub const BASELINE: &[&str] = &[
    r#"{"timestamp":"2026-09-01T20:00:00Z","event":"Commander","FID":"F1","Name":"Jameson"}"#,
    r#"{"timestamp":"2026-09-01T20:00:01Z","event":"Loadout","Ship":"diamondbackxl","MaxJumpRange":43.487682}"#,
    r#"{"timestamp":"2026-09-01T20:00:02Z","event":"Rank","Empire":7,"Federation":2}"#,
    r#"{"timestamp":"2026-09-01T20:00:03Z","event":"Progress","Empire":71,"Federation":100}"#,
    r#"{"timestamp":"2026-09-01T20:00:04Z","event":"Location","StarSystem":"Ngalinn","StarPos":[-33.65,72.46,-20.65],"Body":"Ngalinn","BodyType":"Star","Docked":false}"#,
];

pub fn apply(builder: &mut CommanderStateBuilder, lines: &[&str]) {
    for line in lines {
        builder.apply_line(&parse_line(line));
    }
}

pub fn baseline() -> CommanderStateBuilder {
    let mut builder = CommanderStateBuilder::new();
    apply(&mut builder, BASELINE);
    builder
}
