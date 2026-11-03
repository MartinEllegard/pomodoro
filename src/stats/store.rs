//! Session history persisted as JSON Lines (one `SessionRecord` per line).

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use color_eyre::eyre::{OptionExt, Result, WrapErr};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::phase::Phase;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub phase: Phase,
    pub started_at: DateTime<Local>,
    pub ended_at: DateTime<Local>,
    /// Planned length of the phase, including `+`/`-` adjustments.
    pub planned_secs: u64,
    /// Time actually spent running, excluding pauses.
    pub focused_secs: u64,
}

/// `$XDG_DATA_HOME/pomodoro/sessions.jsonl` (or the platform equivalent).
pub fn log_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "pomodoro")
        .ok_or_eyre("could not determine a home directory for the session log")?;
    Ok(dirs.data_dir().join("sessions.jsonl"))
}

/// Read all records. A missing file is an empty history; lines that do not
/// parse (corrupt, truncated, from a future format) are skipped.
pub fn load(path: &Path) -> Result<Vec<SessionRecord>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).wrap_err_with(|| format!("could not open {}", path.display())),
    };
    let mut records = Vec::new();
    // Split on raw bytes so a line with invalid UTF-8 is skipped, not fatal.
    for line in BufReader::new(file).split(b'\n') {
        let line = line.wrap_err_with(|| format!("could not read {}", path.display()))?;
        if let Ok(record) = serde_json::from_slice(&line) {
            records.push(record);
        }
    }
    Ok(records)
}

/// Append one record, creating the file and its directory if needed.
pub fn append(path: &Path, record: &SessionRecord) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).wrap_err_with(|| format!("could not create {}", dir.display()))?;
    }
    let mut line = serde_json::to_string(record)?;
    line.push('\n');
    // One write call per record keeps lines whole even with concurrent writers.
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()))
        .wrap_err_with(|| format!("could not write {}", path.display()))
}
