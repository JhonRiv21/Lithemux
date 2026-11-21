use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::modules::hook_event::ports::hook_log::HookLog;
use crate::platform::clock::system_clock::now_unix;
use crate::platform::paths::home::Home;

const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// `<home>/logs/hook.log`, with one rotated file once it reaches 1 MB.
pub struct FileHookLog<'a> {
    home: &'a Home,
}

impl<'a> FileHookLog<'a> {
    pub fn new(home: &'a Home) -> Self {
        FileHookLog { home }
    }
}

impl HookLog for FileHookLog<'_> {
    fn append(&self, message: &str) {
        let path = self.home.hook_log_path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if fs::metadata(&path).is_ok_and(|meta| meta.len() >= MAX_LOG_BYTES) {
            let _ = fs::rename(&path, path.with_extension("log.1"));
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{} {message}", now_unix());
        }
    }
}
