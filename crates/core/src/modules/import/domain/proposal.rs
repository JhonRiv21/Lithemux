/// Where an imported session goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// A category lithemux already has.
    Existing { category_id: String },
    /// A category made for this folder, one of `Proposal::new_categories`.
    New { root_dir: String },
    /// Nothing fits (it worked in the user's home): the user chooses.
    Unassigned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedSession {
    /// From 1, in the order of cmux's file: how the CLI names it.
    pub number: usize,
    pub name: String,
    /// Its slug where it is proposed to go.
    pub slug: String,
    pub folder: String,
    pub pinned: bool,
    pub claude_session_id: String,
    /// Whether Claude Code keeps its conversation. Without it, resuming
    /// starts over.
    pub has_conversation: bool,
    /// Whether it is imported unless the user says otherwise.
    pub selected: bool,
    pub destination: Destination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCategoryProposal {
    pub slug: String,
    pub name: String,
    pub root_dir: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Proposal {
    pub sessions: Vec<ProposedSession>,
    pub new_categories: Vec<NewCategoryProposal>,
    /// Panels whose conversation lithemux already has.
    pub already_imported: usize,
    /// Claude panels that name no conversation.
    pub without_conversation_id: usize,
    /// Panels with no Claude in them.
    pub not_claude: usize,
}

/// A session the user decided to import, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// Its number in the proposal.
    pub number: usize,
    pub name: String,
    pub folder: String,
    pub pinned: bool,
    pub claude_session_id: String,
    pub destination: Destination,
}

impl ProposedSession {
    pub fn chosen(&self, destination: Destination) -> Chosen {
        Chosen {
            number: self.number,
            name: self.name.clone(),
            folder: self.folder.clone(),
            pinned: self.pinned,
            claude_session_id: self.claude_session_id.clone(),
            destination,
        }
    }
}
