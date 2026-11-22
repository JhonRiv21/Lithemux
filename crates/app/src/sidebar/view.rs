//! The sidebar as a gpui view: paints the tree and turns clicks, menus,
//! forms and drags into commands.

use std::rc::Rc;

use gpui::{Context, Entity, IntoElement, Pixels, Point, Render, Window, div, prelude::*, px, rgb};
use lithemux_core::platform::process::local_page::open_local_page;

use super::form::Form;
use super::menu::{MenuCommand, MenuItem};
use crate::app_shell::live::Live;
use crate::i18n::Texts;
use crate::theme;
use crate::workspace::Workspace;

pub(super) const WIDTH: f32 = 280.0;

/// A menu that is open, and where.
pub(super) struct OpenMenu {
    pub items: Vec<MenuItem>,
    pub at: Point<Pixels>,
}

pub struct Sidebar {
    pub(super) live: Entity<Live>,
    pub(super) workspace: Entity<Workspace>,
    pub(super) texts: Rc<Texts>,
    pub(super) menu: Option<OpenMenu>,
    pub(super) form: Option<Form>,
}

impl Sidebar {
    pub fn new(live: Entity<Live>, workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        cx.observe(&live, |_, _, cx| cx.notify()).detach();
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        let texts = Rc::clone(&live.read(cx).texts);
        Sidebar {
            live,
            workspace,
            texts,
            menu: None,
            form: None,
        }
    }

    pub(super) fn open_menu(
        &mut self,
        items: Vec<MenuItem>,
        at: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.menu = Some(OpenMenu { items, at });
        cx.notify();
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
            MenuCommand::SetMother(id, mother) => {
                self.change(cx, |actions| actions.set_mother(&id, mother));
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
            MenuCommand::SetCollapsed(id, collapsed) => {
                self.change(cx, |actions| actions.set_collapsed(&id, collapsed));
            }
            MenuCommand::DeleteSession(id) => self.confirm_delete_session(id, window, cx),
            MenuCommand::DeleteCategory(id) => self.confirm_delete_category(id, window, cx),
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
        let list = self.render_list(&tree, in_focus.as_deref(), cx);
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(WIDTH))
            .h_full()
            .bg(rgb(theme::SIDEBAR))
            .border_r_1()
            .border_color(rgb(theme::BORDER))
            .text_color(rgb(theme::TEXT))
            .child(list)
            .when_some(notice, |sidebar, notice| {
                sidebar.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(rgb(theme::ATTENTION))
                        .child(notice),
                )
            })
            .child(self.render_footer(cx))
            .children(self.render_menu(cx))
    }
}
