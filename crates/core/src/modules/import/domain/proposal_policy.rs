use std::collections::HashSet;
use std::path::Path;

use uuid::Uuid;

use crate::modules::category::{Category, slug_from_name};
use crate::modules::import::domain::cmux_panel::CmuxPanel;
use crate::modules::import::domain::names_in_use::NamesInUse;
use crate::modules::import::domain::proposal::{
    Destination, NewCategoryProposal, Proposal, ProposedSession,
};
use crate::modules::session::Session;

/// How cmux names the agent of a panel that runs Claude Code.
const CLAUDE: &str = "claude";
/// The name of a session when its panel gives none.
const FALLBACK_NAME: &str = "session";

/// What lithemux has, against which cmux's panels are proposed.
pub struct Known<'a> {
    pub categories: &'a [Category],
    pub sessions: &'a [Session],
    /// The user's home folder.
    pub home: Option<&'a str>,
    pub has_conversation: &'a dyn Fn(&str) -> bool,
}

enum Left {
    NotClaude,
    WithoutId,
}

/// Each Claude panel of cmux as a session of lithemux. A conversation
/// lithemux already has is left out, so importing again adds nothing twice.
pub fn propose(panels: &[CmuxPanel], known: &Known<'_>) -> Proposal {
    let mut proposal = Proposal::default();
    let mut names = NamesInUse::of(known.categories, known.sessions);
    let mut seen: HashSet<String> = known
        .sessions
        .iter()
        .filter_map(|session| session.claude_session_id.clone())
        .collect();
    for (index, panel) in panels.iter().enumerate() {
        match conversation_of(panel) {
            Err(Left::NotClaude) => proposal.not_claude += 1,
            Err(Left::WithoutId) => proposal.without_conversation_id += 1,
            Ok(id) if !seen.insert(id.to_string()) => proposal.already_imported += 1,
            Ok(id) => {
                let session = propose_one(index + 1, panel, id, known, &mut names, &mut proposal);
                proposal.sessions.push(session);
            }
        }
    }
    proposal
}

fn conversation_of(panel: &CmuxPanel) -> Result<&str, Left> {
    if panel.agent.as_deref() != Some(CLAUDE) {
        return Err(Left::NotClaude);
    }
    panel
        .claude_session_id
        .as_deref()
        .filter(|id| Uuid::parse_str(id).is_ok())
        .ok_or(Left::WithoutId)
}

fn propose_one(
    number: usize,
    panel: &CmuxPanel,
    claude_session_id: &str,
    known: &Known<'_>,
    names: &mut NamesInUse,
    proposal: &mut Proposal,
) -> ProposedSession {
    let folder = panel.folder.clone().unwrap_or_default();
    let name = name_of(panel, &folder);
    let destination = destination_of(&folder, known);
    let slug = match &destination {
        Destination::Existing { category_id } => {
            let category = known.categories.iter().find(|c| &c.id == category_id);
            let category_slug = category.map_or("", |c| c.slug.as_str());
            names.claim_session(category_slug, &name)
        }
        Destination::New { root_dir } => {
            let category_slug = new_category_for(root_dir, names, &mut proposal.new_categories);
            names.claim_session(&category_slug, &name)
        }
        // Its slug is settled once it has a category.
        Destination::Unassigned => {
            slug_from_name(&name).unwrap_or_else(|| FALLBACK_NAME.to_string())
        }
    };
    let has_conversation = (known.has_conversation)(claude_session_id);
    ProposedSession {
        number,
        name,
        slug,
        folder,
        pinned: panel.pinned,
        claude_session_id: claude_session_id.to_string(),
        has_conversation,
        selected: has_conversation,
        destination,
    }
}

/// The user's name for it, the title its program set or its folder. Claude
/// puts a spinner in front of its title, which is no part of the name.
fn name_of(panel: &CmuxPanel, folder: &str) -> String {
    let given = |title: &Option<String>| {
        title
            .as_deref()
            .map(|title| {
                title
                    .trim_start_matches(|c: char| !c.is_alphanumeric())
                    .trim()
            })
            .filter(|title| !title.is_empty())
            .map(str::to_string)
    };
    let custom = panel
        .custom_title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_string);
    custom
        .or_else(|| given(&panel.process_title))
        .or_else(|| {
            Path::new(folder)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| FALLBACK_NAME.to_string())
}

/// The home folder is anybody's: the user chooses. Otherwise the deepest
/// category whose root holds the folder (one rooted at the home or above
/// does not count), or a new one for it.
fn destination_of(folder: &str, known: &Known<'_>) -> Destination {
    let path = Path::new(folder);
    let is_home = known.home.is_some_and(|home| Path::new(home) == path);
    if folder.is_empty() || is_home {
        return Destination::Unassigned;
    }
    // A category rooted at the home folder, or above it, would hold every
    // folder: it is the user's to choose, like the home itself.
    let holds_the_home = |category: &&Category| {
        known
            .home
            .is_some_and(|home| Path::new(home).starts_with(&category.root_dir))
    };
    known
        .categories
        .iter()
        .filter(|category| path.starts_with(&category.root_dir))
        .filter(|category| !holds_the_home(category))
        .max_by_key(|category| Path::new(&category.root_dir).components().count())
        .map_or_else(
            || Destination::New {
                root_dir: folder.to_string(),
            },
            |category| Destination::Existing {
                category_id: category.id.clone(),
            },
        )
}

/// The slug of the new category for `root_dir`, made the first time a
/// session of that folder comes.
fn new_category_for(
    root_dir: &str,
    names: &mut NamesInUse,
    new_categories: &mut Vec<NewCategoryProposal>,
) -> String {
    if let Some(made) = new_categories.iter().find(|c| c.root_dir == root_dir) {
        return made.slug.clone();
    }
    let (name, slug) = names.claim_category(root_dir);
    new_categories.push(NewCategoryProposal {
        slug: slug.clone(),
        name,
        root_dir: root_dir.to_string(),
    });
    slug
}
