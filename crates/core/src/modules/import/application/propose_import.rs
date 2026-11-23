use crate::modules::category::list_categories;
use crate::modules::import::application::import_ports::ImportPorts;
use crate::modules::import::domain::proposal::Proposal;
use crate::modules::import::domain::proposal_policy::{Known, propose};
use crate::modules::import::error::ImportError;

/// What importing cmux's sessions would do. Writes nothing.
pub fn propose_import(
    ports: &ImportPorts<'_>,
    home: Option<&str>,
) -> Result<Proposal, ImportError> {
    let panels = ports.cmux.panels()?;
    let categories = list_categories(ports.categories)?;
    let sessions = ports.sessions.list_all()?;
    let has_conversation = |id: &str| ports.transcripts.exists(id);
    Ok(propose(
        &panels,
        &Known {
            categories: &categories,
            sessions: &sessions,
            home,
            has_conversation: &has_conversation,
        },
    ))
}
