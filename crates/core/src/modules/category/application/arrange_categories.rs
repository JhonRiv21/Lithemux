use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::category_repository::CategoryRepository;

/// Changes the name shown. The slug, part of every session's address,
/// never changes. An empty name is not one.
pub fn rename_category(
    categories: &dyn CategoryRepository,
    category_id: &str,
    typed: &str,
) -> Result<(), CategoryError> {
    let name = typed.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err(CategoryError::EmptyName);
    }
    categories.update_name(category_id, &name)
}

/// Puts the categories in the order given. One that is not named keeps its
/// place after the named ones; an unknown id is ignored.
pub fn reorder_categories(
    categories: &dyn CategoryRepository,
    ordered_ids: &[String],
) -> Result<(), CategoryError> {
    let current = categories.list()?;
    let named = ordered_ids
        .iter()
        .filter(|id| current.iter().any(|category| &category.id == *id));
    let unnamed = current
        .iter()
        .map(|category| &category.id)
        .filter(|id| !ordered_ids.contains(id));
    for (place, id) in (1_i64..).zip(named.chain(unnamed)) {
        categories.update_sort(id, place)?;
    }
    Ok(())
}

/// Whether a turn the category's sessions finish out of view rings.
pub fn set_notify(
    categories: &dyn CategoryRepository,
    category_id: &str,
    notify: bool,
) -> Result<(), CategoryError> {
    categories.update_notify(category_id, notify)
}

pub fn set_collapsed(
    categories: &dyn CategoryRepository,
    category_id: &str,
    collapsed: bool,
) -> Result<(), CategoryError> {
    categories.update_collapsed(category_id, collapsed)
}
