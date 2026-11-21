//! The only place that knows where Claude Code keeps its files. The transcript
//! path is documented; its format is not, so only existence is checked.

use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use crate::modules::launch::ports::transcript_locator::TranscriptLocator;

pub struct ClaudeProjectsTranscriptLocator {
    config_dir: Option<PathBuf>,
}

impl ClaudeProjectsTranscriptLocator {
    /// `CLAUDE_CONFIG_DIR` when set, otherwise `~/.claude`.
    pub fn from_env() -> Self {
        let config_dir = std::env::var_os("CLAUDE_CONFIG_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::home_dir().map(|home| home.join(".claude")));
        ClaudeProjectsTranscriptLocator { config_dir }
    }

    pub fn in_config_dir(config_dir: impl Into<PathBuf>) -> Self {
        ClaudeProjectsTranscriptLocator {
            config_dir: Some(config_dir.into()),
        }
    }
}

impl TranscriptLocator for ClaudeProjectsTranscriptLocator {
    fn exists(&self, claude_session_id: &str) -> bool {
        let Some(config_dir) = self.config_dir.as_deref() else {
            return false;
        };
        if Uuid::parse_str(claude_session_id).is_err() {
            return false;
        }
        let file_name = format!("{claude_session_id}.jsonl");
        let Ok(projects) = fs::read_dir(config_dir.join("projects")) else {
            return false;
        };
        projects
            .flatten()
            .any(|project| project.path().join(&file_name).is_file())
    }
}
