use std::cell::RefCell;

use lithemux_core::modules::hook_event::{AppNotifier, HookEvent, HookLog, HookOutcome};
use lithemux_core::modules::launch::LaunchNotices;

// Same contract as FileHookLog: appending never fails.
#[derive(Default)]
pub struct RecordingLog {
    pub lines: RefCell<Vec<String>>,
}

impl HookLog for RecordingLog {
    fn append(&self, message: &str) {
        self.lines.borrow_mut().push(message.to_string());
    }
}

// Same contract as SocketAppNotifier: delivery is best effort.
#[derive(Default)]
pub struct RecordingNotifier {
    pub events: RefCell<Vec<(String, HookEvent)>>,
    pub outcomes: RefCell<Vec<HookOutcome>>,
}

impl AppNotifier for RecordingNotifier {
    fn notify(&self, session_id: &str, event: HookEvent, outcome: HookOutcome) {
        self.events
            .borrow_mut()
            .push((session_id.to_string(), event));
        self.outcomes.borrow_mut().push(outcome);
    }
}

#[derive(Default)]
pub struct RecordingNotices {
    pub warnings: RefCell<Vec<String>>,
}

impl LaunchNotices for RecordingNotices {
    fn warn(&self, message: &str) {
        self.warnings.borrow_mut().push(message.to_string());
    }
}
