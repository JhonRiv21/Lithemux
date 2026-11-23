use std::path::Path;

use uuid::Uuid;

use crate::modules::category::{Category, CategoryError, Slug};
use crate::modules::import::application::import_ports::ImportPorts;
use crate::modules::import::domain::names_in_use::NamesInUse;
use crate::modules::import::domain::proposal::{Chosen, Destination};
use crate::modules::import::error::ImportError;
use crate::modules::session::{Kind, Profile, Session, State};

/// What an import adds, before it is stored.
pub(super) struct Batch {
    pub(super) names: NamesInUse,
    pub(super) existing: Vec<Category>,
    pub(super) made: Vec<Category>,
    pub(super) sessions: Vec<Session>,
    pub(super) next_category_sort: i64,
    pub(super) next_pin: i64,
}

impl Batch {
    pub(super) fn add(
        &mut self,
        ports: &ImportPorts<'_>,
        one: &Chosen,
        now: i64,
    ) -> Result<(), ImportError> {
        let category = self.category_for(one, now)?;
        let slug = self.names.claim_session(&category.slug, &one.name);
        // Never stored unless it can be opened (`claude -n <cat>-<slug>`).
        Slug::parse(&slug).map_err(CategoryError::from)?;
        let earlier = self
            .sessions
            .iter()
            .filter(|s| s.category_id == category.id)
            .count();
        let sort = ports.sessions.next_sort(&category.id)? + earlier as i64;
        let pin_sort = one.pinned.then(|| {
            self.next_pin += 1;
            self.next_pin - 1
        });
        let in_root = Path::new(&one.folder) == Path::new(&category.root_dir);
        self.sessions.push(Session {
            id: Uuid::new_v4().to_string(),
            category_id: category.id.clone(),
            title: (one.name != slug).then(|| one.name.clone()),
            slug,
            role: one.name.clone(),
            kind: Kind::Member,
            profile: Profile::Claude,
            cwd: (!in_root && !one.folder.is_empty()).then(|| one.folder.clone()),
            claude_session_id: Some(one.claude_session_id.clone()),
            model: None,
            keep_alive: false,
            state: State::Hibernated,
            claude_version: None,
            last_activity_at: None,
            created_at: now,
            sort,
            pin_sort,
            preview: None,
            notice: None,
            unread: 0,
        });
        Ok(())
    }

    fn category_for(&mut self, one: &Chosen, now: i64) -> Result<Category, ImportError> {
        let root_dir = match &one.destination {
            Destination::Existing { category_id } => {
                return self
                    .existing
                    .iter()
                    .find(|category| &category.id == category_id)
                    .cloned()
                    .ok_or_else(|| CategoryError::NotFound(category_id.clone()).into());
            }
            Destination::Unassigned => return Err(ImportError::WithoutCategory(vec![one.number])),
            Destination::New { root_dir } => root_dir,
        };
        // A category with that root may exist by now: the proposal can be
        // older than it.
        let same_root = |category: &&Category| Path::new(&category.root_dir) == Path::new(root_dir);
        if let Some(there) = self.existing.iter().chain(&self.made).find(same_root) {
            return Ok(there.clone());
        }
        let (name, slug) = self.names.claim_category(root_dir);
        Slug::parse(&slug).map_err(CategoryError::from)?;
        let category = Category {
            id: Uuid::new_v4().to_string(),
            slug,
            name,
            root_dir: root_dir.clone(),
            color: None,
            sort: self.next_category_sort,
            created_at: now,
            collapsed: false,
            notify: true,
        };
        self.next_category_sort += 1;
        self.made.push(category.clone());
        Ok(category)
    }
}
