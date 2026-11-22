use crate::sidebar::order::{dropped_on, moved, shifted};

fn ids(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

// PLAN §2.2: reorder categories and sessions.
#[test]
fn an_item_moves_to_the_place_given_and_the_rest_keep_their_order() {
    let list = ids(&["a", "b", "c", "d"]);
    assert_eq!(moved(&list, "c", 0), ids(&["c", "a", "b", "d"]));
    assert_eq!(moved(&list, "a", 2), ids(&["b", "c", "a", "d"]));
    assert_eq!(moved(&list, "a", 99), ids(&["b", "c", "d", "a"]));
}

// PLAN §2.2
#[test]
fn moving_something_that_is_not_in_the_list_changes_nothing() {
    let list = ids(&["a", "b"]);
    assert_eq!(moved(&list, "x", 0), list);
    assert_eq!(shifted(&list, "x", 1), list);
    assert_eq!(dropped_on(&list, "a", "x"), list);
}

// PLAN §2.2
#[test]
fn up_and_down_move_one_place_and_stop_at_the_ends() {
    let list = ids(&["a", "b", "c"]);
    assert_eq!(shifted(&list, "b", -1), ids(&["b", "a", "c"]));
    assert_eq!(shifted(&list, "b", 1), ids(&["a", "c", "b"]));
    assert_eq!(shifted(&list, "a", -1), list);
    assert_eq!(shifted(&list, "c", 1), list);
}

// PLAN §2.2: dragging a card onto another puts it in that place.
#[test]
fn a_card_dropped_on_another_takes_its_place() {
    let list = ids(&["a", "b", "c", "d"]);
    assert_eq!(dropped_on(&list, "d", "b"), ids(&["a", "d", "b", "c"]));
    assert_eq!(dropped_on(&list, "a", "c"), ids(&["b", "c", "a", "d"]));
    assert_eq!(dropped_on(&list, "b", "b"), list);
}

// Audit of block 3 (6): the items that are not shown keep their places
// while the shown ones take a new order among themselves.
#[test]
fn a_new_order_of_the_shown_items_leaves_the_hidden_ones_where_they_were() {
    use crate::sidebar::order::with_shown_reordered;
    let all = ids(&["coordinator", "b", "pinned", "d", "e"]);
    let new_order = ids(&["e", "b", "d"]);
    assert_eq!(
        with_shown_reordered(&all, &new_order),
        ids(&["coordinator", "e", "pinned", "b", "d"])
    );
    assert_eq!(with_shown_reordered(&all, &[]), all);
}
