use std::collections::HashSet;

use crate::modules::category::list_categories;
use crate::modules::import::application::import_batch::Batch;
use crate::modules::import::application::import_ports::ImportPorts;
use crate::modules::import::domain::names_in_use::NamesInUse;
use crate::modules::import::domain::proposal::Chosen;
use crate::modules::import::error::ImportError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    pub categories: usize,
    pub sessions: usize,
    /// Chosen, but lithemux had their conversation by then.
    pub already_there: usize,
    /// Boards of new categories that could not be made, with why. The
    /// import stands without them.
    pub boards_failed: Vec<String>,
}

/// Makes the chosen sessions, hibernated and with their conversation, so
/// that opening one resumes it. Nothing is started.
///
/// What is there is read again and everything is stored inside one write
/// transaction, taken before reading: two imports at once cannot both add
/// a conversation, and a failure leaves nothing. The boards of the new
/// categories are made after it commits.
pub fn apply_import(
    ports: &ImportPorts<'_>,
    chosen: &[Chosen],
    now: i64,
) -> Result<Imported, ImportError> {
    let mut stored = None;
    ports.store.atomically(&mut || {
        let batch = build(ports, chosen, now)?;
        for category in &batch.made {
            ports.categories.insert(category)?;
        }
        for session in &batch.sessions {
            ports.sessions.insert(session)?;
        }
        stored = Some(batch);
        Ok(())
    })?;
    let Some(batch) = stored else {
        return Err(ImportError::NothingStored);
    };
    let boards_failed = batch
        .made
        .iter()
        .filter_map(|category| {
            let made = ports.boards.ensure(category);
            made.err()
                .map(|error| format!("{}: {error}", category.slug))
        })
        .collect();
    Ok(Imported {
        categories: batch.made.len(),
        sessions: batch.sessions.len(),
        already_there: chosen.len() - batch.sessions.len(),
        boards_failed,
    })
}

/// The categories and sessions to add, against what is stored now.
fn build(ports: &ImportPorts<'_>, chosen: &[Chosen], now: i64) -> Result<Batch, ImportError> {
    let existing = list_categories(ports.categories)?;
    let sessions = ports.sessions.list_all()?;
    let mut known: HashSet<String> = sessions
        .iter()
        .filter_map(|session| session.claude_session_id.clone())
        .collect();
    let mut batch = Batch {
        names: NamesInUse::of(&existing, &sessions),
        existing,
        made: Vec::new(),
        sessions: Vec::new(),
        next_category_sort: ports.categories.next_sort()?,
        next_pin: ports
            .sessions
            .list_pinned()?
            .iter()
            .filter_map(|session| session.pin_sort)
            .max()
            .map_or(0, |last| last + 1),
    };
    for one in chosen {
        if known.insert(one.claude_session_id.clone()) {
            batch.add(ports, one, now)?;
        }
    }
    Ok(batch)
}
