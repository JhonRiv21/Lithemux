use std::path::Path;

use crate::modules::session::Session;
use crate::platform::paths::home::{HOME_ENV, SESSION_ENV};

/// The hook binary recognizes a lithemux session by `LITHEMUX_SESSION_ID`.
pub fn child_env(session: &Session, lithemux_home: Option<&Path>) -> Vec<(String, String)> {
    let mut env = vec![(SESSION_ENV.to_string(), session.id.clone())];
    if let Some(home) = lithemux_home {
        env.push((HOME_ENV.to_string(), home.display().to_string()));
    }
    env
}
