use crate::modules::category::{
    Category, NewCategory,
    create_category,
};

pub fn policy(_: &Category, _: NewCategory) {
    let _ = create_category;
}
