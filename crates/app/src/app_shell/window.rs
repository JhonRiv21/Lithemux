//! The windows of the app: the main one, with the sidebar and the
//! workspace, and the settings. And what the menu bar asks of them.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{
    App, AppContext, Bounds, Context, Entity, IntoElement, PromptLevel, Render, TitlebarOptions,
    Window, WindowBounds, WindowHandle, WindowOptions, div, point, prelude::*, px, rgb, size,
};
use lithemux_core::platform::process::local_page::open_folder;

use super::live::Live;
use super::menus::{
    About, Minimize, OpenDataFolder, OpenSettings, Quit, Zoom, bind_keys as bind_menu_keys,
    set_menus,
};
use super::quit_signals::quit_on_signals;
use super::title_bar::title_bar;
use crate::edit_actions;
use crate::settings_window::SettingsWindow;
use crate::sidebar::Sidebar;
use crate::status_bar::StatusBar;
use crate::theme;
use crate::workspace::{Workspace, bind_keys};

pub struct MainWindow {
    live: Entity<Live>,
    pub sidebar: Entity<Sidebar>,
    pub workspace: Entity<Workspace>,
    status_bar: Entity<StatusBar>,
}

impl MainWindow {
    pub fn new(live: Entity<Live>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace = cx.new(|cx| Workspace::new(live.clone(), window, cx));
        let sidebar = cx.new(|cx| Sidebar::new(live.clone(), workspace.clone(), cx));
        let status_bar = cx.new(|cx| StatusBar::new(live.clone(), cx));
        workspace.update(cx, |workspace, cx| workspace.arrange(window, cx));
        // The theme may be «system»: it follows macOS when that changes.
        cx.observe_window_appearance(window, |this, _, cx| {
            this.live.update(cx, |live, cx| {
                live.reload(cx);
                cx.notify();
            });
        })
        .detach();
        // A change of theme repaints the frame of the window too.
        cx.observe(&live, |_, _, cx| cx.notify()).detach();
        MainWindow {
            live,
            sidebar,
            workspace,
            status_bar,
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(theme::colors().window))
            .child(title_bar("lithemux"))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.sidebar.clone())
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .child(self.workspace.clone()),
                    ),
            )
            .child(self.status_bar.clone())
    }
}

/// Without the system's title bar: the window paints its own, in the
/// colors of the theme.
fn window_options(width: f32, height: f32, cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(width), px(height)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("lithemux".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(12.0), px(9.0))),
        }),
        ..WindowOptions::default()
    }
}

/// The keys of the app, for any window.
pub fn bind_app_keys(cx: &mut App) {
    bind_keys(cx);
    bind_menu_keys(cx);
    edit_actions::bind_keys(cx);
}

/// The one settings window: shown, or brought to the front if it is open.
fn show_settings(
    live: &Entity<Live>,
    open: &Rc<RefCell<Option<WindowHandle<SettingsWindow>>>>,
    cx: &mut App,
) {
    let existing = *open.borrow();
    if let Some(handle) = existing
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        return;
    }
    let live = live.clone();
    let opened = cx.open_window(window_options(680.0, 620.0, cx), |window, cx| {
        cx.new(|cx| SettingsWindow::new(live, window, cx))
    });
    *open.borrow_mut() = opened.ok();
}

fn on_the_active_window(cx: &mut App, change: impl FnOnce(&mut Window)) {
    if let Some(active) = cx.active_window() {
        let _ = active.update(cx, |_, window, _| change(window));
    }
}

fn show_about(live: &Entity<Live>, cx: &mut App) {
    let texts = Rc::clone(&live.read(cx).texts);
    let detail = texts.with(
        "menu_bar.about_detail",
        &[("version", env!("CARGO_PKG_VERSION"))],
    );
    let ok = texts.get("menu_bar.ok");
    on_the_active_window_with_cx(cx, |window, cx| {
        // The answer does not matter: there is one button.
        drop(window.prompt(
            PromptLevel::Info,
            "lithemux",
            Some(&detail),
            &[ok.as_str()],
            cx,
        ));
    });
}

fn on_the_active_window_with_cx(cx: &mut App, change: impl FnOnce(&mut Window, &mut App)) {
    if let Some(active) = cx.active_window() {
        let _ = active.update(cx, |_, window, cx| change(window, cx));
    }
}

/// What the menu bar and its keys ask for, whatever window has the focus.
fn handle_the_menu_bar(live: &Entity<Live>, main: WindowHandle<MainWindow>, cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &Minimize, cx| on_the_active_window(cx, |window| window.minimize_window()));
    cx.on_action(|_: &Zoom, cx| on_the_active_window(cx, |window| window.zoom_window()));
    let settings = Rc::new(RefCell::new(None));
    cx.on_action({
        let live = live.clone();
        move |_: &OpenSettings, cx| show_settings(&live, &settings, cx)
    });
    cx.on_action({
        let live = live.clone();
        move |_: &About, cx| show_about(&live, cx)
    });
    cx.on_action({
        let live = live.clone();
        move |_: &OpenDataFolder, cx| {
            let folder = live.read(cx).home_folder();
            if let Err(error) = open_folder(&folder) {
                eprintln!("lithemux-app: {error}");
            }
        }
    });
    // Closing the main window ends the app; closing the settings does not.
    cx.on_window_closed(move |cx, closed| {
        if closed == main.window_id() {
            cx.quit();
        }
    })
    .detach();
}

/// The menus are set in the language of the interface, and again whenever
/// it changes.
fn keep_the_menus_in_the_language(live: &Entity<Live>, cx: &mut App) {
    let texts = Rc::clone(&live.read(cx).texts);
    set_menus(cx, &texts);
    let shown = Cell::new(texts.language());
    cx.observe(live, move |live, cx| {
        let texts = Rc::clone(&live.read(cx).texts);
        if shown.replace(texts.language()) != texts.language() {
            set_menus(cx, &texts);
        }
    })
    .detach();
}

pub fn open(live: Entity<Live>, cx: &mut App) {
    bind_app_keys(cx);
    let weak = live.downgrade();
    quit_on_signals(cx, move |cx| {
        let _ = weak.update(cx, |live, _| live.host.shut_down());
    });
    if cx.text_system().all_font_names().is_empty() {
        eprintln!("lithemux: this build has no fonts, so no text will be drawn");
    }
    let opened = cx.open_window(window_options(1360.0, 820.0, cx), {
        let live = live.clone();
        |window, cx| cx.new(|cx| MainWindow::new(live, window, cx))
    });
    let Ok(main) = opened else {
        eprintln!("lithemux-app: the window could not be opened");
        cx.quit();
        return;
    };
    handle_the_menu_bar(&live, main, cx);
    keep_the_menus_in_the_language(&live, cx);
    cx.activate(true);
}
