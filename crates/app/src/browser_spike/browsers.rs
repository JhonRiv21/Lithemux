use std::collections::HashMap;
use std::rc::Rc;

use lithemux_core::modules::workspace::Layout;

/// What the workspace does with a page. The real one is a WebKit view; a
/// test's only remembers.
pub trait Surface {
    fn show(&self, visible: bool);
    fn focus(&self);
    fn give_back_focus(&self);
}

struct BrowserTab<P> {
    page: Rc<P>,
    /// Its tab is the one chosen in its pane.
    chosen: bool,
    /// The session the pane had on show when the page was chosen: once the
    /// pane shows another, the user chose that one.
    over: Option<String>,
    /// What the page was last told: it is told again only on a change.
    visible: bool,
}

/// The browser tabs of the workspace, by pane.
pub struct Browsers<P> {
    tabs: HashMap<u32, BrowserTab<P>>,
    /// Something of gpui is drawn where a page could be: every page hides,
    /// since the page is drawn over gpui (D-64).
    covered: bool,
    /// The pane whose page has the keyboard, or should have it once
    /// nothing covers it.
    keyboard: Option<u32>,
}

impl<P> Default for Browsers<P> {
    fn default() -> Self {
        Browsers {
            tabs: HashMap::new(),
            covered: false,
            keyboard: None,
        }
    }
}

impl<P: Surface> Browsers<P> {
    /// Opens the pane's page with `make`, or chooses it if it is there.
    /// `over` is the session the pane has on show.
    pub fn open<E>(
        &mut self,
        pane: u32,
        over: Option<String>,
        make: impl FnOnce() -> Result<P, E>,
    ) -> Result<(), E> {
        if let Some(tab) = self.tabs.get_mut(&pane) {
            tab.chosen = true;
            tab.over = over;
            return Ok(());
        }
        let tab = BrowserTab {
            page: Rc::new(make()?),
            chosen: true,
            over,
            visible: false,
        };
        self.tabs.insert(pane, tab);
        Ok(())
    }

    pub fn has(&self, pane: u32) -> bool {
        self.tabs.contains_key(&pane)
    }

    pub fn is_chosen(&self, pane: u32) -> bool {
        self.tabs.get(&pane).is_some_and(|tab| tab.chosen)
    }

    pub fn choose(&mut self, pane: u32, chosen: bool) {
        if let Some(tab) = self.tabs.get_mut(&pane) {
            tab.chosen = chosen;
        }
    }

    pub fn close(&mut self, pane: u32) {
        self.tabs.remove(&pane);
    }

    /// After any change of the layout, however it came: the page of a pane
    /// that is gone goes with it, and a pane that shows another session
    /// than when its page was chosen shows that session.
    pub fn follow_the_layout(&mut self, layout: &Layout) {
        self.tabs.retain(|pane, _| layout.pane(*pane).is_some());
        for (pane, tab) in &mut self.tabs {
            let on_show = layout.pane(*pane).and_then(|pane| pane.active_session());
            if tab.chosen && on_show != tab.over.as_deref() {
                tab.chosen = false;
            }
        }
        self.show_what_is_chosen();
    }

    /// What covers the pages needs the keyboard too: a hidden page would
    /// keep it, since AppKit's keyboard is apart from gpui's focus.
    pub fn set_covered(&mut self, covered: bool) {
        if covered == self.covered {
            return;
        }
        self.covered = covered;
        self.show_what_is_chosen();
        match self.keyboard.and_then(|pane| self.tabs.get(&pane)) {
            Some(tab) if covered => tab.page.give_back_focus(),
            Some(tab) if tab.chosen => tab.page.focus(),
            _ => {}
        }
    }

    /// A click in the pane's page gave it the keyboard.
    pub fn clicked(&mut self, pane: u32) {
        self.keyboard = Some(pane);
    }

    /// The page of a pane whose browser tab is chosen and in view.
    pub fn shown_page(&self, pane: u32) -> Option<Rc<P>> {
        let tab = self.tabs.get(&pane)?;
        (tab.chosen && !self.covered).then(|| Rc::clone(&tab.page))
    }

    /// Each page shows only while its tab is chosen and nothing covers it.
    fn show_what_is_chosen(&mut self) {
        for tab in self.tabs.values_mut() {
            let visible = tab.chosen && !self.covered;
            if visible != tab.visible {
                tab.page.show(visible);
                tab.visible = visible;
            }
        }
    }

    /// The keyboard goes to the pane's page if it is shown, or back to the
    /// window's view, where gpui has it.
    pub fn focus(&mut self, pane: Option<u32>) {
        self.keyboard = pane.filter(|pane| self.is_chosen(*pane));
        match pane.and_then(|pane| self.shown_page(pane)) {
            Some(page) => page.focus(),
            None if self.covered => {}
            None => {
                if let Some(tab) = self.tabs.values().next() {
                    tab.page.give_back_focus();
                }
            }
        }
    }
}
