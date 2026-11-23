use crate::modules::import::domain::proposal::{Chosen, Destination, Proposal};
use crate::modules::import::error::ImportError;

/// What the user changed of a proposal, by the numbers of its sessions.
#[derive(Debug, Clone, Default)]
pub struct Choices<'a> {
    /// Left out, even if selected.
    pub skip: &'a [usize],
    /// Taken in, even if not selected (a session without a saved
    /// conversation).
    pub include: &'a [usize],
    /// Where the sessions without a category go.
    pub into: Option<&'a Destination>,
}

/// The sessions to import. Fails, without choosing any, when a number is
/// not in the proposal or a chosen session is left without a category.
pub fn choose(proposal: &Proposal, choices: &Choices<'_>) -> Result<Vec<Chosen>, ImportError> {
    let known = |number: &usize| proposal.sessions.iter().any(|s| s.number == *number);
    if let Some(number) = choices
        .skip
        .iter()
        .chain(choices.include)
        .find(|number| !known(number))
    {
        return Err(ImportError::UnknownNumber(*number));
    }
    let mut chosen = Vec::new();
    let mut without_category = Vec::new();
    for session in &proposal.sessions {
        let wanted = session.selected || choices.include.contains(&session.number);
        if !wanted || choices.skip.contains(&session.number) {
            continue;
        }
        let destination = match (&session.destination, choices.into) {
            (Destination::Unassigned, Some(into)) => into.clone(),
            (Destination::Unassigned, None) => {
                without_category.push(session.number);
                continue;
            }
            (destination, _) => destination.clone(),
        };
        chosen.push(session.chosen(destination));
    }
    if !without_category.is_empty() {
        return Err(ImportError::WithoutCategory(without_category));
    }
    Ok(chosen)
}
