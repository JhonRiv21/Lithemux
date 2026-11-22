//! The small forms of the sidebar: a name for something new, or a new
//! name for something that exists.

use std::path::PathBuf;

use gpui::Entity;
use lithemux_core::modules::session::Profile;

use crate::text_field::TextField;

pub(super) enum Form {
    NewCategory {
        name: Entity<TextField>,
        folder: Option<PathBuf>,
    },
    NewSession {
        category_id: String,
        profile: Profile,
        name: Entity<TextField>,
    },
    RenameSession {
        session_id: String,
        title: Entity<TextField>,
    },
    RenameCategory {
        category_id: String,
        name: Entity<TextField>,
    },
}

impl Form {
    pub fn field(&self) -> &Entity<TextField> {
        match self {
            Form::NewCategory { name, .. }
            | Form::NewSession { name, .. }
            | Form::RenameCategory { name, .. } => name,
            Form::RenameSession { title, .. } => title,
        }
    }

    pub fn is_for_new_session_in(&self, category: &str) -> bool {
        matches!(self, Form::NewSession { category_id, .. } if category_id == category)
    }

    pub fn renames_session(&self, session: &str) -> bool {
        matches!(self, Form::RenameSession { session_id, .. } if session_id == session)
    }

    pub fn renames_category(&self, category: &str) -> bool {
        matches!(self, Form::RenameCategory { category_id, .. } if category_id == category)
    }
}
