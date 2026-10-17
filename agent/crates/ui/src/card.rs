use crate::CommanderLines;

pub(crate) fn card_text(lines: &CommanderLines) -> String {
    [
        ("CMDR", &lines.cmdr),
        ("SHIP", &lines.ship),
        ("SUIT", &lines.suit),
        ("SYSTEM", &lines.system),
        ("STATION", &lines.station),
    ]
    .iter()
    .map(|(label, value)| format!("{label}: {}", value.as_deref().unwrap_or("—")))
    .collect::<Vec<_>>()
    .join("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_is_there_even_before_the_journal_said_anything() {
        assert_eq!(
            card_text(&CommanderLines::default()),
            "CMDR: —\r\nSHIP: —\r\nSUIT: —\r\nSYSTEM: —\r\nSTATION: —"
        );
    }

    #[test]
    fn known_values_are_shown_as_is() {
        let text = card_text(&CommanderLines {
            cmdr: Some("Jameson".into()),
            ship: Some("Panther Clipper Mk II".into()),
            suit: Some("Летный комбинезон".into()),
            system: Some("Ngalinn".into()),
            station: None,
        });

        assert!(text.starts_with("CMDR: Jameson\r\nSHIP: Panther Clipper Mk II\r\n"));
        assert!(text.ends_with("SYSTEM: Ngalinn\r\nSTATION: —"));
    }
}
