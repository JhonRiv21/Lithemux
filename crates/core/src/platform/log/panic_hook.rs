//! A panic leaves its trace in the app's log before the process goes on
//! unwinding, or aborts. The hook runs on the thread that panicked, first
//! of all, whatever that thread is.

use std::panic::{self, PanicHookInfo};
use std::sync::Arc;

use crate::platform::clock::system_clock::now_unix;
use crate::platform::log::app_log::{AppLog, Level};

/// A panic's message can carry a piece of what was being handled (slicing
/// text, unwrapping an error): only its start is kept.
const MESSAGE_CHARS: usize = 120;

/// Where it happened and the start of what the panic said.
pub fn describe_panic(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(no message)".to_string());
    let message = match message.char_indices().nth(MESSAGE_CHARS) {
        Some((cut, _)) => format!("{}…", &message[..cut]),
        None => message,
    };
    let place = info
        .location()
        .map_or_else(String::new, |at| format!(" at {}:{}", at.file(), at.line()));
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("unnamed");
    format!("panic in thread {name}{place}: {message}")
}

/// Writes every panic to `log`, then lets the hook that was there (the
/// one that prints to stderr) do its part.
pub fn log_panics(log: Arc<AppLog>) {
    let before = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        log.write(Level::Error, &describe_panic(info), now_unix());
        before(info);
    }));
}
