use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim() {
            "en" => Some(Self::En),
            "ru" => Some(Self::Ru),
            _ => None,
        }
    }
}

static RUSSIAN: AtomicBool = AtomicBool::new(false);

pub fn set_lang(lang: Lang) {
    RUSSIAN.store(lang == Lang::Ru, Ordering::Relaxed);
}

pub fn lang() -> Lang {
    if RUSSIAN.load(Ordering::Relaxed) {
        Lang::Ru
    } else {
        Lang::En
    }
}

#[macro_export]
macro_rules! tr {
    ($en:literal, $ru:literal $(, $arg:expr)* $(,)?) => {
        match $crate::lang::lang() {
            $crate::lang::Lang::En => format!($en $(, $arg)*),
            $crate::lang::Lang::Ru => format!($ru $(, $arg)*),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_read_back() {
        for lang in [Lang::En, Lang::Ru] {
            assert_eq!(Lang::from_code(lang.code()), Some(lang));
        }
        assert_eq!(Lang::from_code("de"), None);
    }

    #[test]
    fn english_is_the_default() {
        assert_eq!(Lang::default(), Lang::En);
    }
}
