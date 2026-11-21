use std::fmt;

use crate::modules::category::domain::slug_policy::is_valid_slug;
use crate::modules::category::error::InvalidSlug;

/// Validated identifier, used by categories and sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slug(String);

impl Slug {
    pub fn parse(value: &str) -> Result<Self, InvalidSlug> {
        if is_valid_slug(value) {
            Ok(Slug(value.to_string()))
        } else {
            Err(InvalidSlug(value.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
