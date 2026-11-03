use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::{Config, LONG_BREAK_EVERY};
use crate::phase::Phase;
use crate::stats::Stats;
use crate::stats::store::SessionRecord;
use crate::timer::Timer;

/// How far `+` / `-` move the running timer.
const ADJUST_SECS: i64 = 60;
/// Lateness of up to this much (tick granularity) is carried into the next
/// phase so ticks never accumulate. Larger gaps (e.g. the process was stopped
/// with SIGSTOP, or platforms where `Instant` advances during sleep) restart
/// the next phase at "now" instead of cascading through finished phases.
const MAX_CARRY_OVER: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Timer,
    Stats,
}

impl Screen {
    pub const ALL: [Self; 2] = [Self::Timer, Self::Stats];

    pub fn title(self) -> &'static str {
        match self {
            Self::Timer => "Timer",
            Self::Stats => "Stats",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popup {
    Settings,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Work,
    ShortBreak,
    LongBreak,
}

impl SettingsField {
    pub fn all() -> [Self; 3] {
        [Self::Work, Self::ShortBreak, Self::LongBreak]
    }

    pub fn next(self) -> Self {
        match self {
            Self::Work => Self::ShortBreak,
            Self::ShortBreak => Self::LongBreak,
            Self::LongBreak => Self::Work,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Work => Self::LongBreak,
            Self::ShortBreak => Self::Work,
            Self::LongBreak => Self::ShortBreak,
        }
    }

    pub fn label(self) -> &'static str {
        self.phase().label()
    }

    /// The phase whose length this field controls.
    pub fn phase(self) -> Phase {
        match self {
            Self::Work => Phase::Work,
            Self::ShortBreak => Phase::ShortBreak,
            Self::LongBreak => Phase::LongBreak,
        }
    }
}

/// An in-progress edit of the configuration in the settings popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsEdit {
    pub draft: Config,
    pub selected: SettingsField,
}

/// Side effects requested by the state machine; executed by the event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Notify { summary: String, body: String },
    AppendRecord(SessionRecord),
    Quit,
}

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub phase: Phase,
    pub timer: Timer,
    pub phase_started_wall: DateTime<Local>,
    /// Completed (not skipped) work phases this session.
    pub work_completed: u32,
    pub screen: Screen,
    pub popup: Option<Popup>,
    /// `Some` exactly while `popup == Some(Popup::Settings)`.
    pub settings: Option<SettingsEdit>,
    pub stats: Stats,
    /// The instant the UI renders against; set by the event loop.
    pub now: Instant,
    pub last_error: Option<String>,
}

impl App {
    pub fn new(config: Config, stats: Stats, now: Instant) -> Self {
        Self {
            config,
            phase: Phase::Work,
            timer: Timer::start(config.duration_for(Phase::Work), now),
            phase_started_wall: Local::now(),
            work_completed: 0,
            screen: Screen::default(),
            popup: None,
            settings: None,
            stats,
            now,
            last_error: None,
        }
    }

    /// Advance the clock; completes the phase when its time is up.
    pub fn tick(&mut self, now: Instant) -> Vec<Effect> {
        self.now = now;
        if self.timer.is_finished(now) {
            self.complete_phase()
        } else {
            Vec::new()
        }
    }

    /// Position of the current (or just finished) pomodoro in the long-break
    /// cycle, `1..=LONG_BREAK_EVERY`.
    pub fn cycle_position(&self) -> u32 {
        let done_before = match self.phase {
            Phase::Work => self.work_completed,
            Phase::ShortBreak | Phase::LongBreak => self.work_completed.saturating_sub(1),
        };
        done_before % LONG_BREAK_EVERY + 1
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        // Ctrl-C always quits, even with a popup open.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return vec![Effect::Quit];
        }

        match self.popup {
            Some(Popup::Help) => {
                self.help_key(key);
                return Vec::new();
            }
            Some(Popup::Settings) => {
                self.settings_key(key);
                return Vec::new();
            }
            None => {}
        }

