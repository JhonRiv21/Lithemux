use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::session_locks::{HeldSessionLock, SessionLocks};
use crate::platform::paths::fs_error::FsError;
use crate::platform::paths::home::Home;

/// `flock` on `<home>/locks/<session_id>.lock`. The kernel releases it when
/// the holder dies, so a crash never leaves a session locked.
pub struct FlockSessionLocks<'a> {
    home: &'a Home,
}

impl<'a> FlockSessionLocks<'a> {
    pub fn new(home: &'a Home) -> Self {
        FlockSessionLocks { home }
    }
}

struct HeldFlock {
    file: File,
}

impl HeldSessionLock for HeldFlock {
    fn record_pid(&mut self, pid: u32) {
        let _ = self.file.set_len(0);
        let _ = self.file.seek(SeekFrom::Start(0));
        let _ = writeln!(self.file, "{pid}");
        let _ = self.file.flush();
    }
}

impl SessionLocks for FlockSessionLocks<'_> {
    fn acquire(&self, session_id: &str) -> Result<Box<dyn HeldSessionLock>, LaunchError> {
        let dir = self.home.locks_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| FsError::new(format!("creating {}", dir.display()), e))?;
        let path = self.home.lock_path(session_id);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|e| FsError::new(format!("opening {}", path.display()), e))?;
        match file.try_lock() {
            Ok(()) => Ok(Box::new(HeldFlock { file })),
            Err(TryLockError::WouldBlock) => Err(LaunchError::AlreadyOpen {
                pid: read_pid(&mut file),
            }),
            Err(TryLockError::Error(e)) => {
                Err(FsError::new(format!("locking {}", path.display()), e).into())
            }
        }
    }

    fn holder(&self, session_id: &str) -> Option<u32> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.home.lock_path(session_id))
            .ok()?;
        match file.try_lock() {
            Ok(()) => None,
            Err(_) => Some(read_pid(&mut file).unwrap_or(0)),
        }
    }
}

fn read_pid(file: &mut File) -> Option<u32> {
    let mut text = String::new();
    file.seek(SeekFrom::Start(0)).ok()?;
    file.read_to_string(&mut text).ok()?;
    text.trim().parse().ok()
}
