use std::path::PathBuf;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::time::MissedTickBehavior;

use crate::app::{App, Effect};
use crate::stats::store;
use crate::{notify, ui};

const TICK: Duration = Duration::from_millis(250);

/// Draw, wait for a tick or a terminal event, apply the resulting effects.
pub async fn run(terminal: &mut DefaultTerminal, mut app: App, log_path: PathBuf) -> Result<()> {
    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(TICK);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        terminal.draw(|frame| ui::view(&app, frame))?;

        let effects = tokio::select! {
            _ = tick.tick() => app.tick(Instant::now()),
            event = events.next() => match event {
                Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                    app.now = Instant::now();
                    app.handle_key(key)
                }
                // Resize, focus, key release/repeat, ...: just redraw.
                Some(Ok(_)) => Vec::new(),
                Some(Err(e)) => return Err(e.into()),
                None => vec![Effect::Quit],
            },
        };

        for effect in effects {
            match effect {
                Effect::Quit => return Ok(()),
                Effect::Notify { summary, body } => notify::spawn(summary, body),
                Effect::AppendRecord(record) => {
                    match store::append(&log_path, &record) {
                        Ok(()) => app.last_error = None,
                        Err(e) => app.last_error = Some(format!("{e:#}")),
                    }
                    app.stats.push(record);
                }
            }
        }
    }
}
