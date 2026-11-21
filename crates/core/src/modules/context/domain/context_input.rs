use std::path::Path;

use crate::modules::category::Category;
use crate::modules::context::domain::lang::Lang;
use crate::modules::session::Session;

pub const MAX_CONTEXT_CHARS: usize = 2000;

pub struct ContextInput<'a> {
    pub category: &'a Category,
    pub me: &'a Session,
    /// Every other session of the category, in display order.
    pub others: &'a [Session],
    pub board: &'a Path,
    pub now: i64,
    pub lang: Lang,
}
