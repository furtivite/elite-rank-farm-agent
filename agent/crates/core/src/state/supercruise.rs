use chrono::{DateTime, Utc};

use crate::contract::{SupercruiseLeg, MAX_LEG_SECONDS};

#[derive(Debug, Clone)]
pub(crate) struct LegInProgress {
    pub(crate) arrived_at: DateTime<Utc>,
    pub(crate) exited_at: Option<DateTime<Utc>>,
}

pub(crate) fn finish_leg(
    leg: Option<LegInProgress>,
    distance_ls: Option<f64>,
    docked_at: DateTime<Utc>,
) -> Option<SupercruiseLeg> {
    let leg = leg?;
    let (exited_at, distance_ls) = (leg.exited_at?, distance_ls?);
    if distance_ls <= 0.0 || !distance_ls.is_finite() {
        return None;
    }

    let seconds = (exited_at - leg.arrived_at).num_seconds();
    let docking = (docked_at - exited_at).num_seconds();
    if seconds <= 0 || seconds > MAX_LEG_SECONDS {
        return None;
    }

    Some(SupercruiseLeg {
        at: docked_at,
        distance_ls,
        seconds: seconds as u32,
        docking_seconds: (docking > 0 && docking <= MAX_LEG_SECONDS).then_some(docking as u32),
    })
}
