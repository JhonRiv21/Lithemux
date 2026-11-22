use crate::modules::category::{self, Category};

/// A domain type of another module, reached every way a path can be written.
pub fn name(category: &Category, rule: &category::Rule) -> String {
    let _: Option<&super::super::super::category::Category> = None;
    format!("{}-{rule:?}", category.slug)
}
