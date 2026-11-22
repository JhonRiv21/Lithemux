//! The native menu bar of macOS (PLAN §2.6): the app's own menu, then
//! File, Edit, View, Window and Help. Every entry is an action that the
//! keyboard can ask for too.

use gpui::{App, KeyBinding, Menu, MenuItem, OsAction, actions};

use crate::edit_actions::{Copy, Paste};
use crate::i18n::Texts;
use crate::workspace::shortcuts::{
    ClosePane, CloseTab, NewTerminal, NextPane, NextTab, PreviousPane, PreviousTab, SplitDown,
    SplitRight,
};

actions!(
    lithemux,
    [Quit, OpenSettings, About, OpenDataFolder, Minimize, Zoom]
);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-m", Minimize, None),
    ]);
}

/// Sets the menus, in the language of `texts`. Called again when the
/// language changes.
pub fn set_menus(cx: &mut App, texts: &Texts) {
    let text = |key: &str| texts.get(&format!("menu_bar.{key}"));
    cx.set_menus(vec![
        // The first one is the app's own menu, whatever its name.
        Menu::new("lithemux").items(app_items(&text)),
        Menu::new(text("file")).items(vec![
            MenuItem::action(text("new_terminal"), NewTerminal),
            MenuItem::separator(),
            MenuItem::action(text("close_tab"), CloseTab),
            MenuItem::action(text("close_pane"), ClosePane),
        ]),
        Menu::new(text("edit")).items(vec![
            MenuItem::os_action(text("copy"), Copy, OsAction::Copy),
            MenuItem::os_action(text("paste"), Paste, OsAction::Paste),
        ]),
        Menu::new(text("view")).items(view_items(&text)),
        Menu::new(text("window")).items(vec![
            MenuItem::action(text("minimize"), Minimize),
            MenuItem::action(text("zoom"), Zoom),
        ]),
        Menu::new(text("help")).items(vec![MenuItem::action(
            text("open_data_folder"),
            OpenDataFolder,
        )]),
    ]);
}

fn app_items(text: &dyn Fn(&str) -> String) -> Vec<MenuItem> {
    vec![
        MenuItem::action(text("about"), About),
        MenuItem::separator(),
        MenuItem::action(text("settings"), OpenSettings),
        MenuItem::separator(),
        MenuItem::action(text("quit"), Quit),
    ]
}

fn view_items(text: &dyn Fn(&str) -> String) -> Vec<MenuItem> {
    vec![
        MenuItem::action(text("split_right"), SplitRight),
        MenuItem::action(text("split_down"), SplitDown),
        MenuItem::separator(),
        MenuItem::action(text("next_tab"), NextTab),
        MenuItem::action(text("previous_tab"), PreviousTab),
        MenuItem::action(text("next_pane"), NextPane),
        MenuItem::action(text("previous_pane"), PreviousPane),
    ]
}
