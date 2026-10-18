use std::time::Duration;

use agent_core::ContractViolation;

pub fn token_hint() -> String {
    agent_core::tr!(
        "The agent key no longer works. Open your account on the site, the agent section: \
         “Reissue”, then “Download elite-agent.json”. Put the file next to elite-agent.exe or \
         paste the key into the “Agent key” field in the agent window.",
        "Ключ агента больше не действует. Откройте кабинет, раздел «Ключ агента»: \
         «Перевыпустить», затем «Скачать elite-agent.json». Положите файл рядом с \
         elite-agent.exe или вставьте ключ в поле «Ключ агента» в окне агента."
    )
}

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

    pub fn hint(&self) -> Option<String> {
        match self {
            Self::Rejected {
                status: 401 | 403, ..
            } => Some(token_hint()),
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
                f.write_str(&agent_core::tr!(
                    "payload does not match the contract: {}",
                    "payload не соответствует контракту: {}",
                    listed.join("; ")
                ))
            }
            Self::Transport(reason) => f.write_str(&agent_core::tr!(
                "network unavailable: {reason}",
                "сеть недоступна: {reason}"
            )),
            Self::Retryable {
                status,
                message,
                retry_after,
            } => {
                f.write_str(&agent_core::tr!(
                    "server answered {status}: {message}",
                    "сервер ответил {status}: {message}"
                ))?;
                if let Some(delay) = retry_after {
                    f.write_str(&agent_core::tr!(
                        " (retry in {} s)",
                        " (повтор через {} с)",
                        delay.as_secs()
                    ))?;
                }
                Ok(())
            }
            Self::Rejected { status, message } => f.write_str(&agent_core::tr!(
                "request rejected, {status}: {message}",
                "запрос отклонён, {status}: {message}"
            )),
        }
    }
}

impl std::error::Error for SendError {}
