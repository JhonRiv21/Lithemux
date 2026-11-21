mod application;
mod domain;
mod error;

pub use application::build_session_context::{ContextPorts, build_session_context};
pub use application::resolve_language::resolve_language;
pub use domain::context_input::{ContextInput, MAX_CONTEXT_CHARS};
pub use domain::lang::Lang;
pub use domain::render_context::render_context;
pub use error::ContextError;
