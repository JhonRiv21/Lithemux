//! The sidebar as a gpui view: paints the tree and turns clicks, menus,
//! forms and drags into commands.

use std::rc::Rc;

use gpui::{
    Context, Entity, FocusHandle, IntoElement, Pixels, Point, Render, ScrollHandle, Window, div,
    point, prelude::*, px, rgb,
};
use lithemux_core::platform::process::local_page::open_local_page;

use super::form::Form;
use super::menu::{MenuCommand, MenuItem, card_menu};
use crate::app_shell::live::Live;
use crate::i18n::Texts;
use crate::theme;
use crate::workspace::Workspace;

pub(super) const WIDTH: f32 = 280.0;

/// A menu that is open, and where.
pub(super) struct OpenMenu {
    pub items: Vec<MenuItem>,
    pub at: Point<Pixels>,
    /// The entry the keyboard is on.
    pub highlighted: Option<usize>,
}

/// Where the menu of the session in view opens when the keyboard asks for
/// it: at the top of the sidebar, under the title bar.
const KEYBOARD_MENU_AT: (f32, f32) = (24.0, 64.0);

pub struct Sidebar {
    pub(super) live: Entity<Live>,
    pub(super) workspace: Entity<Workspace>,
    pub(super) texts: Rc<Texts>,
    pub(super) menu: Option<OpenMenu>,
    /// The menu has the keyboard while it is open: Esc closes it.
    pub(super) menu_focus: FocusHandle,
    pub(super) form: Option<Form>,
    /// The list's scroll, to bring the session in focus into view.
    pub(super) scroll: ScrollHandle,
    /// The session the list last scrolled to.
    revealed: Option<String>,
}

impl Sidebar {
    pub fn new(live: Entity<Live>, workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        cx.observe(&live, |this, live, cx| {
            // The language may have changed.
            this.texts = Rc::clone(&live.read(cx).texts);
            cx.notify();
        })
        .detach();
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        let texts = Rc::clone(&live.read(cx).texts);
        Sidebar {
            live,
            workspace,
            texts,
            menu: None,
            menu_focus: cx.focus_handle(),
            form: None,
            scroll: ScrollHandle::new(),
            revealed: None,
        }
    }

    pub(super) fn open_menu(
        &mut self,
        items: Vec<MenuItem>,
        at: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.menu = Some(OpenMenu {
            items,
            at,
            highlighted: None,
        });
        window.focus(&self.menu_focus, cx);
        cx.notify();
    }

    /// The menu of a session, asked for from the keyboard.
    pub fn open_session_menu(
        &mut self,
        session_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tree = self.live.read(cx).tree();
        let headers = tree.groups.iter().filter_map(|group| group.header.as_ref());
        let cards = tree.groups.iter().flat_map(|group| group.cards.iter());
        let Some(card) = tree
            .pinned
            .iter()
            .chain(headers)
            .chain(cards)
            .find(|card| card.session_id == session_id)
        else {
            return;
        };
        let (x, y) = KEYBOARD_MENU_AT;
        self.open_menu(card_menu(card), point(px(x), px(y)), window, cx);
    }

    pub fn has_menu_open(&self) -> bool {
        self.menu.is_some()
    }

    /// Closes the menu and gives the keyboard back to what is on show.
    pub(super) fn close_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.menu = None;
        self.give_back_the_keyboard(window, cx);
        cx.notify();
    }

    fn give_back_the_keyboard(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |workspace, cx| {
            workspace.focus_what_is_shown(window, cx)
        });
    }

    pub(super) fn open_port(&mut self, port: u16, cx: &mut Context<Self>) {
        if let Err(error) = open_local_page(port) {
            self.live.update(cx, |live, cx| {
                let said = live
                    .texts
                    .with("error.internal", &[("detail", &error.to_string())]);
                live.notice = Some(said);
                cx.notify();
            });
        }
    }

    /// What a click on a card, a menu entry or a button asks for.
    pub fn run(&mut self, command: MenuCommand, window: &mut Window, cx: &mut Context<Self>) {
        self.menu = None;
        match command {
            MenuCommand::Open(id) => {
                self.workspace
                    .update(cx, |workspace, cx| workspace.show_session(&id, window, cx));
            }
            MenuCommand::Close(id) => self.close_session(&id, window, cx),
            MenuCommand::RenameSession(_)
            | MenuCommand::RenameCategory(_)
            | MenuCommand::NewSession(..) => self.start_form(command, window, cx),
            MenuCommand::SetPinned(id, pinned) => {
                self.change(cx, |actions| actions.set_pinned(&id, pinned));
            }
            MenuCommand::SetCoordinator(id, coordinator) => {
                self.change(cx, |actions| actions.set_coordinator(&id, coordinator));
            }
            MenuCommand::SetKeepAlive(id, keep) => {
                self.change(cx, |actions| actions.set_keep_alive(&id, keep));
            }
            MenuCommand::ShiftSession(id, by) => {
                self.change(cx, |actions| actions.shift_session(&id, by));
            }
            MenuCommand::ShiftCategory(id, by) => {
                self.change(cx, |actions| actions.shift_category(&id, by));
            }
            MenuCommand::SetNotify(id, notify) => {
                self.change(cx, |actions| actions.set_notify(&id, notify));
            }
            MenuCommand::SetCollapsed(id, collapsed) => {
                self.change(cx, |actions| actions.set_collapsed(&id, collapsed));
            }
            MenuCommand::DeleteSession(id) => self.confirm_delete_session(id, window, cx),
            MenuCommand::DeleteCategory(id) => self.confirm_delete_category(id, window, cx),
        }
        // A form keeps the keyboard for its field.
        if self.form.is_none() && self.menu_focus.is_focused(window) {
            self.give_back_the_keyboard(window, cx);
        }
        cx.notify();
    }

    /// A change of what is stored; what goes wrong shows as the notice.
    pub(super) fn change<T>(
        &mut self,
        cx: &mut Context<Self>,
        change: impl FnOnce(
            &super::actions::SidebarActions<'_>,
        ) -> Result<T, super::error::SidebarError>,
    ) -> Option<T> {
        self.live.update(cx, |live, cx| live.perform(change, cx))
    }

    pub(super) fn close_session(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.workspace.read(cx).has_tab(id) {
            self.workspace
                .update(cx, |workspace, cx| workspace.close_tab(id, window, cx));
        } else {
            self.live.update(cx, |live, cx| live.close(id, cx));
        }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (tree, notice) = {
            let live = self.live.read(cx);
            (live.tree(), live.notice.clone())
        };
        let in_focus = self
            .workspace
            .read(cx)
            .session_in_focus()
            .map(str::to_string);
        let (list, at) = self.render_list(&tree, in_focus.as_deref(), cx);
        // A session that comes into focus, by the keyboard, the palette or
        // a click, is scrolled into view once.
        if in_focus != self.revealed {
            if let Some(at) = at {
                self.scroll.scroll_to_item(at);
            }
            self.revealed = in_focus;
        }
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(WIDTH))
            .h_full()
            .bg(rgb(theme::colors().sidebar))
            .border_r_1()
            .border_color(rgb(theme::colors().border))
            .text_color(rgb(theme::colors().text))
            .child(list)
            .when_some(notice, |sidebar, notice| {
                sidebar.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(rgb(theme::colors().attention))
                        .child(notice),
                )
            })
            .child(self.render_footer(cx))
            .children(self.render_menu(cx))
    }
}
