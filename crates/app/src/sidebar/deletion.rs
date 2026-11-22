//! Deleting a session or a category: asked first, and their programs are
//! closed before they go.

use gpui::{Context, PromptLevel, Window};

use super::tree::Tree;
use super::view::Sidebar;

/// The sessions of a category, pinned or not.
fn sessions_of(tree: &Tree, category_id: &str) -> Vec<String> {
    let groups = tree
        .groups
        .iter()
        .filter(|group| group.category_id == category_id);
    let in_groups = groups.flat_map(|group| group.header.iter().chain(&group.cards));
    let pinned = tree
        .pinned
        .iter()
        .filter(|card| card.category_id == category_id);
    in_groups
        .chain(pinned)
        .map(|card| card.session_id.clone())
        .collect()
}

impl Sidebar {
    pub(super) fn confirm_delete_session(
        &mut self,
        id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(name) = self
            .live
            .read(cx)
            .session(&id)
            .map(|s| s.display_title().to_string())
        else {
            return;
        };
        let (delete, cancel) = (
            self.texts.get("confirm.delete"),
            self.texts.get("confirm.cancel"),
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            &self
                .texts
                .with("confirm.delete_session", &[("name", &name)]),
            Some(&self.texts.get("confirm.delete_session_detail")),
            &[delete.as_str(), cancel.as_str()],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if answer.await != Ok(0) {
                return;
            }
            let _ = this.update_in(cx, |this, window, cx| {
                this.close_session(&id, window, cx);
                this.change(cx, |actions| actions.delete_session(&id));
            });
        })
        .detach();
    }

    pub(super) fn confirm_delete_category(
        &mut self,
        id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let live = self.live.read(cx);
        let Some(name) = live
            .categories()
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone())
        else {
            return;
        };
        let count = sessions_of(&live.tree(), &id).len();
        let (delete, cancel) = (
            self.texts.get("confirm.delete"),
            self.texts.get("confirm.cancel"),
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            &self.texts.with(
                "confirm.delete_category",
                &[("name", &name), ("count", &count.to_string())],
            ),
            Some(&self.texts.get("confirm.delete_category_detail")),
            &[delete.as_str(), cancel.as_str()],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if answer.await != Ok(0) {
                return;
            }
            let _ = this.update_in(cx, |this, window, cx| {
                let sessions = sessions_of(&this.live.read(cx).tree(), &id);
                for session in &sessions {
                    this.close_session(session, window, cx);
                }
                this.change(cx, |actions| actions.delete_category(&id));
            });
        })
        .detach();
    }
}
