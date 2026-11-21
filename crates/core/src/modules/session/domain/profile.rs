use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Claude,
    Shell,
}

impl Profile {
    pub fn as_str(self) -> &'static str {
        match self {
            Profile::Claude => "claude",
            Profile::Shell => "shell",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "claude" => Some(Profile::Claude),
            "shell" => Some(Profile::Shell),
            _ => None,
        }
    }
}
