use lithemux_core::modules::category::{Category, CategoryRepository};

fn category(id: &str, slug: &str, sort: i64) -> Category {
    Category {
        id: id.to_string(),
        slug: slug.to_string(),
        name: slug.to_uppercase(),
        root_dir: format!("/work/{slug}"),
        color: Some("#0094ff".to_string()),
        sort,
        created_at: 10,
    }
}

/// What every CategoryRepository must do. PLAN §0.2.
pub fn category_repository_contract(repo: &dyn CategoryRepository) {
    assert_eq!(repo.next_sort().unwrap(), 1);
    assert!(repo.list().unwrap().is_empty());

    let second = category("c2", "selektal", 2);
    let first = category("c1", "demo", 1);
    repo.insert(&second).unwrap();
    repo.insert(&first).unwrap();

    assert_eq!(repo.list().unwrap(), vec![first.clone(), second.clone()]);
    assert_eq!(repo.next_sort().unwrap(), 3);
    assert_eq!(repo.find_by_slug("demo").unwrap(), Some(first.clone()));
    assert_eq!(repo.find_by_id("c2").unwrap(), Some(second.clone()));
    assert_eq!(repo.find_by_slug("missing").unwrap(), None);
    assert_eq!(repo.find_by_id("missing").unwrap(), None);

    repo.delete("c1").unwrap();
    assert_eq!(repo.list().unwrap(), vec![second]);
    repo.delete("c1").unwrap();
}
