/// One terminal of a cmux workspace, with only what the import uses of it.
/// Everything else in cmux's file, the text of its notifications among
/// it, is never read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CmuxPanel {
    /// The name the user gave the workspace.
    pub custom_title: Option<String>,
    /// The title the program in it set.
    pub process_title: Option<String>,
    /// Where it works.
    pub folder: Option<String>,
    /// Whether the workspace is pinned.
    pub pinned: bool,
    /// The kind of agent cmux saw in it (`claude`, …).
    pub agent: Option<String>,
    /// The conversation of that agent.
    pub claude_session_id: Option<String>,
}
