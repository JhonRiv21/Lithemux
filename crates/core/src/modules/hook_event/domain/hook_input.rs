use serde::Deserialize;

/// The fields of the hook's stdin JSON that lithemux uses.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct HookInput {
    /// Claude Code's own session id.
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub notification_type: Option<String>,
    /// Not documented today; used only if Claude Code ever sends it.
    #[serde(default)]
    pub version: Option<String>,
}

impl HookInput {
    /// Never fails: unreadable input is the same as no input.
    pub fn from_json(text: &str) -> Self {
        serde_json::from_str(text).unwrap_or_default()
    }
}
