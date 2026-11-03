use std::time::Duration;

use crate::phase::Phase;

/// Every n-th completed pomodoro is followed by a long break.
pub const LONG_BREAK_EVERY: u32 = 4;
/// Smallest phase length that can be configured, in minutes.
pub const MIN_MINUTES: u64 = 1;
/// Largest phase length that can be configured, in minutes.
pub const MAX_MINUTES: u64 = 999;

/// Phase lengths. Always whole minutes in `MIN_MINUTES..=MAX_MINUTES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub work: Duration,
    pub short_break: Duration,
    pub long_break: Duration,
}

impl Config {
    pub fn duration_for(&self, phase: Phase) -> Duration {
        match phase {
            Phase::Work => self.work,
            Phase::ShortBreak => self.short_break,
            Phase::LongBreak => self.long_break,
        }
    }

    /// The configured length of `phase` in whole minutes.
    pub fn minutes(&self, phase: Phase) -> u64 {
        self.duration_for(phase).as_secs() / 60
    }

    /// Mutable access to the configured duration of `phase`.
    pub fn duration_mut(&mut self, phase: Phase) -> &mut Duration {
        match phase {
            Phase::Work => &mut self.work,
            Phase::ShortBreak => &mut self.short_break,
            Phase::LongBreak => &mut self.long_break,
        }
    }

    /// Add `delta` minutes to `phase`, clamped to `MIN_MINUTES..=MAX_MINUTES`.
    pub fn adjust_minutes(&mut self, phase: Phase, delta: i64) {
        let minutes = self
            .minutes(phase)
            .saturating_add_signed(delta)
            .clamp(MIN_MINUTES, MAX_MINUTES);
        *self.duration_mut(phase) = Duration::from_secs(minutes * 60);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work: Duration::from_secs(25 * 60),
            short_break: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(15 * 60),
        }
    }
}
