use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::session_locks::{HeldSessionLock, SessionLocks};
use crate::platform::paths::fs_error::FsError;
use crate::platform::paths::home::Home;
use crate::platform::process::liveness::is_alive;

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

impl HeldFlock {
    fn write_pid(&mut self, pid: u32) {
        let _ = self.file.set_len(0);
        let _ = self.file.seek(SeekFrom::Start(0));
        let _ = writeln!(self.file, "{pid}");
        let _ = self.file.flush();
    }
}

impl HeldSessionLock for HeldFlock {
    fn record_pid(&mut self, pid: u32) {
        self.write_pid(pid);
    }
}

impl Drop for HeldFlock {
    // An empty file names nobody, so the pid cannot outlive the lock.
    fn drop(&mut self) {
        let _ = self.file.set_len(0);
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
            Ok(()) => {
                let mut held = HeldFlock { file };
                // Whatever a dead holder left is replaced at once.
                held.write_pid(std::process::id());
                Ok(Box::new(held))
            }
            Err(TryLockError::WouldBlock) => Err(LaunchError::AlreadyOpen {
                pid: read_pid(&mut file),
            }),
            Err(TryLockError::Error(e)) => {
                Err(FsError::new(format!("locking {}", path.display()), e).into())
            }
        }
    }

    /// Reads the pid without touching the lock, so asking never makes an
    /// `acquire` fail. A pid left behind by a crash is not alive, and one
    /// that was recycled can only make `status` say "open" by mistake.
    fn holder(&self, session_id: &str) -> Option<u32> {
        let mut file = File::open(self.home.lock_path(session_id)).ok()?;
        read_pid(&mut file).filter(|pid| is_alive(*pid))
    }
}

fn read_pid(file: &mut File) -> Option<u32> {
    let mut text = String::new();
    file.seek(SeekFrom::Start(0)).ok()?;
    file.read_to_string(&mut text).ok()?;
    text.trim().parse().ok()
}
