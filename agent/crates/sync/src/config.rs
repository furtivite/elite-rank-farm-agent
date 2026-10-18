use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const CONFIG_FILE_NAME: &str = "elite-agent.json";

pub const ENV_BASE_URL: &str = "ELITE_AGENT_BASE_URL";

pub const DEFAULT_BASE_URL: &str = "https://frtvt.com/elite";
pub const ENV_DEVICE_TOKEN: &str = "ELITE_AGENT_DEVICE_TOKEN";

pub const RELEASES_URL: &str = "https://github.com/furtivite/elite-rank-farm-agent/releases/latest";

pub const STATE_PATH: &str = "/api/elite/state";

pub const REBUILD_PATH: &str = "/api/bootstrap/rebuild";

pub const OBSERVATIONS_PATH: &str = "/api/galaxy/observations";
const MISSIONS_PATH: &str = "/api/elite/missions";

#[derive(Debug)]
pub enum ConfigError {
    NotFound { looked_at: PathBuf },
    Unreadable { path: PathBuf, reason: String },
    Malformed { path: PathBuf, reason: String },
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { looked_at } => f.write_str(&agent_core::tr!(
                "settings not found: enter the agent key in the window \
                 (it will be saved to {}), set {ENV_BASE_URL} and {ENV_DEVICE_TOKEN} \
                 or put {CONFIG_FILE_NAME} there yourself",
                "конфигурация не найдена: укажите адрес сервера и ключ агента в окне \
                 (они сохранятся в {}), задайте {ENV_BASE_URL} и {ENV_DEVICE_TOKEN} \
                 либо положите туда {CONFIG_FILE_NAME} сами",
                looked_at.display()
            )),
            Self::Unreadable { path, reason } => f.write_str(&agent_core::tr!(
                "{}: could not read: {reason}",
                "{}: не удалось прочитать: {reason}",
                path.display()
            )),
            Self::Malformed { path, reason } => f.write_str(&agent_core::tr!(
                "{}: not readable as JSON: {reason}",
                "{}: не разбирается как JSON: {reason}",
                path.display()
            )),
            Self::Invalid(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigFile {
    base_url: String,
    device_token: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SyncConfig {
    base_url: String,
    device_token: String,
}

impl fmt::Debug for SyncConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncConfig")
            .field("base_url", &self.base_url)
            .field("device_token", &"<скрыт>")
            .finish()
    }
}

impl SyncConfig {
    pub fn new(
        base_url: impl Into<String>,
        device_token: impl Into<String>,
    ) -> Result<Self, ConfigError> {
        let base_url = base_url.into().trim().trim_end_matches('/').to_owned();
        let device_token = device_token.into().trim().to_owned();

        if device_token.is_empty() {
            return Err(ConfigError::Invalid(agent_core::tr!(
                "device token is empty",
                "device token пуст"
            )));
        }
        if !is_allowed_base_url(&base_url) {
            return Err(ConfigError::Invalid(agent_core::tr!(
                "{base_url}: https:// is required (http is allowed only for localhost \
                 in local checks — the server answers 400 to non-HTTPS)",
                "{base_url}: требуется https:// (http допустим только для localhost \
                 при локальной проверке — сервер отвечает 400 на не-HTTPS)"
            )));
        }

        Ok(Self {
            base_url,
            device_token,
        })
    }

    pub fn load() -> Result<Self, ConfigError> {
        let base_url = std::env::var(ENV_BASE_URL).unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());

        if let Ok(device_token) = std::env::var(ENV_DEVICE_TOKEN) {
            return Self::new(base_url, device_token);
        }

        let path = default_config_path();
        if path.exists() {
            return Self::from_file(&path);
        }

        let legacy = legacy_config_path();
        if legacy.exists() {
            return Self::from_file(&legacy);
        }

        Err(ConfigError::NotFound { looked_at: path })
    }

    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|error| ConfigError::Unreadable {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
        let parsed: ConfigFile =
            serde_json::from_str(&text).map_err(|error| ConfigError::Malformed {
                path: path.to_path_buf(),
                reason: error.to_string(),
            })?;

        let base_url = std::env::var(ENV_BASE_URL).unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());

