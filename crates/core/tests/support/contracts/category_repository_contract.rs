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
        collapsed: false,
        notify: true,
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

    // What the sidebar changes (PLAN §2.2).
    repo.update_name("c1", "Demo app").unwrap();
    repo.update_collapsed("c1", true).unwrap();
    let changed = repo.find_by_id("c1").unwrap().unwrap();
    assert_eq!(
        (changed.name.as_str(), changed.collapsed),
        ("Demo app", true)
    );
    assert_eq!(changed.slug, "demo");
    repo.update_sort("c1", 5).unwrap();
    let slugs: Vec<String> = repo.list().unwrap().into_iter().map(|c| c.slug).collect();
    assert_eq!(slugs, ["selektal", "demo"]);
    repo.update_name("missing", "x").unwrap();
    // Notifications are on until switched off (PLAN §3.1).
    assert!(repo.find_by_id("c2").unwrap().unwrap().notify);
    repo.update_notify("c2", false).unwrap();
    assert!(!repo.find_by_id("c2").unwrap().unwrap().notify);
    repo.update_notify("c2", true).unwrap();
    // What is inserted is kept whole, this flag too.
    let quiet = Category {
        notify: false,
        ..category("c3", "quiet", 9)
    };
    repo.insert(&quiet).unwrap();
    assert_eq!(repo.find_by_id("c3").unwrap(), Some(quiet));
    repo.delete("c3").unwrap();

    repo.delete("c1").unwrap();
    assert_eq!(repo.list().unwrap(), vec![second]);
    repo.delete("c1").unwrap();
}