        match key.code {
            KeyCode::Char('q') => return vec![Effect::Quit],
            KeyCode::Char('?') => self.popup = Some(Popup::Help),
            KeyCode::Char(',') => self.open_settings(),
            KeyCode::Tab => self.screen = self.screen.next(),
            KeyCode::BackTab => self.screen = self.screen.prev(),
            KeyCode::Char('1') => self.screen = Screen::Timer,
            KeyCode::Char('2') => self.screen = Screen::Stats,
            _ if self.screen == Screen::Timer => return self.timer_key(key),
            _ => {}
        }
        Vec::new()
    }

    fn timer_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        let now = self.now;
        match key.code {
            KeyCode::Char(' ') => self.timer.toggle(now),
            KeyCode::Char('s') => return self.skip(),
            KeyCode::Char('r') => self.reset(),
            KeyCode::Char('+' | '=') => self.timer.adjust(ADJUST_SECS, now),
            KeyCode::Char('-') => self.timer.adjust(-ADJUST_SECS, now),
            _ => {}
        }
        Vec::new()
    }

    fn help_key(&mut self, key: KeyEvent) {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?' | 'q')
        ) {
            self.popup = None;
        }
    }

    pub fn open_settings(&mut self) {
        self.settings = Some(SettingsEdit {
            draft: self.config,
            selected: SettingsField::Work,
        });
        self.popup = Some(Popup::Settings);
    }

    pub fn close_settings(&mut self) {
        self.settings = None;
        self.popup = None;
    }

    fn select_field(&mut self, f: fn(SettingsField) -> SettingsField) {
        if let Some(edit) = &mut self.settings {
            edit.selected = f(edit.selected);
        }
    }

    fn adjust_selected(&mut self, delta: i64) {
        if let Some(edit) = &mut self.settings {
            edit.draft.adjust_minutes(edit.selected.phase(), delta);
        }
    }

    /// Keys while the settings popup is open. Committing only changes
    /// `config`, which is read when the *next* phase starts; the running
    /// timer is deliberately left alone.
    pub fn settings_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select_field(SettingsField::prev),
            KeyCode::Down | KeyCode::Char('j') => self.select_field(SettingsField::next),
            KeyCode::Char('+' | '=') | KeyCode::Right | KeyCode::Char('l') => {
                self.adjust_selected(1);
            }
            KeyCode::Char('-') | KeyCode::Left | KeyCode::Char('h') => self.adjust_selected(-1),
            KeyCode::Esc => self.close_settings(),
            KeyCode::Enter => {
                if let Some(edit) = self.settings {
                    self.config = edit.draft;
                }
                self.close_settings();
            }
            _ => {}
        }
    }

    /// The running phase ran to completion: record it (work only), notify,
    /// and start the next phase.
    fn complete_phase(&mut self) -> Vec<Effect> {
        let finished = self.phase;
        let planned = self.timer.duration();
        let elapsed = self.timer.elapsed(self.now);
        let mut effects = Vec::new();
        if finished == Phase::Work {
            self.work_completed += 1;
            effects.push(Effect::AppendRecord(SessionRecord {
                phase: finished,
                started_at: self.phase_started_wall,
                ended_at: Local::now(),
                planned_secs: planned.as_secs(),
                focused_secs: elapsed.min(planned).as_secs(),
            }));
        }
        let next = finished.next(self.work_completed);
        effects.push(self.transition_notice(format!("{} finished.", finished.label()), next));

        let late_by = elapsed.saturating_sub(planned);
        let next_start = if late_by <= MAX_CARRY_OVER {
            self.now.checked_sub(late_by).unwrap_or(self.now)
        } else {
            self.now
        };
        self.start_phase(next, next_start);
        effects
    }

    /// Abandon the current phase without recording or counting it.
    ///
    /// Skipping work goes to a short break: the skipped pomodoro does not
    /// count, so it cannot earn the long break.
    pub fn skip(&mut self) -> Vec<Effect> {
        let skipped = self.phase;
        let next = match skipped {
            Phase::Work => Phase::ShortBreak,
            Phase::ShortBreak | Phase::LongBreak => skipped.next(self.work_completed),
        };
        let notice = self.transition_notice(format!("{} skipped.", skipped.label()), next);
        self.start_phase(next, self.now);
        vec![notice]
    }

    /// Restart the current phase from its full configured length.
    pub fn reset(&mut self) {
        self.start_phase(self.phase, self.now);
    }

    fn start_phase(&mut self, phase: Phase, at: Instant) {
        self.phase = phase;
        self.timer = Timer::start(self.config.duration_for(phase), at);
        self.phase_started_wall = Local::now();
    }

    fn transition_notice(&self, what: String, next: Phase) -> Effect {
        let minutes = self.config.duration_for(next).as_secs() / 60;
        Effect::Notify {
            summary: format!("Pomodoro: {}", next.label()),
            body: format!("{what} {} for {minutes} min.", next.label()),
        }
    }
}
