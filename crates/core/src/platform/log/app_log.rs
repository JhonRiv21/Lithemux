//! The app's own log: problems worth knowing about later, one per line,
//! with time and level. It stays on this Mac; nothing is sent anywhere.
//! It never says what was said in a conversation: only which session and
//! what went wrong.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

use crate::platform::log::timestamp::utc_timestamp;

pub const MAX_BYTES: u64 = 1024 * 1024;
/// The current file and the ones moved aside before it.
pub const FILES_KEPT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
        }
    }
}

pub struct AppLog {
    path: PathBuf,
    max_bytes: u64,
    files_kept: usize,
    /// Threads write whole lines, one at a time.
    writing: Mutex<()>,
}

impl AppLog {
    /// The log of `path` for the whole process, with the usual limits.
    /// Whoever asks for the same file gets the same writer: two would
    /// rotate it under each other.
    pub fn shared(path: PathBuf) -> Arc<AppLog> {
        static OPEN: Mutex<Vec<(PathBuf, Weak<AppLog>)>> = Mutex::new(Vec::new());
        let Ok(mut open) = OPEN.lock() else {
            return Arc::new(AppLog::new(path, MAX_BYTES, FILES_KEPT));
        };
        open.retain(|(_, log)| log.strong_count() > 0);
        if let Some(log) = open
            .iter()
            .find(|(at, _)| *at == path)
            .and_then(|(_, log)| log.upgrade())
        {
            return log;
        }
        let log = Arc::new(AppLog::new(path.clone(), MAX_BYTES, FILES_KEPT));
        open.push((path, Arc::downgrade(&log)));
        log
    }

    pub fn new(path: PathBuf, max_bytes: u64, files_kept: usize) -> Self {
        AppLog {
            path,
            max_bytes,
            files_kept: files_kept.max(1),
            writing: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends one line. A log that cannot be written is left alone: it
    /// must never stop what was being logged.
    pub fn write(&self, level: Level, message: &str, now: i64) {
        let flat = message.split_whitespace().collect::<Vec<_>>().join(" ");
        let line = format!("{} {} {flat}\n", utc_timestamp(now), level.as_str());
        let _guard = self.writing.lock();
        let _ = self.append(&line);
    }

    fn append(&self, line: &str) -> std::io::Result<()> {
        if let Some(folder) = self.path.parent() {
            fs::create_dir_all(folder)?;
        }
        let size = fs::metadata(&self.path).map_or(0, |found| found.len());
        if size > 0 && size + line.len() as u64 > self.max_bytes {
            self.rotate();
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(line.as_bytes())
    }

    /// `app.log` becomes `app.log.1`, that one `app.log.2`, and so on; the
    /// oldest beyond `files_kept` is dropped.
    fn rotate(&self) {
        let aside = |n: usize| PathBuf::from(format!("{}.{n}", self.path.display()));
        let _ = fs::remove_file(aside(self.files_kept - 1));
        for n in (1..self.files_kept - 1).rev() {
            let _ = fs::rename(aside(n), aside(n + 1));
        }
        if self.files_kept > 1 {
            let _ = fs::rename(&self.path, aside(1));
        } else {
            let _ = fs::remove_file(&self.path);
        }
    }
}
