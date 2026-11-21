use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    New,
    Working,
    Idle,
    Waiting,
    Hibernated,
    Ended,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::New => "new",
            State::Working => "working",
            State::Idle => "idle",
            State::Waiting => "waiting",
            State::Hibernated => "hibernated",
            State::Ended => "ended",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "new" => Some(State::New),
            "working" => Some(State::Working),
            "idle" => Some(State::Idle),
            "waiting" => Some(State::Waiting),
            "hibernated" => Some(State::Hibernated),
            "ended" => Some(State::Ended),
            _ => None,
        }
    }
}
