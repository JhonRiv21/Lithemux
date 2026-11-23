//! Who runs a conversation, told from the command lines of the processes:
//! any `claude` started with `--resume <id>` or `--session-id <id>`.

use crate::modules::launch::domain::conversation_in_use::{CONVERSATION_FLAGS, conversation_in};
use crate::modules::launch::ports::conversation_processes::ConversationProcesses;
use crate::platform::process::command_lines::command_lines_with;

pub struct CommandLineConversations;

impl ConversationProcesses for CommandLineConversations {
    fn holder_of(&self, claude_session_id: &str) -> Option<u32> {
        command_lines_with(&CONVERSATION_FLAGS)
            .into_iter()
            .find(|line| conversation_in(&line.args) == Some(claude_session_id))
            .map(|line| line.pid)
    }
}
