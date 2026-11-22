//! The app's socket (`Home::app_socket`): where the hook says that a session changed. A message
//! carries no state: it names the session, and the host reads the database.

use std::fs;
use std::io::{ErrorKind, Read};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use super::error::HostError;
use super::mailbox::{Mailbox, Signal};

/// Only the user may connect.
const SOCKET_MODE: u32 = 0o600;
/// All the time a client gets to send its line, however it paces it.
const READ_PATIENCE: Duration = Duration::from_millis(200);
/// Nothing a hook sends is longer than this; the rest is not read.
const MAX_MESSAGE_BYTES: usize = 1024;
/// The pause after a failed `accept`.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(50);

pub(super) struct EventSocket {
    path: PathBuf,
    closing: Arc<AtomicBool>,
}

impl EventSocket {
    pub fn bind(path: &Path, mailbox: Mailbox) -> Result<Self, HostError> {
        remove_if_nobody_answers(path)?;
        let listener = UnixListener::bind(path).map_err(HostError::io("binding", path))?;
        fs::set_permissions(path, fs::Permissions::from_mode(SOCKET_MODE))
            .map_err(HostError::io("restricting", path))?;
        let closing = Arc::new(AtomicBool::new(false));
        thread::spawn({
            let closing = Arc::clone(&closing);
            move || accept_until_closed(&listener, &mailbox, &closing)
        });
        Ok(EventSocket {
            path: path.to_path_buf(),
            closing,
        })
    }
}

impl Drop for EventSocket {
    fn drop(&mut self) {
        self.closing.store(true, Ordering::Release);
        // Wakes the thread that waits in `accept`.
        let _ = UnixStream::connect(&self.path);
        let _ = fs::remove_file(&self.path);
    }
}

/// A socket file left by an app that died is in the way of a new one.
fn remove_if_nobody_answers(path: &Path) -> Result<(), HostError> {
    if !path.exists() {
        return Ok(());
    }
    if UnixStream::connect(path).is_ok() {
        return Err(HostError::SocketInUse(path.to_path_buf()));
    }
    fs::remove_file(path).map_err(HostError::io("removing the stale", path))
}

fn accept_until_closed(listener: &UnixListener, mailbox: &Mailbox, closing: &AtomicBool) {
    for stream in listener.incoming() {
        if closing.load(Ordering::Acquire) {
            return;
        }
        match stream {
            // Each client gets a short-lived thread, so one that is silent
            // or slow holds up nobody else.
            Ok(stream) => {
                let mailbox = mailbox.clone();
                thread::spawn(move || {
                    if let Some(session_id) = session_named_by(stream) {
                        mailbox.post(Signal::Hook { session_id });
                    }
                });
            }
            // Out of descriptors, for one: do not spin while it lasts.
            Err(_) => thread::sleep(ACCEPT_BACKOFF),
        }
    }
}

/// The session a well-formed message names; `None` for anything else, and
/// for a client that does not finish its line within `READ_PATIENCE`.
fn session_named_by(mut stream: UnixStream) -> Option<String> {
    let deadline = Instant::now() + READ_PATIENCE;
    let mut message = Vec::new();
    let mut chunk = [0_u8; 256];
    while !message.contains(&b'\n') && message.len() < MAX_MESSAGE_BYTES {
        let left = deadline.checked_duration_since(Instant::now())?;
        // A hook writes and hangs up at once, often before it is accepted
        // here; the timeout cannot be set on such a connection and is not
        // needed: what it sent is waiting, followed by the end.
        let _ = stream.set_read_timeout(Some(left.max(Duration::from_millis(1))));
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => message.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
    let line = message.split(|byte| *byte == b'\n').next()?;
    session_id_of(std::str::from_utf8(line).ok()?)
}

/// `{"session_id": "…", "event": "…"}`, both of them text.
pub(super) fn session_id_of(message: &str) -> Option<String> {
    let message: serde_json::Value = serde_json::from_str(message.trim()).ok()?;
    message.get("event")?.as_str()?;
    let session_id = message.get("session_id")?.as_str()?;
    (!session_id.is_empty()).then(|| session_id.to_string())
}
