use std::time::{Duration, Instant};

/// `-` never pushes the remaining time below this (unless it already is).
const MIN_REMAINING: Duration = Duration::from_secs(60);

/// A drift-free countdown.
///
/// Nothing is decremented per tick: the remaining time is always derived from
/// `Instant` bookkeeping, so a late or skipped tick never loses time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timer {
    duration: Duration,
    started_at: Instant,
    paused_total: Duration,
    paused_since: Option<Instant>,
}

impl Timer {
    pub fn start(duration: Duration, now: Instant) -> Self {
        Self {
            duration,
            started_at: now,
            paused_total: Duration::ZERO,
            paused_since: None,
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Running time, excluding pauses. Frozen while paused.
    pub fn elapsed(&self, now: Instant) -> Duration {
        let until = self.paused_since.unwrap_or(now);
        until
            .saturating_duration_since(self.started_at)
            .saturating_sub(self.paused_total)
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        self.duration.saturating_sub(self.elapsed(now))
    }

    pub fn is_finished(&self, now: Instant) -> bool {
        self.elapsed(now) >= self.duration
    }

    /// Fraction of the phase that has elapsed, always in `0.0..=1.0`.
    pub fn progress(&self, now: Instant) -> f64 {
        if self.duration.is_zero() {
            return 1.0;
        }
        (self.elapsed(now).as_secs_f64() / self.duration.as_secs_f64()).clamp(0.0, 1.0)
    }

    pub fn is_paused(&self) -> bool {
        self.paused_since.is_some()
    }

    pub fn pause(&mut self, now: Instant) {
        if self.paused_since.is_none() {
            self.paused_since = Some(now);
        }
    }

    pub fn resume(&mut self, now: Instant) {
        if let Some(since) = self.paused_since.take() {
            self.paused_total += now.saturating_duration_since(since);
        }
    }

    pub fn toggle(&mut self, now: Instant) {
        if self.is_paused() {
            self.resume(now);
        } else {
            self.pause(now);
        }
    }

    /// Move the remaining time by `delta_secs`. Shortening never takes the
    /// remaining time below one minute (or below its current value if that is
    /// already less), so `-` can never complete a phase by itself.
    pub fn adjust(&mut self, delta_secs: i64, now: Instant) {
        let remaining = self.remaining(now);
        let delta = Duration::from_secs(delta_secs.unsigned_abs());
        let target = if delta_secs >= 0 {
            remaining.saturating_add(delta)
        } else {
            remaining
                .saturating_sub(delta)
                .max(remaining.min(MIN_REMAINING))
        };
        self.duration = self.elapsed(now) + target;
    }
}
