use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::{Session, SessionRef};

pub fn created(target: &SessionRef, category: &Category, session: &Session) {
    println!("Created session {target}");
    details(category, session);
    println!("Open it with: lithemux open {target}");
}

pub fn updated(target: &SessionRef, category: &Category, session: &Session) {
    println!("Updated session {target}");
    details(category, session);
}

pub fn removed(target: &SessionRef, session: &Session) {
    println!("Removed session {target}.");
    if let Some(id) = session.claude_session_id.as_deref() {
        println!("Its Claude conversation is untouched: claude --resume {id}");
    }
}

fn details(category: &Category, session: &Session) {
    println!("  name:       {}", session.messaging_name(category));
    println!("  kind:       {}", session.kind.as_str());
    println!("  profile:    {}", session.profile.as_str());
    println!("  role:       {}", session.role);
    println!("  cwd:        {}", session.effective_cwd(category));
    println!(
        "  model:      {}",
        session.model.as_deref().unwrap_or("(user default)")
    );
    println!("  keep alive: {}", session.keep_alive);
}
