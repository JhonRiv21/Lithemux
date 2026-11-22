//! The one window of the app: the sidebar and, next to it, the workspace.

use gpui::{
    App, AppContext, Bounds, Context, Entity, IntoElement, KeyBinding, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};

use super::live::Live;
use super::quit_signals::quit_on_signals;
use crate::sidebar::Sidebar;
use crate::theme;
use crate::workspace::{Workspace, bind_keys};

actions!(lithemux, [Quit]);

pub struct MainWindow {
    pub sidebar: Entity<Sidebar>,
    pub workspace: Entity<Workspace>,
}

impl MainWindow {
    pub fn new(live: Entity<Live>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace = cx.new(|cx| Workspace::new(live.clone(), window, cx));
        let sidebar = cx.new(|cx| Sidebar::new(live, workspace.clone(), cx));
        workspace.update(cx, |workspace, cx| workspace.arrange(window, cx));
        MainWindow { sidebar, workspace }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .bg(rgb(theme::WINDOW))
            .child(self.sidebar.clone())
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.workspace.clone()),
            )
    }
}

fn window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1360.0), px(820.0)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("lithemux".into()),
            ..TitlebarOptions::default()
        }),
        ..WindowOptions::default()
    }
}

/// The keys of the app, for any window.
pub fn bind_app_keys(cx: &mut App) {
    bind_keys(cx);
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
}

pub fn open(live: Entity<Live>, cx: &mut App) {
    bind_app_keys(cx);
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_window_closed(|cx, _| cx.quit()).detach();
    let weak = live.downgrade();
    quit_on_signals(cx, move |cx| {
        let _ = weak.update(cx, |live, _| live.host.shut_down());
    });
    if cx.text_system().all_font_names().is_empty() {
        eprintln!("lithemux: this build has no fonts, so no text will be drawn");
    }
    let opened = cx.open_window(window_options(cx), |window, cx| {
        cx.new(|cx| MainWindow::new(live, window, cx))
    });
    if opened.is_err() {
        eprintln!("lithemux-app: the window could not be opened");
        cx.quit();
        return;
    }
    cx.activate(true);
}
