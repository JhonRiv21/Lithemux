use crate::modules::category::domain::slug_policy::is_valid_slug;

#[test]
fn slug_cannot_be_empty() {
    assert!(!is_valid_slug(""));
}
