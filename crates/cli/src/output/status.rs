use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::{ActivityAge, Kind, Session};
use serde::Serialize;

#[derive(Serialize)]
pub struct SessionStatus {
    pub name: String,
    /// Pid of the process running the session, when it is open.
    pub open_pid: Option<u32>,
    #[serde(flatten)]
    pub session: Session,
}

#[derive(Serialize)]
pub struct CategoryStatus {
    #[serde(flatten)]
    pub category: Category,
    pub sessions: Vec<SessionStatus>,
}

pub fn json(tree: &[CategoryStatus]) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(tree)?);
    Ok(())
}

pub fn tree(tree: &[CategoryStatus], now: i64) {
    if tree.is_empty() {
        println!("No categories yet. Create one with `lithemux cat add`.");
        return;
    }
    for (index, entry) in tree.iter().enumerate() {
        if index > 0 {
            println!();
        }
        category(entry, now);
    }
}

fn category(entry: &CategoryStatus, now: i64) {
    let Category {
        name,
        slug,
        root_dir,
        ..
    } = &entry.category;
    println!("{name} ({slug})  {root_dir}");
    if entry.sessions.is_empty() {
        println!("  (no sessions)");
        return;
    }
    let mother = entry
        .sessions
        .iter()
        .find(|s| s.session.kind == Kind::Mother);
    let rest: Vec<&SessionStatus> = entry
        .sessions
        .iter()
        .filter(|s| s.session.kind != Kind::Mother)
        .collect();
    let name_width = entry
        .sessions
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0);

    // Without a mother every session sits at the same level.
    let indent = if let Some(mother) = mother {
        println!("  {}", row(mother, "◆ ", name_width + 3, now));
        "  "
    } else {
        ""
    };
    for (index, session) in rest.iter().enumerate() {
        let branch = if index + 1 == rest.len() {
            "└─ "
        } else {
            "├─ "
        };
        let width = if mother.is_some() {
            name_width
        } else {
            name_width + 2
        };
        println!("  {indent}{}", row(session, branch, width, now));
    }
}

fn row(status: &SessionStatus, marker: &str, name_width: usize, now: i64) -> String {
    let session = &status.session;
    let open = match status.open_pid {
        Some(pid) => format!("open (pid {pid})"),
        None => "closed".to_string(),
    };
    format!(
        "{marker}{:name_width$}  {:10}  {:12}  {:16}  {}",
        status.name,
        session.state.as_str(),
        last_activity(now, session.last_activity_at),
        open,
        session.role
    )
}

fn last_activity(now: i64, at: Option<i64>) -> String {
    at.map_or_else(
        || "never".to_string(),
        |at| ActivityAge::between(now, at).to_string(),
    )
}
