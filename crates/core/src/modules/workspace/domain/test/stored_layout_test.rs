use crate::modules::workspace::domain::layout::Layout;
use crate::modules::workspace::domain::split_tree::{Axis, Node};
use crate::modules::workspace::domain::stored_layout::{from_stored, to_stored};

fn busy_layout() -> Layout {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    layout.open("b");
    let right = layout.split(first, Axis::Horizontal).unwrap();
    layout.open("c");
    layout.split(right, Axis::Vertical).unwrap();
    layout.open("d");
    layout.focus_pane(first);
    layout
}

// PLAN §2.3: the layout and the content of the panes are saved.
#[test]
fn a_layout_comes_back_as_it_was_saved() {
    let layout = busy_layout();
    assert_eq!(from_stored(&to_stored(&layout)), Some(layout));
}

// PLAN §2.3
#[test]
fn a_restored_layout_can_still_be_split() {
    let mut restored = from_stored(&to_stored(&busy_layout())).unwrap();
    let active = restored.active_pane().id;
    let added = restored.split(active, Axis::Vertical).unwrap();
    let ids: Vec<u32> = restored.panes_in_order().iter().map(|p| p.id).collect();
    assert_eq!(ids.iter().filter(|id| **id == added).count(), 1);
    assert_eq!(ids.len(), 4);
}

// PLAN principle 3: what is stored may be damaged; the app still starts.
#[test]
fn text_that_is_not_a_layout_is_not_restored() {
    for stored in ["", "not json", "{}", "[1,2]", r#"{"root":{"Pane":1}}"#] {
        assert_eq!(from_stored(stored), None, "{stored}");
    }
}

fn edited(change: impl FnOnce(&mut serde_json::Value)) -> String {
    let mut value: serde_json::Value = serde_json::from_str(&to_stored(&busy_layout())).unwrap();
    change(&mut value);
    value.to_string()
}

// PLAN principle 3
#[test]
fn a_layout_that_contradicts_itself_is_not_restored() {
    // The focused pane is not one of the panes.
    assert_eq!(
        from_stored(&edited(|v| v["active_pane"] = 999.into())),
        None
    );
    // A pane the tree does not show.
    assert_eq!(
        from_stored(&edited(|v| {
            let pane = v["panes"][0].clone();
            let mut extra = pane;
            extra["id"] = 777.into();
            extra["tabs"] = serde_json::json!([]);
            v["panes"].as_array_mut().unwrap().push(extra);
        })),
        None
    );
    // The same session in two tabs.
    assert_eq!(
        from_stored(&edited(|v| v["panes"][1]["tabs"] = serde_json::json!(["a"]))),
        None
    );
    // The shown tab is past the end.
    assert_eq!(
        from_stored(&edited(|v| v["panes"][0]["active"] = 9.into())),
        None
    );
}

// PLAN §2.3: up to 4 panes.
#[test]
fn a_stored_layout_with_too_many_panes_is_not_restored() {
    let mut layout = busy_layout();
    let first = layout.panes_in_order()[0].id;
    layout.split(first, Axis::Vertical).unwrap();
    let four = to_stored(&layout);
    assert!(from_stored(&four).is_some());
    // A fifth pane, written by hand into the tree and the list.
    let five = {
        let mut value: serde_json::Value = serde_json::from_str(&four).unwrap();
        let root = value["root"].take();
        value["root"] = serde_json::json!({
            "Split": {"id": 900, "axis": "Horizontal", "share": 500,
                      "first": root, "second": {"Pane": 901}}
        });
        value["panes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"id": 901, "tabs": [], "active": 0}));
        value.to_string()
    };
    assert_eq!(from_stored(&five), None);
}

// PLAN §2.3
#[test]
fn a_stored_divider_outside_its_limits_is_brought_back_in() {
    let stored = edited(|v| v["root"]["Split"]["share"] = 5.into());
    let restored = from_stored(&stored).unwrap();
    let Node::Split { share, .. } = restored.root() else {
        panic!("the root is not a split");
    };
    assert_eq!(*share, 100);
}

// Audit of block 3 (4): the panes stored must be exactly the ones the tree
// shows. Two entries for one pane leave another pane without its tabs.
#[test]
fn a_stored_layout_that_names_a_pane_twice_is_not_restored() {
    let stored = serde_json::json!({
        "root": {"Split": {"id": 2, "axis": "Horizontal", "share": 500,
                           "first": {"Pane": 1}, "second": {"Pane": 3}}},
        "panes": [{"id": 1, "tabs": ["a"], "active": 0},
                  {"id": 1, "tabs": [], "active": 0}],
        "active_pane": 1
    });
    assert_eq!(from_stored(&stored.to_string()), None);
}
