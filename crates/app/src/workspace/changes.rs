//! What the user does to the workspace: open, split, close, move.

use gpui::{Context, Window};
use lithemux_core::modules::workspace::Axis;

use super::start_policy::starts_when_shown;
use super::view::Workspace;

impl Workspace {
    /// Shows the session in the focused pane, or in the tab it already
    /// has, and starts its program unless the app has it.
    pub fn show_session(&mut self, session_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.notice = None;
        self.layout.open(session_id);
        self.start_if_stopped(session_id, cx);
        self.changed(window, cx);
    }

    pub(super) fn start_if_stopped(&mut self, session_id: &str, cx: &mut Context<Self>) {
        self.live.update(cx, |live, cx| {
            if starts_when_shown(live.host.phase(session_id)) {
                live.open(session_id, cx);
            }
        });
    }

    /// Starts the program again: for a session that ended or could not start.
    pub(super) fn reopen(&mut self, session_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.live.update(cx, |live, cx| live.open(session_id, cx));
        self.changed(window, cx);
    }

    /// A new shell session in the category of what the pane shows, or in
    /// the first one, opened in that pane.
    pub(super) fn new_terminal(&mut self, pane: u32, window: &mut Window, cx: &mut Context<Self>) {
        let category_id = {
            let live = self.live.read(cx);
            let on_show = self
                .layout
                .pane(pane)
                .and_then(|pane| pane.active_session());
            on_show
                .and_then(|id| live.session(id))
                .map(|session| session.category_id.clone())
                .or_else(|| {
                    live.categories()
                        .first()
                        .map(|category| category.id.clone())
                })
        };
        let Some(category_id) = category_id else {
            self.notice = Some(self.texts.get("workspace.no_category"));
            cx.notify();
            return;
        };
        let created = self.live.update(cx, |live, cx| {
            live.perform(|actions| actions.create_terminal(&category_id), cx)
        });
        if let Some((_, session)) = created {
            self.layout.focus_pane(pane);
            self.show_session(&session.id, window, cx);
        }
    }

    pub(super) fn split(
        &mut self,
        pane: u32,
        axis: Axis,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let split = self.layout.split(pane, axis);
        self.notice = split.err().map(|error| self.texts.say(&(&error).into()));
        self.changed(window, cx);
    }

    /// Closes the tab and ends the session's program.
    pub fn close_tab(&mut self, session_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.live.update(cx, |live, cx| live.close(session_id, cx));
        self.layout.close_tab(session_id);
        self.changed(window, cx);
    }

    pub(super) fn close_pane(&mut self, pane: u32, window: &mut Window, cx: &mut Context<Self>) {
        let sessions = self.layout.close_pane(pane);
        self.live.update(cx, |live, cx| {
            for session_id in &sessions {
                live.close(session_id, cx);
            }
        });
        self.changed(window, cx);
    }

    pub(super) fn focus_pane(&mut self, pane: u32, window: &mut Window, cx: &mut Context<Self>) {
        if self.layout.active_pane().id != pane {
            self.layout.focus_pane(pane);
            self.changed(window, cx);
        }
    }

    pub(super) fn select_tab(
        &mut self,
        session_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        #[cfg(all(feature = "webview-spike", target_os = "macos"))]
        if let Some(pane) = self.layout.pane_of(session_id) {
            self.browsers.choose(pane, false);
        }
        self.layout.select_tab(session_id);
        self.changed(window, cx);
    }

    pub(super) fn move_tab(
        &mut self,
        session_id: &str,
        to: u32,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.layout.move_tab(session_id, to, index);
        self.changed(window, cx);
    }
}
