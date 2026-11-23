use crate::modules::import::error::ImportError;

/// Stores an import all at once, or nothing of it.
pub trait ImportStore {
    /// Runs `work` inside one write transaction, taken before `work` reads
    /// anything, and keeps what it wrote only if it succeeds.
    fn atomically(
        &self,
        work: &mut dyn FnMut() -> Result<(), ImportError>,
    ) -> Result<(), ImportError>;
}
