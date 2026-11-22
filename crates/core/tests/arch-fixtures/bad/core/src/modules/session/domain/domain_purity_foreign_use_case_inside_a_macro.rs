use crate::modules::category;

pub fn policy() -> String {
    format!("{:p}", category::create_category as *const ())
}
