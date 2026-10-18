use std::time::Duration;

use agent_core::CommanderState;

use crate::client::{SendAccepted, StateClient, MIN_SEND_INTERVAL};
use crate::send_error::SendError;

const MAX_BACKOFF: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: MIN_SEND_INTERVAL,
        }
    }
}

impl RetryPolicy {
    pub fn delay_before(&self, attempt: u32, suggested: Option<Duration>) -> Duration {
        let computed = self
            .base_delay
            .saturating_mul(2_u32.saturating_pow(attempt.saturating_sub(2)));
        suggested.unwrap_or(computed).min(MAX_BACKOFF)
    }
}

pub fn send_with_retry(
    client: &StateClient,
    state: &CommanderState,
    policy: &RetryPolicy,
    on_retry: &mut dyn FnMut(&SendError, Duration),
) -> Result<SendAccepted, SendError> {
    send_with_retry_waiting(client, state, policy, on_retry, &mut |delay| {
        std::thread::sleep(delay)
    })
}

pub fn send_with_retry_waiting(
    client: &StateClient,
    state: &CommanderState,
    policy: &RetryPolicy,
    on_retry: &mut dyn FnMut(&SendError, Duration),
    wait: &mut dyn FnMut(Duration),
) -> Result<SendAccepted, SendError> {
    let mut attempt = 1;
    loop {
        match client.put_state(state) {
            Ok(accepted) => return Ok(accepted),
            Err(error) => {
                if !error.is_retryable() || attempt >= policy.max_attempts {
                    return Err(error);
                }
                let suggested = match &error {
                    SendError::Retryable { retry_after, .. } => *retry_after,
                    _ => None,
                };
                attempt += 1;
                let delay = policy.delay_before(attempt, suggested);
                on_retry(&error, delay);
                wait(delay);
            }
        }
    }
}
