use std::time::Duration;

use agent_core::contract::ActiveMissionsPayload;
use agent_core::journal::Observation;
use agent_core::{CommanderState, ContractViolation};
use serde::{Deserialize, Serialize};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub const MIN_SEND_INTERVAL: Duration = Duration::from_secs(5);

pub const TOKEN_HINT: &str = "Ключ агента больше не действует. Откройте кабинет, раздел \
     «Ключ агента»: «Перевыпустить», затем «Скачать elite-agent.json». Положите файл рядом с \
     elite-agent.exe или вставьте ключ в поле «Ключ агента» в окне агента.";

#[derive(Debug)]
pub enum SendError {
    Contract(Vec<ContractViolation>),
    Transport(String),
    Retryable {
        status: u16,
        message: String,
        retry_after: Option<Duration>,
    },
    Rejected {
        status: u16,
        message: String,
    },
}

impl SendError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Retryable { .. })
    }

    pub fn hint(&self) -> Option<&'static str> {
        match self {
            Self::Rejected {
                status: 401 | 403, ..
            } => Some(TOKEN_HINT),
            _ => None,
        }
    }

    pub fn is_token_rejected(&self) -> bool {
        matches!(
            self,
            Self::Rejected {
                status: 401 | 403,
                ..
            }
        )
    }
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(violations) => {
                let listed: Vec<String> = violations.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "payload не соответствует контракту: {}",
                    listed.join("; ")
                )
            }
            Self::Transport(reason) => write!(f, "сеть недоступна: {reason}"),
            Self::Retryable {
                status,
                message,
                retry_after,
            } => {
                write!(f, "сервер ответил {status}: {message}")?;
                if let Some(delay) = retry_after {
                    write!(f, " (повтор через {} с)", delay.as_secs())?;
                }
                Ok(())
            }
            Self::Rejected { status, message } => {
                write!(f, "запрос отклонён, {status}: {message}")
            }
        }
    }
}

impl std::error::Error for SendError {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendAccepted {
    pub updated_at: String,
    #[serde(default)]
    pub agent_update: Option<AgentUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUpdate {
    #[serde(default)]
    pub current_version: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    errors: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuildAccepted {
    #[serde(default)]
    pub rebuilt: Vec<RebuiltRegion>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuiltRegion {
    pub region: String,
    pub revision: u64,
    pub systems: u32,
    pub stations: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationsAccepted {
    pub accepted: u32,
    pub applied: u32,
    pub unchanged: u32,
    #[serde(default)]
    pub skipped: Vec<SkippedObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedObservation {
    pub index: u32,
    pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ObservationsBatch<'a> {
    schema_version: u8,
    observations: &'a [Observation],
}

pub struct StateClient {
    url: String,
    rebuild_url: String,
    observations_url: String,
    missions_url: String,
    device_token: String,
    agent: ureq::Agent,
}

impl StateClient {
    pub fn new(config: &crate::SyncConfig) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .http_status_as_error(false)
            .build()
            .into();

        Self {
            url: config.state_url(),
            rebuild_url: config.rebuild_url(),
            observations_url: config.observations_url(),
            missions_url: config.missions_url(),
            device_token: config.device_token().to_owned(),
            agent,
        }
    }

    pub fn post_observations(
        &self,
        observations: &[Observation],
    ) -> Result<ObservationsAccepted, SendError> {
        let batch = ObservationsBatch {
            schema_version: 1,
            observations,
        };

        let response = self
            .agent
            .post(&self.observations_url)
            .header("Authorization", &format!("Bearer {}", self.device_token))
            .header("Content-Type", "application/json")
            .send_json(&batch)
            .map_err(|error| SendError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .into_body()
            .read_to_string()
            .unwrap_or_else(|error| format!("тело ответа не прочиталось: {error}"));

        classify_status(status, &body)?;

        serde_json::from_str::<ObservationsAccepted>(&body).map_err(|error| SendError::Rejected {
            status,
            message: format!("ответ 200, но тело не разобралось: {error}"),
        })
    }

    pub fn put_missions(&self, payload: &ActiveMissionsPayload) -> Result<(), SendError> {
        let response = self
            .agent
            .put(&self.missions_url)
            .header("Authorization", &format!("Bearer {}", self.device_token))
            .header("Content-Type", "application/json")
            .send_json(payload)
            .map_err(|error| SendError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .into_body()
            .read_to_string()
            .unwrap_or_else(|error| format!("тело ответа не прочиталось: {error}"));

        classify_status(status, &body)
    }

    pub fn request_rebuild(&self) -> Result<RebuildAccepted, SendError> {
        let response = self
            .agent
            .post(&self.rebuild_url)
            .header("Authorization", &format!("Bearer {}", self.device_token))
            .header("Content-Length", "0")
            .send_empty()
            .map_err(|error| SendError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .into_body()
            .read_to_string()
            .unwrap_or_else(|error| format!("тело ответа не прочиталось: {error}"));

        classify_status(status, &body)?;

        serde_json::from_str::<RebuildAccepted>(&body).map_err(|error| SendError::Rejected {
            status,
            message: format!("ответ 200, но тело не разобралось: {error}"),
        })
    }

    pub fn put_state(&self, state: &CommanderState) -> Result<SendAccepted, SendError> {
        let violations = state.validate();
        if !violations.is_empty() {
            return Err(SendError::Contract(violations));
        }

        let response = self
            .agent
            .put(&self.url)
            .header("Authorization", &format!("Bearer {}", self.device_token))
            .header("Content-Type", "application/json")
            .send_json(state)
            .map_err(|error| SendError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .into_body()
            .read_to_string()
            .unwrap_or_else(|error| format!("тело ответа не прочиталось: {error}"));

        classify_status(status, &body)?;

        serde_json::from_str::<SendAccepted>(&body).map_err(|error| SendError::Rejected {
            status,
            message: format!("ответ 200, но тело не разобралось: {error}"),
        })
    }
}

fn classify_status(status: u16, body: &str) -> Result<(), SendError> {
    if status == 200 {
        return Ok(());
    }

    let parsed = serde_json::from_str::<ErrorBody>(body).ok();
    let message = describe(parsed, body);

    Err(match status {
        429 => SendError::Retryable {
            status,
            message,
            retry_after: retry_after_from(body),
        },
        500..=599 => SendError::Retryable {
            status,
            message,
            retry_after: None,
        },
        _ => SendError::Rejected { status, message },
    })
}

fn describe(parsed: Option<ErrorBody>, body: &str) -> String {
    let Some(parsed) = parsed else {
        return body.chars().take(200).collect();
    };
    let message = parsed.message.unwrap_or_else(|| "без описания".to_owned());
    match parsed.errors {
        Some(errors) => format!("{message} {errors}"),
        None => message,
    }
}

fn retry_after_from(body: &str) -> Option<Duration> {
    let parsed: serde_json::Value = serde_json::from_str(body).ok()?;
    let seconds = parsed.get("errors")?.get("retryAfterSeconds")?;
    let seconds = match seconds {
        serde_json::Value::String(text) => text.parse::<u64>().ok()?,
        serde_json::Value::Number(number) => number.as_u64()?,
        _ => return None,
    };
    Some(Duration::from_secs(seconds))
}
