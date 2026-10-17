use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
pub(crate) struct NavyRank {
    pub(crate) rank: Option<(u8, DateTime<Utc>)>,
    pub(crate) progress: Option<(f64, DateTime<Utc>)>,
    pub(crate) stale_rank_at: Option<DateTime<Utc>>,
}

impl NavyRank {
    pub(crate) fn is_newer_rank(&self, at: DateTime<Utc>) -> bool {
        self.rank.is_none_or(|(_, seen)| at >= seen)
    }

    pub(crate) fn is_newer_progress(&self, at: DateTime<Utc>) -> bool {
        self.progress.is_none_or(|(_, seen)| at >= seen)
    }

    pub(crate) fn observe_rank(&mut self, value: u8, at: DateTime<Utc>) {
        if !self.is_newer_rank(at) {
            return;
        }

        if self.rank.is_some_and(|(seen, _)| value < seen) {
            self.stale_rank_at = Some(at);
            return;
        }

        let changed = self.rank.is_some_and(|(seen, _)| seen != value);
        self.rank = Some((value, at));

        if changed {
            self.reset_progress(at);
        }
    }

    pub(crate) fn observe_promotion(&mut self, value: u8, at: DateTime<Utc>) {
        if self.is_newer_rank(at) && self.rank.is_none_or(|(seen, _)| value >= seen) {
            self.rank = Some((value, at));
        }
        self.reset_progress(at);
    }

    pub(crate) fn observe_progress(&mut self, value: f64, at: DateTime<Utc>) {
        if self.stale_rank_at == Some(at) {
            return;
        }

        if self.is_newer_progress(at) {
            self.progress = Some((value, at));
        }
    }

    pub(crate) fn restart(&mut self, at: DateTime<Utc>) {
        *self = Self {
            rank: Some((0, at)),
            progress: Some((0.0, at)),
            stale_rank_at: None,
        };
    }

    pub(crate) fn reset_progress(&mut self, at: DateTime<Utc>) {
        if self.is_newer_progress(at) {
            self.progress = Some((0.0, at));
        }
    }
}
