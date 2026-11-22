/// What became of a request to open a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// It is being prepared; `phase` will say how it went.
    Opening { session_id: String },
    /// It was already running, or being opened, here.
    AlreadyHere { session_id: String },
    /// Its last run is still leaving; it opens as soon as that is over.
    WhenClosed { session_id: String },
}

impl Opened {
    pub fn session_id(&self) -> &str {
        match self {
            Opened::Opening { session_id }
            | Opened::AlreadyHere { session_id }
            | Opened::WhenClosed { session_id } => session_id,
        }
    }
}
