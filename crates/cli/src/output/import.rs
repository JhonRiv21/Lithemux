use lithemux_core::modules::category::Category;
use lithemux_core::modules::import::{Choices, Destination, Imported, Proposal, ProposedSession};
use serde_json::{Value, json};

/// A proposal as the user's choices leave it.
pub struct Shown<'a> {
    pub proposal: &'a Proposal,
    pub choices: &'a Choices<'a>,
    pub categories: &'a [Category],
    pub home: &'a str,
}

impl Shown<'_> {
    fn is_marked(&self, session: &ProposedSession) -> bool {
        let wanted = session.selected || self.choices.include.contains(&session.number);
        wanted && !self.choices.skip.contains(&session.number)
    }

    fn destination(&self, session: &ProposedSession) -> Destination {
        match (&session.destination, self.choices.into) {
            (Destination::Unassigned, Some(into)) => into.clone(),
            (destination, _) => destination.clone(),
        }
    }

    /// The slug of where it goes, and whether that category is new.
    fn category(&self, destination: &Destination) -> Option<(String, bool)> {
        match destination {
            Destination::Existing { category_id } => self
                .categories
                .iter()
                .find(|category| &category.id == category_id)
                .map(|category| (category.slug.clone(), false)),
            Destination::New { root_dir } => self
                .proposal
                .new_categories
                .iter()
                .find(|category| &category.root_dir == root_dir)
                .map(|category| (category.slug.clone(), true)),
            Destination::Unassigned => None,
        }
    }

    fn folder(&self, folder: &str) -> String {
        match folder.strip_prefix(self.home) {
            Some("") => "~".to_string(),
            Some(rest) if rest.starts_with('/') => format!("~{rest}"),
            _ => folder.to_string(),
        }
    }
}

fn summary(proposal: &Proposal) -> String {
    let count =
        |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    format!(
        "cmux: {} to import ({} already in lithemux, {} without a conversation id, {} without Claude)",
        count(proposal.sessions.len(), "Claude session", "Claude sessions"),
        proposal.already_imported,
        proposal.without_conversation_id,
        proposal.not_claude,
    )
}

pub fn proposal(shown: &Shown<'_>) {
    println!("{}", summary(shown.proposal));
    if shown.proposal.sessions.is_empty() {
        return;
    }
    let width = shown
        .proposal
        .sessions
        .iter()
        .map(|session| session.name.chars().count())
        .max()
        .unwrap_or(0);
    println!();
    for session in &shown.proposal.sessions {
        println!("{}", row(shown, session, width));
    }
    println!();
    println!(
        "Nothing was written. To import the sessions marked yes: lithemux import cmux --apply \
         [--into <category>] [--skip <n,…>] [--include <n,…>]"
    );
}

fn row(shown: &Shown<'_>, session: &ProposedSession, width: usize) -> String {
    let category = match shown.category(&shown.destination(session)) {
        Some((slug, true)) => format!("{slug} (new)"),
        Some((slug, false)) => slug,
        None => "(no category: use --into)".to_string(),
    };
    let mut notes = Vec::new();
    if session.pinned {
        notes.push("pinned");
    }
    if !session.has_conversation {
        notes.push("no saved conversation");
    }
    format!(
        "{:>3}  {:<3}  {:<width$}  {:<28}  {}{}",
        session.number,
        if shown.is_marked(session) {
            "yes"
        } else {
            "no"
        },
        session.name,
        category,
        shown.folder(&session.folder),
        if notes.is_empty() {
            String::new()
        } else {
            format!("  ({})", notes.join(", "))
        },
    )
}

pub fn proposal_json(shown: &Shown<'_>) -> anyhow::Result<()> {
    let sessions: Vec<Value> = shown
        .proposal
        .sessions
        .iter()
        .map(|session| {
            let category = shown.category(&shown.destination(session));
            json!({
                "number": session.number,
                "name": session.name,
                "slug": session.slug,
                "folder": session.folder,
                "pinned": session.pinned,
                "has_conversation": session.has_conversation,
                "import": shown.is_marked(session),
                "category": category.as_ref().map(|(slug, _)| slug),
                "new_category": category.is_some_and(|(_, new)| new),
            })
        })
        .collect();
    let proposal = shown.proposal;
    let value = json!({
        "sessions": sessions,
        "new_categories": proposal.new_categories.iter().map(|c| json!({
            "slug": c.slug, "name": c.name, "root_dir": c.root_dir,
        })).collect::<Vec<_>>(),
        "already_imported": proposal.already_imported,
        "without_conversation_id": proposal.without_conversation_id,
        "not_claude": proposal.not_claude,
    });
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

pub fn imported(imported: &Imported) {
    let plural =
        |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    println!(
        "Imported {} and {}, hibernated: opening one resumes its conversation.",
        plural(imported.sessions, "session", "sessions"),
        plural(imported.categories, "new category", "new categories"),
    );
    for failed in &imported.boards_failed {
        eprintln!("lithemux: warning: the board of {failed}");
    }
    if imported.already_there > 0 {
        println!(
            "{} already in lithemux, left as they were.",
            imported.already_there
        );
    }
}
