# pomodoro

A Pomodoro timer for the terminal, built with [ratatui](https://ratatui.rs).
It cycles through work sessions and short breaks, with a long break after
every 4th completed pomodoro. Desktop notifications announce each phase
change, and every completed work session is appended to a log that the
stats screen summarises.

## Requirements

- Rust 1.89 or newer (see `rust-version` in `Cargo.toml`)
- A terminal with Unicode support

## Install and run

```sh
cargo install --path .
pomodoro [OPTIONS]

# or, without installing
cargo run -- [OPTIONS]
```

Durations are whole minutes in the range 1 to 999.

```
A Pomodoro Technique timer for the terminal

Usage: pomodoro [OPTIONS]

Options:
  -w, --work <MINUTES>        Length of a work session in minutes [default: 25]
  -b, --break <MINUTES>       Length of a short break in minutes [default: 5]
  -l, --long-break <MINUTES>  Length of a long break (after every 4th pomodoro) in minutes [default: 15]
  -h, --help                  Print help
  -V, --version               Print version
```

## Key bindings

| Context  | Key                         | Action                                  |
| -------- | --------------------------- | --------------------------------------- |
| Global   | `q` / `ctrl-c`              | Quit                                    |
| Global   | `?`                         | Open help                               |
| Global   | `,`                         | Open settings                           |
| Global   | `tab` / `shift-tab` / `1` / `2` | Switch between Timer and Stats      |
| Timer    | `space`                     | Pause / resume                          |
| Timer    | `s`                         | Skip the current phase                  |
| Timer    | `r`                         | Restart the current phase               |
| Timer    | `+` / `-`                   | Add / remove one minute on the running timer |
| Settings | `up` / `down` or `j` / `k`  | Select a field                          |
| Settings | `+` / `-`, `left` / `right` or `h` / `l` | Change the value           |
| Settings | `enter`                     | Save and close                          |
| Settings | `esc`                       | Cancel and close                        |
| Help     | `esc` / `?` / `q` / `enter` | Close                                   |

- Saved settings apply from the next phase; `+` / `-` change the phase that
  is already running.
- `-` never shortens the remaining time below one minute.
- Skipping a work phase does not count as a completed pomodoro, so it cannot
  earn a long break.

## Session log

Completed work phases are appended to
`$XDG_DATA_HOME/pomodoro/sessions.jsonl`, which is
`~/.local/share/pomodoro/sessions.jsonl` on Linux. Other platforms use their
equivalent data directory, as resolved by the `directories` crate. Each line
is one JSON object:

```json
{"phase":"work","started_at":"2026-11-03T09:00:00+01:00","ended_at":"2026-11-03T09:25:00+01:00","planned_secs":1500,"focused_secs":1500}
```

`planned_secs` is the phase length including any `+` / `-` adjustments.
`focused_secs` is the time the timer actually ran, so pauses are excluded.
Skipped phases and breaks are not logged. Lines that fail to parse are
ignored when the log is read.

## Notifications

Phase changes are sent as desktop notifications with
[notify-rust](https://crates.io/crates/notify-rust) over D-Bus. A missing
notification daemon or D-Bus session is not an error: the notification is
dropped and the timer carries on.

## Architecture

| Module       | Role                                                              |
| ------------ | ----------------------------------------------------------------- |
| `cli`        | clap argument parsing and range validation                        |
| `config`     | Phase durations and the long-break interval                       |
| `phase`      | Work / short break / long break and which phase follows which     |
| `timer`      | Drift-free countdown with pause and +/- adjustment                |
| `app`        | Application state, key handling and phase transitions             |
| `event_loop` | Async loop that drives `App` and executes its effects             |
| `notify`     | Fire-and-forget desktop notifications                             |
| `stats`      | Session records, JSONL storage and aggregation                    |
| `ui`         | Rendering of the timer, stats, settings and help views            |

The program runs on a tokio current-thread runtime. The event loop uses
`tokio::select!` over a 250 ms tick and crossterm's `EventStream`;
`App::handle_key` and `App::tick` return `Effect`s (append a record, send a
notification, quit) that the loop executes, and `ui::view` is a pure function
of `&App`. The timer derives the remaining time from `Instant`s instead of
decrementing a counter, so late ticks never make it drift.
