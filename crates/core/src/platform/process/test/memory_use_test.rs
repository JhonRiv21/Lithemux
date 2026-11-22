use crate::platform::process::memory_use::descendants;

// PLAN §3.3: memory per session, from its process tree.
#[test]
fn a_tree_is_the_root_and_everything_below_it() {
    let parents = [(2, 1), (3, 2), (4, 2), (5, 99), (6, 4)];
    let mut tree = descendants(&parents, 1);
    tree.sort_unstable();
    assert_eq!(tree, [1, 2, 3, 4, 6]);
    assert_eq!(descendants(&parents, 5), [5]);
}

// PLAN §3.3: a process that is not below the session is not counted.
#[test]
fn a_process_elsewhere_is_not_in_the_tree() {
    let parents = [(2, 1), (3, 7)];
    assert!(!descendants(&parents, 1).contains(&3));
}

// A cycle in what the kernel reported (a pid reused while listing) does
// not make the walk go round forever.
#[test]
fn a_cycle_ends_the_walk() {
    let parents = [(2, 1), (1, 2)];
    let mut tree = descendants(&parents, 1);
    tree.sort_unstable();
    assert_eq!(tree, [1, 2]);
}
