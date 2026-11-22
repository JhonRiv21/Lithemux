use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Coordinates the category and keeps its board. One per category.
    Coordinator,
    /// Any other session of the category.
    Member,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Coordinator => "coordinator",
            Kind::Member => "member",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "coordinator" => Some(Kind::Coordinator),
            "member" => Some(Kind::Member),
            _ => None,
        }
    }
}