        Self::new(base_url, parsed.device_token)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = default_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| ConfigError::Unreadable {
                path: parent.to_path_buf(),
                reason: error.to_string(),
            })?;
        }

        let file = ConfigFile {
            base_url: self.base_url.clone(),
            device_token: self.device_token.clone(),
        };

        let text = serde_json::to_string_pretty(&file).map_err(|error| ConfigError::Malformed {
            path: path.clone(),
            reason: error.to_string(),
        })?;

        std::fs::write(&path, text).map_err(|error| ConfigError::Unreadable {
            path,
            reason: error.to_string(),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn device_token(&self) -> &str {
        &self.device_token
    }

    pub fn rebuild_url(&self) -> String {
        format!("{}{REBUILD_PATH}", self.base_url)
    }

    pub fn observations_url(&self) -> String {
        format!("{}{OBSERVATIONS_PATH}", self.base_url)
    }

    pub fn state_url(&self) -> String {
        format!("{}{STATE_PATH}", self.base_url)
    }

    pub fn missions_url(&self) -> String {
        format!("{}{MISSIONS_PATH}", self.base_url)
    }
}

fn profile_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("elite-agent"))
}

pub fn default_config_path() -> PathBuf {
    profile_dir()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .unwrap_or_else(legacy_config_path)
}

pub fn legacy_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(CONFIG_FILE_NAME)))
        .unwrap_or_else(|| PathBuf::from(CONFIG_FILE_NAME))
}

fn is_allowed_base_url(base_url: &str) -> bool {
    if base_url.starts_with("https://") {
        return base_url.len() > "https://".len();
    }
    if let Some(rest) = base_url.strip_prefix("http://") {
        let host = rest.split(['/', ':']).next().unwrap_or("");
        return host == "localhost" || host == "127.0.0.1" || host == "[::1]";
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_file_is_read_back() {
        let path = std::env::temp_dir().join("elite-agent-config-roundtrip.json");
        let text = serde_json::to_string_pretty(&ConfigFile {
            base_url: "https://пример.ру".to_owned(),
            device_token: "секрет".to_owned(),
        })
        .expect("конфиг сериализуется");
        std::fs::write(&path, &text).expect("временный файл пишется");

        assert!(text.contains("\"baseUrl\""), "{text}");
        assert!(text.contains("\"deviceToken\""), "{text}");

        let config = SyncConfig::from_file(&path).expect("конфиг читается");
        assert_eq!(config.device_token(), "секрет");
        assert_eq!(config.base_url(), DEFAULT_BASE_URL);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    #[cfg(windows)]
    fn config_lives_in_the_user_profile() {
        let path = default_config_path();
        assert!(
            path.ends_with(std::path::Path::new("elite-agent").join(CONFIG_FILE_NAME)),
            "{}",
            path.display()
        );
    }

    #[test]
    fn token_is_hidden_in_debug_output() {
        let config = SyncConfig::new("https://пример.ру", "секрет").expect("значения годные");
        let shown = format!("{config:?}");
        assert!(!shown.contains("секрет"), "{shown}");
    }
}

#[cfg(test)]
mod baked_url_tests {
    use super::*;

    #[test]
    fn baked_base_url_is_usable() {
        assert!(
            DEFAULT_BASE_URL.starts_with("https://"),
            "{DEFAULT_BASE_URL}"
        );
        assert!(is_allowed_base_url(DEFAULT_BASE_URL), "{DEFAULT_BASE_URL}");
        assert!(
            !DEFAULT_BASE_URL.ends_with('/'),
            "хвостовой слэш даст // в путях"
        );
    }

    #[test]
    fn config_uses_baked_url_regardless_of_file() {
        let config = SyncConfig::new(DEFAULT_BASE_URL, "секрет").expect("конфиг собирается");

        assert_eq!(config.base_url(), DEFAULT_BASE_URL);
        assert!(config.state_url().starts_with(DEFAULT_BASE_URL));
        assert!(config.missions_url().starts_with(DEFAULT_BASE_URL));
    }
}
