//! What can go wrong, as messages of the interface. The errors of `core`
//! say it in English for the command line and the logs; the window says it
//! in the user's language, from `locales/*.toml`.

use lithemux_core::modules::category::CategoryError;
use lithemux_core::modules::launch::LaunchError;
use lithemux_core::modules::session::SessionError;
use lithemux_core::modules::workspace::WorkspaceError;

use super::message::Message;
use crate::session_host::{Failure, HostError};
use crate::sidebar::error::SidebarError;

/// For what has no words of its own: the technical detail, said to be one.
fn internal(error: &dyn std::fmt::Display) -> Message {
    Message::with("error.internal", "detail", error.to_string())
}

impl From<&CategoryError> for Message {
    fn from(error: &CategoryError) -> Self {
        match error {
            CategoryError::InvalidSlug(slug) => {
                Message::with("error.invalid_name", "name", &slug.0)
            }
            CategoryError::AlreadyExists(name) => {
                Message::with("error.category_exists", "name", name)
            }
            CategoryError::NotFound(name) => Message::with("error.not_found", "name", name),
            CategoryError::EmptyName => Message::plain("error.empty_name"),
            CategoryError::NotADirectory(path) => {
                Message::with("error.not_a_folder", "path", path.display().to_string())
            }
            other => internal(other),
        }
    }
}

impl From<&SessionError> for Message {
    fn from(error: &SessionError) -> Self {
        match error {
            SessionError::InvalidSlug(slug) => Message::with("error.invalid_name", "name", &slug.0),
            SessionError::InvalidRef(name) => Message::with("error.invalid_name", "name", name),
            SessionError::AlreadyExists(name) => {
                Message::with("error.session_exists", "name", name)
            }
            SessionError::NotFound(name) => Message::with("error.not_found", "name", name),
            SessionError::MessagingNameTaken { name, owner } => {
                Message::with("error.name_taken", "name", name).and("owner", owner)
            }
            SessionError::CoordinatorExists {
                category,
                coordinator,
            } => Message::with("error.coordinator_exists", "category", category)
                .and("coordinator", coordinator),
            SessionError::Category(category) => category.into(),
            other => internal(other),
        }
    }
}

impl From<&SidebarError> for Message {
    fn from(error: &SidebarError) -> Self {
        match error {
            SidebarError::UnusableName(name) => Message::with("error.unusable_name", "name", name),
            SidebarError::Gone => Message::plain("error.gone"),
            SidebarError::Category(category) => category.into(),
            SidebarError::Session(session) => session.into(),
        }
    }
}

impl From<&Failure> for Message {
    fn from(failure: &Failure) -> Self {
        match failure {
            Failure::ClaudeNotFound(detail) => {
                Message::with("error.claude_not_found", "detail", detail)
            }
            Failure::MissingFolder(path) => Message::with("error.missing_folder", "path", path),
            Failure::HookMissing(path) => Message::with("error.hook_missing", "path", path),
            Failure::CouldNotStart(program) => {
                Message::with("error.could_not_start", "program", program)
            }
            Failure::Other(detail) => Message::with("error.internal", "detail", detail),
        }
    }
}

impl From<&HostError> for Message {
    fn from(error: &HostError) -> Self {
        match error {
            HostError::Session(session) => session.into(),
            HostError::Launch(LaunchError::Session(session)) => session.into(),
            HostError::Launch(LaunchError::AlreadyOpen { .. }) => {
                Message::plain("status.open_elsewhere")
            }
            other => (&Failure::from(other)).into(),
        }
    }
}

impl From<&WorkspaceError> for Message {
    fn from(error: &WorkspaceError) -> Self {
        match error {
            WorkspaceError::TooManyPanes => Message::plain("workspace.too_many_panes"),
            other => internal(other),
        }
    }
}
