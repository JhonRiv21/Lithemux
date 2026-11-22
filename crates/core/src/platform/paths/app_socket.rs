//! Where the app listens for the hooks. Next to the rest of lithemux's
//! files when that path fits in a Unix socket address; in a short private
//! directory when it does not (a long `LITHEMUX_HOME`, a long user name).
//! The app and the hook both come here, so they always agree.

use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

/// `sun_path` holds 104 bytes on macOS, one of them the final NUL.
pub const MAX_SOCKET_PATH_BYTES: usize = 103;

const SOCKET_NAME: &str = "app.sock";
/// Short on every machine, unlike `$TMPDIR`.
const SHORT_ROOT: &str = "/tmp";
const PRIVATE_MODE: u32 = 0o700;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSocket {
    path: PathBuf,
    /// The directory of the short path, when that is the one in use.
    private_dir: Option<PathBuf>,
}

/// The socket of the app that runs on `home`, for the user `uid`.
pub fn app_socket(home: &Path, uid: u32) -> AppSocket {
    let in_home = home.join(SOCKET_NAME);
    if in_home.as_os_str().len() <= MAX_SOCKET_PATH_BYTES {
        return AppSocket {
            path: in_home,
            private_dir: None,
        };
    }
    let private_dir = Path::new(SHORT_ROOT).join(format!("lithemux-{uid}"));
    let name = format!("{:016x}.sock", fnv1a(home.as_os_str().as_encoded_bytes()));
    AppSocket {
        path: private_dir.join(name),
        private_dir: Some(private_dir),
    }
}

/// One home, one name, in every build: the app and the hook are separate
/// binaries and must compute the same.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

impl AppSocket {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_outside_the_home(&self) -> bool {
        self.private_dir.is_some()
    }

    /// For the app, before it binds: the private directory exists, is a
    /// real directory of this user and nobody else can enter it.
    pub fn prepare(&self, uid: u32) -> io::Result<()> {
        let Some(dir) = &self.private_dir else {
            return Ok(());
        };
        match DirBuilder::new().mode(PRIVATE_MODE).create(dir) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        if self.is_private_to(uid) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("{} is not a private directory of this user", dir.display()),
            ))
        }
    }

    /// For the hook, before it connects: a socket in a directory somebody
    /// else made, or can enter, is not the app's.
    pub fn is_private_to(&self, uid: u32) -> bool {
        let Some(dir) = &self.private_dir else {
            return true;
        };
        // Not followed: a symlink in `/tmp` could point anywhere.
        fs::symlink_metadata(dir).is_ok_and(|found| {
            found.file_type().is_dir() && found.uid() == uid && found.mode() & 0o077 == 0
        })
    }
}
