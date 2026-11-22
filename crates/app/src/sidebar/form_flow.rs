//! The forms of the sidebar as they are filled: opened, given a folder,
//! sent or dropped.

use gpui::{Context, Entity, Focusable, PathPromptOptions, Window, prelude::*};

use lithemux_core::modules::session::Profile;

use super::form::Form;
use super::menu::MenuCommand;
use super::view::Sidebar;
use crate::text_field::{TextField, TextFieldEvent};

impl Sidebar {
    pub(super) fn field(
        &mut self,
        text: &str,
        placeholder: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TextField> {
        let placeholder = self.texts.get(placeholder);
        let field = cx.new(|cx| TextField::new(text, placeholder, cx));
        cx.subscribe_in(&field, window, |this, _, event, window, cx| match event {
            TextFieldEvent::Submitted => this.submit_form(window, cx),
            TextFieldEvent::Cancelled => this.cancel_form(window, cx),
        })
        .detach();
        window.focus(&field.focus_handle(cx), cx);
        field
    }

    /// Opens the form a command needs: a new name, or a name for something
    /// new. A new terminal needs none: it is created and opened.
    pub(super) fn start_form(
        &mut self,
        command: MenuCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match command {
            MenuCommand::RenameSession(session_id) => {
                let live = self.live.read(cx);
                let current = live
                    .session(&session_id)
                    .map(|s| s.display_title().to_string());
                let title = self.field(&current.unwrap_or_default(), "form.title", window, cx);
                self.form = Some(Form::RenameSession { session_id, title });
            }
            MenuCommand::RenameCategory(category_id) => {
                let live = self.live.read(cx);
                let current = live.categories().iter().find(|c| c.id == category_id);
                let current = current.map(|category| category.name.clone());
                let name = self.field(
                    &current.unwrap_or_default(),
                    "form.category_name",
                    window,
                    cx,
                );
                self.form = Some(Form::RenameCategory { category_id, name });
            }
            MenuCommand::NewSession(category_id, Profile::Shell) => {
                let created = self.change(cx, |actions| actions.create_terminal(&category_id));
                if let Some((_, session)) = created {
                    self.run(MenuCommand::Open(session.id), window, cx);
                }
            }
            MenuCommand::NewSession(category_id, profile) => {
                let _ = self.change(cx, |actions| actions.set_collapsed(&category_id, false));
                let name = self.field("", "form.session_name", window, cx);
                self.form = Some(Form::NewSession {
                    category_id,
                    profile,
                    name,
                });
            }
            _ => {}
        }
    }

    pub(super) fn start_new_category(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.field("", "form.category_name", window, cx);
        self.form = Some(Form::NewCategory { name, folder: None });
        cx.notify();
    }

    pub(super) fn choose_folder(&mut self, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                if let Some(Form::NewCategory { folder, .. }) = &mut this.form {
                    *folder = paths.into_iter().next();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn cancel_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.form = None;
        self.workspace.update(cx, |workspace, cx| {
            workspace.focus_what_is_shown(window, cx)
        });
        cx.notify();
    }

    pub(super) fn submit_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = self.form.take() else {
            return;
        };
        let typed = form.field().read(cx).text().to_string();
        let done = match &form {
            // A category needs its folder: the form stays until it has one.
            Form::NewCategory { folder: None, .. } => false,
            Form::NewCategory {
                folder: Some(folder),
                ..
            } => self
                .change(cx, |actions| actions.create_category(&typed, folder))
                .is_some(),
            Form::NewSession {
                category_id,
                profile,
                ..
            } => {
                let created = self.change(cx, |actions| {
                    actions.create_session(category_id, &typed, *profile)
                });
                if let Some((_, session)) = &created {
                    self.workspace.update(cx, |workspace, cx| {
                        workspace.show_session(&session.id, window, cx);
                    });
                }
                created.is_some()
            }
            Form::RenameSession { session_id, .. } => self
                .change(cx, |actions| actions.rename_session(session_id, &typed))
                .is_some(),
            Form::RenameCategory { category_id, .. } => self
                .change(cx, |actions| actions.rename_category(category_id, &typed))
                .is_some(),
        };
        if done {
            if !matches!(form, Form::NewSession { .. }) {
                self.workspace.update(cx, |workspace, cx| {
                    workspace.focus_what_is_shown(window, cx)
                });
            }
        } else {
            self.form = Some(form);
        }
        cx.notify();
    }
}
