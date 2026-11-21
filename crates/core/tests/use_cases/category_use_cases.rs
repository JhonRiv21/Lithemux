use lithemux_core::modules::category::{
    CategoryError, create_category, get_category, list_categories, remove_category,
};
use lithemux_core::modules::session::{Kind, list_sessions};

use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::sample_data::{NOW, add_category, add_session, new_category};

// PLAN §0.5: the board is created when the category is created
#[test]
fn creating_a_category_also_creates_its_board() {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    let category = create_category(&store, &boards, new_category("selektal"), NOW).unwrap();
    assert_eq!(category.slug, "selektal");
    assert_eq!(category.root_dir, "/work/selektal");
    assert_eq!(category.created_at, NOW);
    assert_eq!(*boards.created.borrow(), vec!["selektal".to_string()]);
}

// PLAN §0.2: slug UNIQUE
#[test]
fn a_category_slug_cannot_be_used_twice() {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    add_category(&store, &boards, "demo");
    let again = create_category(&store, &boards, new_category("demo"), NOW);
    assert!(matches!(again, Err(CategoryError::AlreadyExists(slug)) if slug == "demo"));
    assert_eq!(list_categories(&store).unwrap().len(), 1);
}

// PLAN §0.6: `cat list` keeps creation order
#[test]
fn categories_are_listed_in_the_order_they_were_created() {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    for slug in ["zeta", "alfa", "media"] {
        add_category(&store, &boards, slug);
    }
    let slugs: Vec<String> = list_categories(&store)
        .unwrap()
        .into_iter()
        .map(|c| c.slug)
        .collect();
    assert_eq!(slugs, ["zeta", "alfa", "media"]);
}

// PLAN §0.2: ON DELETE CASCADE
#[test]
fn removing_a_category_removes_its_sessions() {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    let demo = add_category(&store, &boards, "demo");
    let other = add_category(&store, &boards, "otra");
    add_session(&store, &store, "demo/planificadora", Kind::Mother);
    add_session(&store, &store, "otra/ejecutora", Kind::Child);

    remove_category(&store, "demo").unwrap();

    assert!(matches!(
        get_category(&store, "demo"),
        Err(CategoryError::NotFound(_))
    ));
    assert!(list_sessions(&store, &demo).unwrap().is_empty());
    assert_eq!(list_sessions(&store, &other).unwrap().len(), 1);
}

// PLAN §0.6
#[test]
fn removing_a_category_that_does_not_exist_fails() {
    let store = InMemoryStore::new();
    assert!(matches!(
        remove_category(&store, "nada"),
        Err(CategoryError::NotFound(_))
    ));
}
