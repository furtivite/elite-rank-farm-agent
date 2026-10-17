use std::time::Duration;

use agent_core::CommanderState;

use crate::client::MIN_SEND_INTERVAL;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendDecision {
    Send,
    Unchanged,
    TooSoon(Duration),
    NotReady,
}

const FUEL_STEP: f64 = 0.05;

const FUEL_STEP_LOW: f64 = 0.02;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShipContext {
    pub parked: bool,
    pub fsd_charging: bool,
    pub low_fuel: bool,
}

fn worth_sending(current: &CommanderState, last: &CommanderState, ship: ShipContext) -> bool {
    let quiet = |state: &CommanderState| {
        let mut copy = state.clone();
        copy.observed_at = current.observed_at;
        if let Some(loadout) = copy.loadout.as_mut() {
            loadout.fuel_level_t = None;
            loadout.cargo_t = None;
        }
        copy
    };

    if quiet(current) != quiet(last) {
        return true;
    }

    let (Some(now), Some(before)) = (current.loadout.as_ref(), last.loadout.as_ref()) else {
        return false;
    };

    if ship.fsd_charging && now.fuel_level_t != before.fuel_level_t {
        return true;
    }

    if now.cargo_t != before.cargo_t && !ship.parked {
        return true;
    }

    match (now.fuel_level_t, before.fuel_level_t) {
        (Some(now_fuel), Some(before_fuel)) => {
            let step = if ship.low_fuel {
                FUEL_STEP_LOW
            } else {
                FUEL_STEP
            };
            (now_fuel - before_fuel).abs() >= now.fuel_capacity_t * step
        }
        (Some(_), None) => true,
        _ => false,
    }
}

pub fn decide(
    current: Option<&CommanderState>,
    last_sent: Option<&CommanderState>,
    since_last_send: Option<Duration>,
    min_interval: Duration,
) -> SendDecision {
    decide_for_ship(
        current,
        last_sent,
        since_last_send,
        min_interval,
        ShipContext::default(),
    )
}

pub fn decide_for_ship(
    current: Option<&CommanderState>,
    last_sent: Option<&CommanderState>,
    since_last_send: Option<Duration>,
    min_interval: Duration,
    ship: ShipContext,
) -> SendDecision {
    let Some(current) = current else {
        return SendDecision::NotReady;
    };

    match last_sent {
        Some(last) if !worth_sending(current, last, ship) => return SendDecision::Unchanged,
        _ => {}
    }

    match since_last_send {
        Some(elapsed) if elapsed < min_interval => SendDecision::TooSoon(min_interval - elapsed),
        _ => SendDecision::Send,
    }
}

pub fn decide_default(
    current: Option<&CommanderState>,
    last_sent: Option<&CommanderState>,
    since_last_send: Option<Duration>,
) -> SendDecision {
    decide(current, last_sent, since_last_send, MIN_SEND_INTERVAL)
}

const MAX_LEARNED_INTERVAL: Duration = Duration::from_secs(60);

const RETRY_AFTER_MARGIN: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendPace {
    base: Duration,
    interval: Duration,
}

impl SendPace {
    pub fn new(base: Duration) -> Self {
        Self {
            base,
            interval: base,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn learn(&mut self, suggested: Option<Duration>) {
        let wanted = match suggested {
            Some(delay) => delay.saturating_add(RETRY_AFTER_MARGIN),
            None => self.interval.saturating_mul(2),
        };

        self.interval = wanted.clamp(self.base, MAX_LEARNED_INTERVAL);
    }
}

impl Default for SendPace {
    fn default() -> Self {
        Self::new(MIN_SEND_INTERVAL)
    }
}
