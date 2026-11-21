//! The threads that move bytes between the PTY and the emulator.

use std::io::{ErrorKind, Read, Write};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use alacritty_terminal::sync::FairMutex;

use super::emulator::Emulator;

const READ_BUFFER_BYTES: usize = 64 * 1024;

pub(super) fn write_input(mut writer: Box<dyn Write + Send>, pending: mpsc::Receiver<Vec<u8>>) {
    for bytes in pending {
        if writer.write_all(&bytes).is_err() || writer.flush().is_err() {
            return;
        }
    }
}

pub(super) fn read_output(
    mut reader: Box<dyn Read + Send>,
    emulator: &FairMutex<Emulator>,
    wake: &async_channel::Sender<()>,
    sync_deadline: &mpsc::Sender<Instant>,
) {
    let mut buffer = vec![0u8; READ_BUFFER_BYTES];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return,
            Ok(read) => {
                if let Some(deadline) = emulator.lock().feed(&buffer[..read]) {
                    let _ = sync_deadline.send(deadline);
                }
                // Full means a repaint is already pending.
                let _ = wake.try_send(());
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(_) => return,
        }
    }
}

/// The reader blocks on the PTY, so a program that opens a synchronized
/// update and then stalls or dies would freeze the screen. This thread waits
/// out the deadline instead and shows what was held back.
pub(super) fn end_stalled_syncs(
    deadlines: &mpsc::Receiver<Instant>,
    emulator: &FairMutex<Emulator>,
    wake: &async_channel::Sender<()>,
) {
    let mut pending: Option<Instant> = None;
    loop {
        let next = match pending {
            Some(deadline) => {
                deadlines.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            }
            None => deadlines.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match next {
            Ok(deadline) => pending = Some(deadline),
            Err(RecvTimeoutError::Timeout) => {
                pending = None;
                if emulator.lock().end_sync_due_by(Instant::now()) {
                    let _ = wake.try_send(());
                }
            }
            // The reader is gone: nothing else will close the update.
            Err(RecvTimeoutError::Disconnected) => {
                let far_future = Instant::now() + Duration::from_secs(3600);
                if emulator.lock().end_sync_due_by(far_future) {
                    let _ = wake.try_send(());
                }
                return;
            }
        }
    }
}
