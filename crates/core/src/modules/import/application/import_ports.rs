use crate::modules::category::{BoardStore, CategoryRepository};
use crate::modules::import::ports::cmux_snapshot::CmuxSnapshot;
use crate::modules::import::ports::import_store::ImportStore;
use crate::modules::launch::TranscriptLocator;
use crate::modules::session::SessionRepository;

pub struct ImportPorts<'a> {
    pub cmux: &'a dyn CmuxSnapshot,
    pub categories: &'a dyn CategoryRepository,
    pub sessions: &'a dyn SessionRepository,
    /// Whether Claude Code keeps a conversation: the only look inside
    /// `~/.claude` (PLAN principle 4).
    pub transcripts: &'a dyn TranscriptLocator,
    pub boards: &'a dyn BoardStore,
    /// The transaction the import is stored in; `categories` and
    /// `sessions` write inside it.
    pub store: &'a dyn ImportStore,
}
