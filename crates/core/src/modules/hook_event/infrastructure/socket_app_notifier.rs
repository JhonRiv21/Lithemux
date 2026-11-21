use std::io::Write;
use std::os::unix::net::UnixStream;

use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::ports::app_notifier::AppNotifier;
use crate::platform::paths::home::Home;

/// One JSON line to `<home>/app.sock`. With no app listening, connecting
/// fails at once and nothing happens.
pub struct SocketAppNotifier<'a> {
    home: &'a Home,
}

impl<'a> SocketAppNotifier<'a> {
    pub fn new(home: &'a Home) -> Self {
        SocketAppNotifier { home }
    }
}

impl AppNotifier for SocketAppNotifier<'_> {
    fn notify(&self, session_id: &str, event: HookEvent) {
        let Ok(mut stream) = UnixStream::connect(self.home.app_socket_path()) else {
            return;
        };
        if stream.set_nonblocking(true).is_err() {
            return;
        }
        let line = serde_json::json!({ "session_id": session_id, "event": event.as_str() });
        let _ = stream.write_all(format!("{line}\n").as_bytes());
    }
}
