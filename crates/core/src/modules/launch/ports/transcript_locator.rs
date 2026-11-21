pub trait TranscriptLocator {
    /// Whether Claude Code already has a conversation with this session id.
    fn exists(&self, claude_session_id: &str) -> bool;
}
