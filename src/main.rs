mod app;
mod cli;
mod config;
mod event_loop;
mod notify;
mod phase;
mod stats;
mod timer;
mod ui;

use std::time::Instant;

use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::WrapErr;

use crate::app::App;
use crate::cli::Cli;
use crate::stats::{Stats, store};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Before ratatui::init(), so ratatui's panic hook restores the terminal
    // first and then chains to color-eyre's report.
    color_eyre::install()?;
    let config = Cli::parse().into();

    // Fallible startup work happens on the normal terminal so errors are readable.
    let log_path = store::log_path()?;
    let stats = Stats::new(store::load(&log_path)?);
    let app = App::new(config, stats, Instant::now());

    // try_init instead of init: no tty (piped, cron, CI) is an error, not a panic.
    let mut terminal = match ratatui::try_init() {
        Ok(terminal) => terminal,
        Err(e) => {
            // Undo whatever part of the setup succeeded; nothing to report if it fails too.
            let _ = ratatui::try_restore();
            return Err(e).wrap_err("could not initialize the terminal");
        }
    };
    let result = event_loop::run(&mut terminal, app, log_path).await;
    ratatui::restore();
    result
}
