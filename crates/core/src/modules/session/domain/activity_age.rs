use std::fmt;

/// How long ago a session was last active, in the unit people read it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityAge {
    JustNow,
    Minutes(i64),
    Hours(i64),
    Days(i64),
}

impl ActivityAge {
    /// A moment in the future (a clock that moved back) counts as now.
    pub fn between(now: i64, last_activity_at: i64) -> Self {
        let seconds = (now - last_activity_at).max(0);
        let (minutes, hours, days) = (seconds / 60, seconds / 3600, seconds / 86_400);
        if minutes < 1 {
            ActivityAge::JustNow
        } else if hours < 1 {
            ActivityAge::Minutes(minutes)
        } else if days < 1 {
            ActivityAge::Hours(hours)
        } else {
            ActivityAge::Days(days)
        }
    }
}

/// The English wording, shared by the CLI and the English context.
impl fmt::Display for ActivityAge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityAge::JustNow => f.write_str("just now"),
            ActivityAge::Minutes(minutes) => write!(f, "{minutes} min ago"),
            ActivityAge::Hours(hours) => write!(f, "{hours} h ago"),
            ActivityAge::Days(1) => f.write_str("1 day ago"),
            ActivityAge::Days(days) => write!(f, "{days} days ago"),
        }
    }
}
