use std::fmt;

use crate::modules::category::Slug;
use crate::modules::session::domain::messaging_name::messaging_name;
use crate::modules::session::error::SessionError;

/// A `<category>/<session>` reference as typed on the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRef {
    pub category: Slug,
    pub session: Slug,
}

impl SessionRef {
    pub fn parse(value: &str) -> Result<Self, SessionError> {
        let (category, session) = value
            .split_once('/')
            .ok_or_else(|| SessionError::InvalidRef(value.to_string()))?;
        if category.is_empty() || session.is_empty() || session.contains('/') {
            return Err(SessionError::InvalidRef(value.to_string()));
        }
        Ok(SessionRef {
            category: Slug::parse(category)?,
            session: Slug::parse(session)?,
        })
    }

    pub fn messaging_name(&self) -> String {
        messaging_name(self.category.as_str(), self.session.as_str())
    }
}

impl fmt::Display for SessionRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.category, self.session)
    }
}
