use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;

use rustix::io::ioctl_fionbio;
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType, connect, socket};

use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_outcome::HookOutcome;
use crate::modules::hook_event::ports::app_notifier::AppNotifier;
use crate::platform::paths::home::{Home, current_user};

/// One JSON line to the app's socket (`Home::app_socket`). With no app listening, connecting
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
    fn notify(&self, session_id: &str, event: HookEvent, outcome: HookOutcome) {
        let socket = self.home.app_socket();
        // A socket outside the home is only the app's in a directory that
        // is ours alone.
        if !socket.is_private_to(current_user()) {
            return;
        }
        let Some(mut stream) = connect_without_waiting(socket.path()) else {
            return;
        };
        let line = serde_json::json!({
            "session_id": session_id,
            "event": event.as_str(),
            "entered": outcome.entered.map(|state| state.as_str()),
        });
        let _ = stream.write_all(format!("{line}\n").as_bytes());
    }
}

/// The socket is non-blocking before it connects, so an app that is not
/// accepting can never hold the hook: the connection either is there at
/// once or is given up.
fn connect_without_waiting(path: &Path) -> Option<UnixStream> {
    let address = SocketAddrUnix::new(path).ok()?;
    let descriptor = socket(AddressFamily::UNIX, SocketType::STREAM, None).ok()?;
    ioctl_fionbio(&descriptor, true).ok()?;
    connect(&descriptor, &address).ok()?;
    Some(UnixStream::from(descriptor))
}
