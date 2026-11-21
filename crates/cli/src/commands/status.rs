use lithemux_core::modules::category::{get_category, list_categories};
use lithemux_core::modules::launch::SessionLocks;
use lithemux_core::modules::session::list_sessions;
use lithemux_core::platform::clock::system_clock::now_unix;

use crate::output::status::{CategoryStatus, SessionStatus};
use crate::wiring::Wiring;

pub fn show(category: Option<&str>, json: bool) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let (categories, sessions, locks) = (wiring.categories(), wiring.sessions(), wiring.locks());
    let selected = match category {
        Some(slug) => vec![get_category(&categories, slug)?],
        None => list_categories(&categories)?,
    };
    let mut tree = Vec::with_capacity(selected.len());
    for category in selected {
        let sessions = list_sessions(&sessions, &category)?
            .into_iter()
            .map(|session| SessionStatus {
                name: session.messaging_name(&category),
                open_pid: locks.holder(&session.id),
                session,
            })
            .collect();
        tree.push(CategoryStatus { category, sessions });
    }
    if json {
        crate::output::status::json(&tree)
    } else {
        crate::output::status::tree(&tree, now_unix());
        Ok(())
    }
}
