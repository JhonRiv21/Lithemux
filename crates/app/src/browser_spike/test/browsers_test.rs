use std::cell::Cell;
use std::convert::Infallible;
use std::rc::Rc;

use lithemux_core::modules::workspace::{Axis, Layout};

use crate::browser_spike::browsers::{Browsers, Surface};

/// What the user would see of a page: whether it is shown, and whether it
/// still exists (a dropped page is gone from the window).
#[derive(Default)]
struct Seen {
    visible: Cell<bool>,
    alive: Cell<bool>,
}

impl Seen {
    fn on_screen(&self) -> bool {
        self.alive.get() && self.visible.get()
    }
}

struct FakePage(Rc<Seen>);

impl Surface for FakePage {
    fn show(&self, visible: bool) {
        self.0.visible.set(visible);
    }
    fn focus(&self) {}
    fn give_back_focus(&self) {}
}

impl Drop for FakePage {
    fn drop(&mut self) {
        self.0.alive.set(false);
    }
}

fn open(browsers: &mut Browsers<FakePage>, layout: &Layout, pane: u32) -> Rc<Seen> {
    let seen = Rc::new(Seen::default());
    seen.alive.set(true);
    let over = layout.pane(pane).and_then(|pane| pane.active_session());
    let page = FakePage(Rc::clone(&seen));
    browsers
        .open(pane, over.map(str::to_string), || Ok::<_, Infallible>(page))
        .unwrap();
    browsers.follow_the_layout(layout);
    seen
}

/// Left pane with session a; right pane, the newest, with b and its page
/// on show.
fn split_with_a_page_on_the_right() -> (Layout, Browsers<FakePage>, u32, Rc<Seen>) {
    let mut layout = Layout::single();
    layout.open("a");
    let right = layout.split(1, Axis::Horizontal).unwrap();
    layout.open("b");
    let mut browsers = Browsers::default();
    let seen = open(&mut browsers, &layout, right);
    (layout, browsers, right, seen)
}

// PLAN §3.7, criterion 1: a page never stays on screen outside a pane
// that exists, however the pane went: here, its only session was deleted.
#[test]
fn a_page_never_stays_on_screen_outside_a_live_pane() {
    let (mut layout, mut browsers, right, seen) = split_with_a_page_on_the_right();
    assert!(seen.on_screen());

    layout.keep_sessions(|id| id != "b");
    browsers.follow_the_layout(&layout);

    assert!(layout.pane(right).is_none());
    assert!(!seen.on_screen(), "the page stayed over the other pane");
    assert!(!browsers.has(right));
}

// PLAN §3.7, criterion 1: a pane made after one closed does not get the
// closed one's page, even when the layout changed twice before the pages
// followed it.
#[test]
fn a_new_pane_does_not_get_the_page_of_a_closed_one() {
    let (mut layout, mut browsers, right, seen) = split_with_a_page_on_the_right();

    layout.keep_sessions(|id| id != "b");
    let added = layout.split(1, Axis::Vertical).unwrap();
    browsers.follow_the_layout(&layout);

    assert_ne!(added, right);
    assert!(!browsers.has(added));
    assert!(!seen.alive.get());
}

// PLAN §3.7, criterion 2: a pane that goes on to show another session
// hides its page, and keeps it for when its tab is chosen again.
#[test]
fn showing_another_session_in_the_pane_hides_its_page() {
    let (mut layout, mut browsers, right, seen) = split_with_a_page_on_the_right();

    layout.focus_pane(right);
    layout.open("c");
    browsers.follow_the_layout(&layout);

    assert!(!seen.on_screen());
    assert!(seen.alive.get());
    assert!(browsers.has(right));
}

// PLAN §3.7, criterion 3: while gpui draws something over the panes, no
// page shows, and the chosen one comes back after.
#[test]
fn a_covered_page_hides_and_comes_back() {
    let (_, mut browsers, _, seen) = split_with_a_page_on_the_right();

    browsers.set_covered(true);
    assert!(!seen.on_screen());

    browsers.set_covered(false);
    assert!(seen.on_screen());
}
