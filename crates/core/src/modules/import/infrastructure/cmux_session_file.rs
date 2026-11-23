//! The only place that knows where cmux keeps its workspaces, and it only
//! reads: nothing here creates, writes, moves or removes a file (a guard,
//! `cmux_read_only_guard`, checks it).

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::modules::import::domain::cmux_panel::CmuxPanel;
use crate::modules::import::error::ImportError;
use crate::modules::import::infrastructure::cmux_format::{SessionsFile, Versioned};
use crate::modules::import::ports::cmux_snapshot::CmuxSnapshot;

/// Under the user's home.
const CMUX_SESSIONS: [&str; 4] = [
    "Library",
    "Application Support",
    "cmux",
    "session-com.cmuxterm.app.json",
];
const KNOWN_VERSION: i64 = 1;
/// cmux's own file is about 100 KB with two dozen workspaces: anything this
/// big is not one, and is not read into memory.
const MAX_MEGABYTES: u64 = 20;
const MEGABYTE: u64 = 1024 * 1024;
/// cmux rewrites the file every now and then: a read that finds it half
/// written tries once more, a moment later.
const RETRY_AFTER: Duration = Duration::from_millis(250);

pub struct CmuxSessionFile {
    path: PathBuf,
}

impl CmuxSessionFile {
    /// Where cmux keeps it for this user.
    pub fn of_user(home: &Path) -> Self {
        let path = CMUX_SESSIONS
            .iter()
            .fold(home.to_path_buf(), |path, part| path.join(part));
        CmuxSessionFile { path }
    }

    pub fn at(path: impl Into<PathBuf>) -> Self {
        CmuxSessionFile { path: path.into() }
    }

    fn read_once(&self) -> Result<Vec<CmuxPanel>, ImportError> {
        let size = match std::fs::metadata(&self.path) {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ImportError::NoCmuxFile(self.path.clone()));
            }
            Err(error) => return Err(self.unreadable(error)),
        };
        if size > MAX_MEGABYTES * MEGABYTE {
            return Err(ImportError::TooBig {
                path: self.path.clone(),
                megabytes: size / MEGABYTE,
                limit: MAX_MEGABYTES,
            });
        }
        let bytes = std::fs::read(&self.path).map_err(|error| self.unreadable(error))?;
        let versioned: Versioned =
            serde_json::from_slice(&bytes).map_err(|error| self.not_parsed(error))?;
        match versioned.version {
            None => return Err(ImportError::MissingVersion),
            Some(version) => match version.as_i64() {
                Some(KNOWN_VERSION) => {}
                Some(other) => return Err(ImportError::UnknownVersion(other)),
                None => return Err(self.format("the version is not a whole number")),
            },
        }
        let file: SessionsFile =
            serde_json::from_slice(&bytes).map_err(|error| self.not_parsed(error))?;
        Ok(file.panels())
    }

    /// Cut short, or not JSON at all: most likely caught while cmux wrote
    /// it, and worth one more try. JSON of the wrong shape is a format
    /// problem, and another try would find the same.
    fn not_parsed(&self, error: serde_json::Error) -> ImportError {
        if error.is_syntax() || error.is_eof() {
            self.unreadable(error)
        } else {
            self.format(&error.to_string())
        }
    }

    fn format(&self, detail: &str) -> ImportError {
        ImportError::Format {
            path: self.path.clone(),
            detail: detail.to_string(),
        }
    }

    fn unreadable(&self, error: impl std::fmt::Display) -> ImportError {
        ImportError::Unreadable {
            path: self.path.clone(),
            detail: error.to_string(),
        }
    }
}

impl CmuxSnapshot for CmuxSessionFile {
    fn panels(&self) -> Result<Vec<CmuxPanel>, ImportError> {
        match self.read_once() {
            Err(ImportError::Unreadable { .. }) => {
                std::thread::sleep(RETRY_AFTER);
                self.read_once()
            }
            read => read,
        }
    }
}
