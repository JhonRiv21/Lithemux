use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use super::error::HostError;

/// Held by the one running app. The kernel releases it when the app dies,
/// however it dies.
pub(super) struct InstanceLock {
    file: File,
}

impl InstanceLock {
    pub fn acquire(path: &Path) -> Result<Self, HostError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(HostError::io("opening", path))?;
        match file.try_lock() {
            Ok(()) => {
                let _ = file.set_len(0);
                let _ = writeln!(file, "{}", std::process::id());
                let _ = file.flush();
                Ok(InstanceLock { file })
            }
            Err(TryLockError::WouldBlock) => {
                let mut text = String::new();
                let _ = file.seek(SeekFrom::Start(0));
                let _ = file.read_to_string(&mut text);
                Err(HostError::AlreadyRunning {
                    pid: text.trim().parse().ok(),
                })
            }
            Err(TryLockError::Error(error)) => Err(HostError::io("locking", path)(error)),
        }
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = self.file.set_len(0);
    }
}
