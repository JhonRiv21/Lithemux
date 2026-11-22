//! Temporary, for the §2.1 block: the window over the sessions asked for
//! with `--open`. The first two that run are shown side by side; the rest
//! are listed at the bottom. The sidebar of the next block replaces it.

use gpui::{
    App, AppContext, Bounds, Context, Div, Entity, Focusable, IntoElement, KeyBinding, Render,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*,
    px, rgb, size,
};
use lithemux_app::terminal_view::TerminalView;

use crate::hosted_sessions::{Asked, HostedSessions};

const VISIBLE_PANES: usize = 2;
const BORDER: u32 = 0x2a2a2e;
const FOCUSED_BORDER: u32 = 0x4d78cc;

actions!(hosted, [FocusNextPane, FocusPreviousPane, Quit]);

struct HostedWindow {
    sessions: Entity<HostedSessions>,
    /// The terminals on screen, by session id, in the order they started.
    views: Vec<(String, Entity<TerminalView>)>,
}

impl HostedWindow {
    fn new(sessions: Entity<HostedSessions>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.observe_in(&sessions, window, |this, _, window, cx| {
            this.show_what_started(window, cx);
            cx.notify();
        })
        .detach();
        Self {
            sessions,
            views: Vec::new(),
        }
    }

    /// Gives a pane to the sessions that got their terminal, while there is
    /// room.
    fn show_what_started(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let started: Vec<_> = {
            let sessions = self.sessions.read(cx);
            sessions
                .asked
                .iter()
                .filter_map(|asked| asked.session.as_ref().ok())
                .filter(|id| !self.views.iter().any(|(shown, _)| shown == *id))
                .filter_map(|id| Some((id.clone(), sessions.host.terminal(id)?)))
                .collect()
        };
        for (session_id, (terminal, wakes)) in started {
            if self.views.len() == VISIBLE_PANES {
                return;
            }
            let view = cx.new(|cx| TerminalView::new(terminal, wakes, window, cx));
            if self.views.is_empty() {
                window.focus(&view.focus_handle(cx), cx);
            }
            self.views.push((session_id, view));
        }
    }

    fn view_of(&self, asked: &Asked) -> Option<&Entity<TerminalView>> {
        let session_id = asked.session.as_ref().ok()?;
        self.views
            .iter()
            .find(|(shown, _)| shown == session_id)
            .map(|(_, view)| view)
    }

    fn focus_pane(&self, step: usize, window: &mut Window, cx: &mut Context<Self>) {
        let current = self
            .views
            .iter()
            .position(|(_, view)| view.focus_handle(cx).contains_focused(window, cx))
            .unwrap_or(0);
        let count = self.views.len().max(1);
        if let Some((_, view)) = self.views.get((current + step) % count) {
            window.focus(&view.focus_handle(cx), cx);
        }
    }

    /// A pane: its status line and, once the session runs, its terminal.
    fn pane(&self, asked: &Asked, window: &Window, cx: &App) -> Div {
        let view = self.view_of(asked);
        let focused = view.is_some_and(|view| view.focus_handle(cx).contains_focused(window, cx));
        let pane = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .h_full()
            .border_1()
            .border_color(rgb(if focused { FOCUSED_BORDER } else { BORDER }))
            .child(label(self.sessions.read(cx).status(asked)));
        match view {
            Some(view) => pane.child(div().flex_1().min_h_0().p_1().child(view.clone())),
            None => pane,
        }
    }
}

fn label(text: String) -> Div {
    div()
        .px_2()
        .py_1()
        .text_sm()
        .text_color(rgb(0x9a9aa2))
        .child(SharedString::from(text))
}

impl Render for HostedWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sessions = self.sessions.read(cx);
        // The sessions with a terminal come first; while there is room, the
        // others get a pane too, with only their status in it.
        let (with_a_view, without): (Vec<&Asked>, Vec<&Asked>) = sessions
            .asked
            .iter()
            .partition(|asked| self.view_of(asked).is_some());
        let ordered: Vec<&Asked> = with_a_view.into_iter().chain(without).collect();
        let (shown, listed) = ordered.split_at(ordered.len().min(VISIBLE_PANES));
        let panes: Vec<Div> = shown
            .iter()
            .map(|asked| self.pane(asked, window, cx))
            .collect();
        let without_a_pane = listed
            .iter()
            .map(|asked| sessions.status(asked))
            .collect::<Vec<_>>()
            .join("   |   ");
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x161618))
            .on_action(
                cx.listener(|this, _: &FocusNextPane, window, cx| this.focus_pane(1, window, cx)),
            )
            .on_action(cx.listener(|this, _: &FocusPreviousPane, window, cx| {
                this.focus_pane(VISIBLE_PANES - 1, window, cx)
            }))
            .child(div().flex().flex_1().min_h_0().children(panes))
            .when(!without_a_pane.is_empty(), |root| {
                root.child(label(format!("without a pane: {without_a_pane}")))
            })
    }
}

fn window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1280.0), px(720.0)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("lithemux".into()),
            ..TitlebarOptions::default()
        }),
        ..WindowOptions::default()
    }
}

pub fn open(sessions: Entity<HostedSessions>, cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-alt-right", FocusNextPane, None),
        KeyBinding::new("cmd-alt-left", FocusPreviousPane, None),
        KeyBinding::new("cmd-q", Quit, None),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_window_closed(|cx, _| cx.quit()).detach();
    let opened = cx.open_window(window_options(cx), |window, cx| {
        cx.new(|cx| HostedWindow::new(sessions, window, cx))
    });
    if opened.is_err() {
        eprintln!("lithemux-app: the window could not be opened");
        cx.quit();
        return;
    }
    cx.activate(true);
}
