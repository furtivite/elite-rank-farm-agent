use std::path::{Path, PathBuf};

use agent_core::Lang;

pub const SETTINGS_FILE_NAME: &str = "settings.json";

pub fn settings_path() -> PathBuf {
    crate::dir::data_dir().join(SETTINGS_FILE_NAME)
}

pub fn load(path: &Path) -> Lang {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|value| value.get("language")?.as_str().and_then(Lang::from_code))
        .unwrap_or_default()
}

pub fn save(path: &Path, lang: Lang) -> Result<(), String> {
    let mut settings = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .filter(serde_json::Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}));
    settings["language"] = serde_json::Value::from(lang.code());

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    std::fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("elite-agent-language-{name}"));
        std::fs::remove_dir_all(&dir).ok();
        dir.join(SETTINGS_FILE_NAME)
    }

    #[test]
    fn without_a_file_the_language_is_english() {
        assert_eq!(load(&temp_file("missing")), Lang::En);
    }

    #[test]
    fn russian_survives_a_restart() {
        let path = temp_file("roundtrip");
        save(&path, Lang::Ru).expect("настройки пишутся");
        assert_eq!(load(&path), Lang::Ru);

        save(&path, Lang::En).expect("настройки пишутся");
        assert_eq!(load(&path), Lang::En);
    }

    #[test]
    fn a_broken_file_means_english() {
        let path = temp_file("broken");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        for text in ["не json", r#"{"language":"de"}"#, r#"{"language":1}"#, "[]"] {
            std::fs::write(&path, text).unwrap();
            assert_eq!(load(&path), Lang::En, "{text}");
        }
    }

    #[test]
    fn other_settings_are_kept() {
        let path = temp_file("kept");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, r#"{"later":true}"#).unwrap();

        save(&path, Lang::Ru).expect("настройки пишутся");

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"later\": true"), "{text}");
        assert_eq!(load(&path), Lang::Ru);
    }
}
