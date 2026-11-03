use std::time::Duration;

use clap::Parser;

use crate::config::{Config, MAX_MINUTES, MIN_MINUTES};
use crate::phase::Phase;

/// Default length of `phase` in minutes, taken from `Config::default()`.
fn default_minutes(phase: Phase) -> u64 {
    Config::default().minutes(phase)
}

/// A Pomodoro Technique timer for the terminal.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Length of a work session in minutes
    #[arg(
        short,
        long,
        value_name = "MINUTES",
        default_value_t = default_minutes(Phase::Work),
        value_parser = clap::value_parser!(u64).range(MIN_MINUTES..=MAX_MINUTES),
    )]
    pub work: u64,

    /// Length of a short break in minutes
    #[arg(
        short = 'b',
        long = "break",
        value_name = "MINUTES",
        default_value_t = default_minutes(Phase::ShortBreak),
        value_parser = clap::value_parser!(u64).range(MIN_MINUTES..=MAX_MINUTES),
    )]
    pub break_minutes: u64,

    /// Length of a long break (after every 4th pomodoro) in minutes
    #[arg(
        short,
        long,
        value_name = "MINUTES",
        default_value_t = default_minutes(Phase::LongBreak),
        value_parser = clap::value_parser!(u64).range(MIN_MINUTES..=MAX_MINUTES),
    )]
    pub long_break: u64,
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        let minutes = |m: u64| Duration::from_secs(m * 60);
        Self {
            work: minutes(cli.work),
            short_break: minutes(cli.break_minutes),
            long_break: minutes(cli.long_break),
        }
    }
}
