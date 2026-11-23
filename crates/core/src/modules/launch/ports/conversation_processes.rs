/// Who else runs a conversation. Two `claude` on one conversation lose
/// messages, and the lock only knows about lithemux's own launches: a
/// `claude` that cmux or a terminal started holds none.
pub trait ConversationProcesses {
    /// The pid of a live process that runs this conversation, if any.
    fn holder_of(&self, claude_session_id: &str) -> Option<u32>;
}
