use crate::modules::session::domain::session::Session;

/// A category has at most one mother. `candidate_id` is the session that
/// wants the role, or `None` when it does not exist yet. On conflict, returns
/// the slug of the current mother.
pub fn check_can_be_mother(
    current_mother: Option<&Session>,
    candidate_id: Option<&str>,
) -> Result<(), String> {
    match current_mother {
        Some(mother) if Some(mother.id.as_str()) != candidate_id => Err(mother.slug.clone()),
        _ => Ok(()),
    }
}
