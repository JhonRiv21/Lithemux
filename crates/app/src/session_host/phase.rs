use super::failure::Failure;

/// Where a session the host was asked to open stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Being prepared in the background: finding `claude`, taking the lock.
    Opening,
    /// Its program runs in a terminal of the host.
    Running,
    /// It was closed and its program is still leaving. Asked for again
    /// meanwhile, it opens by itself once nothing of the old run is left.
    Closing,
    /// Its program ended; the terminal is kept to be read.
    Ended,
    /// Another terminal has it (`lithemux open`, for one). Nothing was
    /// started here. `pid` is who, when known.
    OpenElsewhere { pid: Option<u32> },
    /// It could not be started, and why.
    Failed(Failure),
}
