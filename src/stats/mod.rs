pub mod store;

use std::time::Duration;

use chrono::{Datelike, Days, NaiveDate};
use store::SessionRecord;

use crate::phase::Phase;

/// In-memory session history, loaded at startup and appended to live.
#[derive(Debug, Default)]
pub struct Stats {
    records: Vec<SessionRecord>,
}

impl Stats {
    pub fn new(records: Vec<SessionRecord>) -> Self {
        Self { records }
    }

    pub fn push(&mut self, record: SessionRecord) {
        self.records.push(record);
    }

    /// Completed pomodoros (work phases) ever recorded.
    pub fn total(&self) -> usize {
        self.work_records().count()
    }

    fn work_records(&self) -> impl Iterator<Item = &SessionRecord> {
        self.records.iter().filter(|r| r.phase == Phase::Work)
    }

    /// Work records that ended on `day`.
    pub fn count_on(&self, day: NaiveDate) -> usize {
        self.work_records()
            .filter(|r| r.ended_at.date_naive() == day)
            .count()
    }

    /// Work records ending in the same ISO week (year + week) as `today`.
    pub fn count_this_week(&self, today: NaiveDate) -> usize {
        let week = today.iso_week();
        self.work_records()
            .filter(|r| r.ended_at.date_naive().iso_week() == week)
            .count()
    }

    /// Total focused time of work records that ended on `today`.
    pub fn focused_today(&self, today: NaiveDate) -> Duration {
        let secs = self
            .work_records()
            .filter(|r| r.ended_at.date_naive() == today)
            .map(|r| r.focused_secs)
            .sum();
        Duration::from_secs(secs)
    }

    /// Pomodoro counts for the last `days` days ending at `today`, ascending, zero-filled.
    pub fn per_day(&self, today: NaiveDate, days: usize) -> Vec<(NaiveDate, u64)> {
        (0..days)
            .rev()
            .filter_map(|n| today.checked_sub_days(Days::new(n as u64)))
            .map(|day| (day, self.count_on(day) as u64))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Days, Local, NaiveDate, TimeZone};

    use super::*;

    fn rec(phase: Phase, day: NaiveDate) -> SessionRecord {
        let t = Local
            .from_local_datetime(&day.and_hms_opt(12, 0, 0).unwrap())
            .unwrap();
        SessionRecord {
            phase,
            started_at: t,
            ended_at: t,
            planned_secs: 1500,
            focused_secs: 1500,
        }
    }

    #[test]
    fn aggregates() {
        // A Wednesday, so yesterday and last week are distinguishable.
        let today = NaiveDate::from_ymd_opt(2026, 11, 4).unwrap();
        let ago = |n| today - Days::new(n);
        let stats = Stats::new(vec![
            rec(Phase::Work, today),
            rec(Phase::Work, today),
            rec(Phase::Work, ago(1)),
            rec(Phase::Work, ago(8)),
            rec(Phase::Work, ago(7)),
            rec(Phase::ShortBreak, today),
            rec(Phase::LongBreak, today),
        ]);
        assert_eq!(stats.count_on(today), 2);
        assert_eq!(stats.count_this_week(today), 3);
        assert_eq!(stats.focused_today(today), Duration::from_secs(3000));
        let days = stats.per_day(today, 14);
        assert_eq!(days.len(), 14);
        assert_eq!(days[13], (today, 2));
        assert!(days.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(days[12].1, 1);
        assert_eq!(days[5].1, 1);
        assert_eq!(days[6].1, 1);
        assert_eq!(days.iter().map(|d| d.1).sum::<u64>(), 5);
    }
}
