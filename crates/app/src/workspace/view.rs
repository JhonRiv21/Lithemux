use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Task, Window, div,
    prelude::*, rgb,
};
use lithemux_core::modules::workspace::{
    Layout, SqliteLayoutRepository, restore_layout, save_layout,
};

use super::read_policy::sessions_being_read;
use super::shortcuts;
use crate::app_shell::live::Live;
use crate::i18n::Texts;
use crate::terminal_view::{TerminalSession, TerminalView};
use crate::theme;

/// How long a divider must rest before its place is saved.
const SAVE_AFTER: Duration = Duration::from_millis(400);

/// A session's terminal on screen.
pub(super) struct Shown {
    terminal: Rc<TerminalSession>,
    pub view: Entity<TerminalView>,
}

pub struct Workspace {
    pub(super) live: Entity<Live>,
    pub(super) layout: Layout,
    pub(super) shown: HashMap<String, Shown>,
    pub(super) texts: Rc<Texts>,
    pub(super) notice: Option<String>,
    pub(super) focus_handle: FocusHandle,
    /// The save of a divider that is still being dragged. Replacing it
    /// drops the one before: only the last move is saved.
    pending_save: Option<Task<()>>,
    _lifecycle: Task<()>,
}

impl Workspace {
    pub fn new(live: Entity<Live>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (layout, texts) = {
            let live = live.read(cx);
            let layouts = SqliteLayoutRepository::new(live.db());
            let restored = restore_layout(&layouts, |id| live.session_exists(id));
            (
                restored.unwrap_or_else(|error| {
                    eprintln!("lithemux-app: {error}");
                    Layout::single()
                }),
                Rc::clone(&live.texts),
            )
        };
        cx.observe_in(&live, window, |this, live, window, cx| {
            // The language may have changed.
            this.texts = Rc::clone(&live.read(cx).texts);
            this.follow_the_sessions(window, cx);
            cx.notify();
        })
        .detach();
        // Coming back to the window is looking at what it shows.
        cx.observe_window_activation(window, |this, window, cx| {
            this.read_what_is_seen(window, cx);
        })
        .detach();
        // The window may go before a divider's save is due.
        cx.on_app_quit(|this, cx| {
            this.save(cx);
            async {}
        })
        .detach();
        Workspace {
            live,
            layout,
            shown: HashMap::new(),
            texts,
            notice: None,
            focus_handle: cx.focus_handle(),
            pending_save: None,
            _lifecycle: Self::keep_the_lifecycle(cx),
        }
    }

    /// Brings the workspace up to date with the sessions: drops the tabs
    /// of the deleted ones, gives each tab the terminal the host has for
    /// it now, and counts what the user is looking at as read.
    fn follow_the_sessions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let before = self.layout.clone();
        {
            let live = self.live.read(cx);
            self.layout.keep_sessions(|id| live.session_exists(id));
        }
        if self.layout != before {
            self.save(cx);
        }
        // What is on show may have just been hibernated, or restored.
        self.wake_what_is_seen(cx);
        if self.adopt_the_terminals(window, cx) {
            self.focus_what_is_shown(window, cx);
        }
        self.read_what_is_seen(window, cx);
    }

    /// Returns whether a terminal that was not on screen now is.
    fn adopt_the_terminals(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let terminals: Vec<_> = {
            let live = self.live.read(cx);
            let panes = self.layout.panes_in_order();
            let tabs = panes.iter().flat_map(|pane| &pane.tabs);
            tabs.filter_map(|id| Some((id.clone(), live.host.terminal(id)?)))
                .collect()
        };
        self.shown.retain(|id, _| {
            terminals
                .iter()
                .any(|(with_terminal, _)| with_terminal == id)
        });
        let mut started = false;
        for (id, (terminal, wakes)) in terminals {
            let is_shown = self.shown.get(&id);
            if is_shown.is_some_and(|shown| Rc::ptr_eq(&shown.terminal, &terminal)) {
                continue;
            }
            let view = cx.new(|cx| TerminalView::new(Rc::clone(&terminal), wakes, window, cx));
            self.shown.insert(id, Shown { terminal, view });
            started = true;
        }
        started
    }

    fn read_what_is_seen(&mut self, window: &Window, cx: &mut Context<Self>) {
        let on_show = self.layout.visible_sessions();
        let seen: Vec<String> = sessions_being_read(&on_show, window.is_window_active())
            .into_iter()
            .map(str::to_string)
            .collect();
        self.live.update(cx, |live, cx| {
            for id in &seen {
                live.mark_read(id, cx);
            }
            live.set_in_view(seen);
        });
    }

    fn save(&mut self, cx: &App) {
        self.pending_save = None;
        let layouts = SqliteLayoutRepository::new(self.live.read(cx).db());
        if let Err(error) = save_layout(&layouts, &self.layout) {
            eprintln!("lithemux-app: {error}");
        }
    }

    /// Saves once the layout has stopped changing for a moment: a divider
    /// being dragged moves many times a second.
    pub(super) fn save_soon(&mut self, cx: &mut Context<Self>) {
        self.pending_save = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_AFTER).await;
            let _ = this.update(cx, |this, cx| this.save(cx));
        }));
    }

    /// Once the window is up, the restored layout comes back to life (PLAN
    /// §2.5): the Claude sessions on show are resumed, the rest stay
    /// hibernated, and the shells start again.
    pub fn arrange(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.wake_what_is_seen(cx);
        self.relaunch_the_shells(cx);
        self.follow_the_sessions(window, cx);
        self.focus_what_is_shown(window, cx);
    }

    /// After any change of tabs or panes.
    pub(super) fn changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.save(cx);
        self.wake_what_is_seen(cx);
        self.follow_the_sessions(window, cx);
        self.focus_what_is_shown(window, cx);
        cx.notify();
    }

    /// The keyboard goes to the terminal on show in the focused pane, or
    /// to the workspace itself, so that its shortcuts keep working.
    pub fn focus_what_is_shown(&self, window: &mut Window, cx: &mut Context<Self>) {
        let on_show = self.layout.active_pane().active_session();
        match on_show.and_then(|id| self.shown.get(id)) {
            Some(shown) => window.focus(&shown.view.focus_handle(cx), cx),
            None => window.focus(&self.focus_handle, cx),
        }
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Whether the session's terminal is on screen, in a tab.
    pub fn has_terminal_of(&self, session_id: &str) -> bool {
        self.shown.contains_key(session_id)
    }

    /// Whether the session has a tab, in any pane.
    pub fn has_tab(&self, session_id: &str) -> bool {
        self.layout.pane_of(session_id).is_some()
    }

    /// The session on show in the focused pane.
    pub fn session_in_focus(&self) -> Option<&str> {
        self.layout.active_pane().active_session()
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tree = self.render_node(&self.layout.root().clone(), window, cx);
        let notice = self.notice.clone();
        let root = div()
            .key_context(shortcuts::CONTEXT)
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(theme::colors().window));
        shortcuts::handle(root, cx)
            .child(div().flex_1().min_h_0().child(tree))
            .when_some(notice, |root, notice| {
                root.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_sm()
                        .text_color(rgb(theme::colors().attention))
                        .child(notice),
                )
            })
    }
}
