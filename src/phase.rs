use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::config::LONG_BREAK_EVERY;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Work => "Work",
            Self::ShortBreak => "Short break",
            Self::LongBreak => "Long break",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Work => Color::Green,
            Self::ShortBreak => Color::Yellow,
            Self::LongBreak => Color::Cyan,
        }
    }

    /// The phase that follows `self`, given the number of completed work phases
    /// *including* the one that just ended.
    pub fn next(self, work_done: u32) -> Self {
        match self {
            Self::Work if work_done > 0 && work_done.is_multiple_of(LONG_BREAK_EVERY) => {
                Self::LongBreak
            }
            Self::Work => Self::ShortBreak,
            Self::ShortBreak | Self::LongBreak => Self::Work,
        }
    }
}
