use crate::contract::PadSize;

const SHIP_PAD_SIZES: &[(&str, PadSize)] = &[
    ("sidewinder", PadSize::S),
    ("eagle", PadSize::S),
    ("hauler", PadSize::S),
    ("adder", PadSize::S),
    ("viper", PadSize::S),
    ("cobramkiii", PadSize::S),
    ("type6", PadSize::M),
    ("dolphin", PadSize::S),
    ("type7", PadSize::L),
    ("asp", PadSize::M),
    ("vulture", PadSize::S),
    ("empire_trader", PadSize::L),
    ("federation_dropship", PadSize::M),
    ("orca", PadSize::L),
    ("type9", PadSize::L),
    ("python", PadSize::M),
    ("belugaliner", PadSize::L),
    ("ferdelance", PadSize::M),
    ("anaconda", PadSize::L),
    ("federation_corvette", PadSize::L),
    ("cutter", PadSize::L),
    ("diamondback", PadSize::S),
    ("empire_courier", PadSize::S),
    ("diamondbackxl", PadSize::S),
    ("empire_eagle", PadSize::S),
    ("federation_dropship_mkii", PadSize::M),
    ("federation_gunship", PadSize::M),
    ("viper_mkiv", PadSize::S),
    ("cobramkiv", PadSize::S),
    ("independant_trader", PadSize::M),
    ("asp_scout", PadSize::M),
    ("type9_military", PadSize::L),
    ("krait_mkii", PadSize::M),
    ("typex", PadSize::M),
    ("typex_2", PadSize::M),
    ("typex_3", PadSize::M),
    ("krait_light", PadSize::M),
    ("mamba", PadSize::M),
    ("python_nx", PadSize::M),
    ("type8", PadSize::M),
    ("mandalay", PadSize::M),
    ("cobramkv", PadSize::S),
    ("corsair", PadSize::M),
    ("panthermkii", PadSize::L),
    ("lakonminer", PadSize::M),
    ("explorer_nx", PadSize::L),
    ("smallcombat01_nx", PadSize::S),
    ("mediumtransport01", PadSize::M),
];

pub fn pad_size_for_ship(journal_ship: &str) -> Option<PadSize> {
    let needle = journal_ship.trim().to_ascii_lowercase();
    SHIP_PAD_SIZES
        .iter()
        .find(|(symbol, _)| *symbol == needle)
        .map(|(_, pad)| *pad)
}

pub fn known_ships() -> impl Iterator<Item = (&'static str, PadSize)> {
    SHIP_PAD_SIZES.iter().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_pad_size_by_journal_symbol() {
        assert_eq!(pad_size_for_ship("SideWinder"), Some(PadSize::S));
        assert_eq!(pad_size_for_ship("Type6"), Some(PadSize::M));
        assert_eq!(pad_size_for_ship("Cutter"), Some(PadSize::L));
    }

    #[test]
    fn lookup_ignores_case_because_the_game_is_inconsistent() {
        assert_eq!(pad_size_for_ship("python"), pad_size_for_ship("Python"));
        assert_eq!(pad_size_for_ship("TYPE8"), Some(PadSize::M));
        assert_eq!(pad_size_for_ship("mediumtransport01"), Some(PadSize::M));
        assert_eq!(pad_size_for_ship("SmallCombat01_NX"), Some(PadSize::S));
    }

    #[test]
    fn unknown_hull_is_none_not_a_guess() {
        assert_eq!(pad_size_for_ship("empire_fighter"), None);
        assert_eq!(pad_size_for_ship("testbuggy"), None);
        assert_eq!(pad_size_for_ship(""), None);
    }

    #[test]
    fn table_has_no_duplicate_symbols() {
        let mut symbols: Vec<&str> = SHIP_PAD_SIZES.iter().map(|(s, _)| *s).collect();
        symbols.sort_unstable();
        let total = symbols.len();
        symbols.dedup();
        assert_eq!(symbols.len(), total, "в таблице есть дубликаты symbol");
    }

    #[test]
    fn table_is_normalized_for_case_insensitive_lookup() {
        for (symbol, _) in SHIP_PAD_SIZES {
            assert_eq!(
                *symbol,
                symbol.to_ascii_lowercase(),
                "ключ {symbol} не в нижнем регистре — поиск его не найдёт"
            );
        }
    }

    #[test]
    fn covers_every_known_hull() {
        assert_eq!(SHIP_PAD_SIZES.len(), 48);

        let count = |pad: PadSize| {
            SHIP_PAD_SIZES
                .iter()
                .filter(|(_, value)| *value == pad)
                .count()
        };
        assert_eq!(
            (count(PadSize::S), count(PadSize::M), count(PadSize::L)),
            (16, 21, 11)
        );
    }
}
