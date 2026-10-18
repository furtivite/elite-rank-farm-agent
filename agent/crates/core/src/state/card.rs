use super::CommanderStateBuilder;
use crate::lang::{lang, Lang};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommanderCard {
    pub cmdr: Option<String>,
    pub ship: Option<String>,
    pub suit: Option<String>,
    pub system: Option<String>,
    pub station: Option<String>,
}

impl CommanderStateBuilder {
    pub fn card(&self) -> CommanderCard {
        CommanderCard {
            cmdr: self.commander_name.clone(),
            ship: self.ship.as_ref().map(|ship| self.ship_name(ship)),
            suit: self
                .suit
                .as_ref()
                .map(|(symbol, localised)| suit_label(symbol, localised.as_deref(), lang())),
            system: self
                .current_system
                .as_ref()
                .map(|system| system.name.clone()),
            station: self.current_station.clone(),
        }
    }

    fn ship_name(&self, ship: &str) -> String {
        match &self.ship_label {
            Some((code, label)) if code.eq_ignore_ascii_case(ship) => label.clone(),
            _ => ship.to_owned(),
        }
    }
}

const SUITS: [(&str, &str, &str); 4] = [
    ("flightsuit", "Flight Suit", "Летный комбинезон"),
    ("utilitysuit", "Maverick Suit", "Комбинезон Maverick"),
    ("explorationsuit", "Artemis Suit", "Комбинезон Artemis"),
    ("tacticalsuit", "Dominator Suit", "Комбинезон Dominator"),
];

fn suit_label(symbol: &str, localised: Option<&str>, lang: Lang) -> String {
    let localised = localised.filter(|name| !name.starts_with('$'));
    let lower = symbol.to_lowercase();
    let base = lower.split("_class").next().unwrap_or(&lower);
    let known = SUITS.iter().find(|(key, _, _)| *key == base);

    match (lang, known) {
        (Lang::En, Some((_, english, _))) => (*english).to_owned(),
        (Lang::Ru, Some((_, _, russian))) => localised.unwrap_or(russian).to_owned(),
        (_, None) => localised.unwrap_or(symbol).to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_window_names_the_suit_in_english() {
        assert_eq!(
            suit_label("utilitysuit_class1", Some("Комбинезон Maverick"), Lang::En),
            "Maverick Suit"
        );
        assert_eq!(
            suit_label("flightsuit", Some("Летный комбинезон"), Lang::En),
            "Flight Suit"
        );
    }

    #[test]
    fn russian_window_keeps_the_game_translation() {
        assert_eq!(
            suit_label("utilitysuit_class1", Some("Комбинезон Maverick"), Lang::Ru),
            "Комбинезон Maverick"
        );
    }

    #[test]
    fn an_untranslated_suit_gets_a_real_name_in_both_languages() {
        let untranslated = Some("$ExplorationSuit_Class1_Name;");
        assert_eq!(
            suit_label("explorationsuit_class3", untranslated, Lang::En),
            "Artemis Suit"
        );
        assert_eq!(
            suit_label("explorationsuit_class3", untranslated, Lang::Ru),
            "Комбинезон Artemis"
        );
    }

    #[test]
    fn an_unknown_suit_is_shown_as_the_game_says() {
        assert_eq!(
            suit_label("newsuit_class1", Some("Новый"), Lang::En),
            "Новый"
        );
        assert_eq!(
            suit_label("newsuit_class1", Some("$New_Name;"), Lang::En),
            "newsuit_class1"
        );
    }
}
