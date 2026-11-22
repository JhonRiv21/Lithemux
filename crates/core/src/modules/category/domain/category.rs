use serde::Serialize;

use crate::modules::category::domain::slug::Slug;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Category {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub root_dir: String,
    pub color: Option<String>,
    pub sort: i64,
    pub created_at: i64,
    /// Whether the sidebar shows it folded.
    pub collapsed: bool,
    /// Whether a turn its sessions finish out of view rings («Sound when
    /// done»). Stored as `notify` (migration 5).
    pub notify: bool,
}

#[derive(Debug, Clone)]
pub struct NewCategory {
    pub slug: Slug,
    pub name: String,
    pub root_dir: String,
    pub color: Option<String>,
}
