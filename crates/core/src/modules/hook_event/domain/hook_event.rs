/// The seven Claude Code events lithemux subscribes to, named by the
/// subcommand the hook binary receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    SessionStart,
    PromptSubmit,
    Stop,
    Notification,
    SessionEnd,
    PostToolUse,
    PostToolUseFailure,
}

impl HookEvent {
    pub fn parse(subcommand: &str) -> Option<Self> {
        match subcommand {
            "session-start" => Some(HookEvent::SessionStart),
            "prompt-submit" => Some(HookEvent::PromptSubmit),
            "stop" => Some(HookEvent::Stop),
            "notification" => Some(HookEvent::Notification),
            "session-end" => Some(HookEvent::SessionEnd),
            "post-tool-use" => Some(HookEvent::PostToolUse),
            "post-tool-use-failure" => Some(HookEvent::PostToolUseFailure),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            HookEvent::SessionStart => "session-start",
            HookEvent::PromptSubmit => "prompt-submit",
            HookEvent::Stop => "stop",
            HookEvent::Notification => "notification",
            HookEvent::SessionEnd => "session-end",
            HookEvent::PostToolUse => "post-tool-use",
            HookEvent::PostToolUseFailure => "post-tool-use-failure",
        }
    }
}
