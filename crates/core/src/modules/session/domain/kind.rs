use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Mother,
    Child,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Mother => "mother",
            Kind::Child => "child",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "mother" => Some(Kind::Mother),
            "child" => Some(Kind::Child),
            _ => None,
        }
    }
}
