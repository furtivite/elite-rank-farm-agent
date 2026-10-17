use super::CommanderStateBuilder;

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
            suit: self.suit.clone(),
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
