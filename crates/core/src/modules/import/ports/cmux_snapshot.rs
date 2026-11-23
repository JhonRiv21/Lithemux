use crate::modules::import::domain::cmux_panel::CmuxPanel;
use crate::modules::import::error::ImportError;

/// What cmux saved of its workspaces. Only ever read: lithemux never
/// writes where cmux keeps its files.
pub trait CmuxSnapshot {
    fn panels(&self) -> Result<Vec<CmuxPanel>, ImportError>;
}
