use crate::modules::session::domain::session::Session;

/// A category has at most one coordinator. `candidate_id` is the session that
/// wants the role, or `None` when it does not exist yet. On conflict, returns
/// the slug of the current coordinator.
pub fn check_can_be_coordinator(
    current_coordinator: Option<&Session>,
    candidate_id: Option<&str>,
) -> Result<(), String> {
    match current_coordinator {
        Some(coordinator) if Some(coordinator.id.as_str()) != candidate_id => {
            Err(coordinator.slug.clone())
        }
        _ => Ok(()),
    }
}
