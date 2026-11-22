//! What the user changes from the sidebar. Each one is a use case of
//! `core`, or a few of them, on the app's database.

use std::path::Path;

use lithemux_core::modules::category::{
    Category, CategoryError, FsBoardStore, FsDirectories, NewCategory, Slug,
    SqliteCategoryRepository, create_category, find_category_by_id, free_slug, list_categories,
    locate_directory, remove_category, rename_category, reorder_categories, set_collapsed,
    set_notify, slug_from_name,
};
use lithemux_core::modules::session::{
    Kind, NewSession, Profile, Session, SessionRef, SessionRepository, SqliteSessionRepository,
    create_session, find_session_by_id, list_all_sessions, list_sessions, pin_session,
    remove_session, reorder_pins, reorder_sessions, set_keep_alive, set_kind, set_title,
    unpin_session,
};
use lithemux_core::platform::clock::system_clock::now_unix;
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;

use super::error::SidebarError;
use super::order::{dropped_on, shifted, with_shown_reordered};

const TERMINAL_SLUG: &str = "terminal";

pub struct SidebarActions<'a> {
    pub db: &'a Db,
    pub home: &'a Home,
}

impl SidebarActions<'_> {
    fn categories(&self) -> SqliteCategoryRepository<'_> {
        SqliteCategoryRepository::new(self.db)
    }

    fn sessions(&self) -> SqliteSessionRepository<'_> {
        SqliteSessionRepository::new(self.db)
    }

    /// Every category, in order, and every session, in the order of its
    /// category.
    pub fn load(&self) -> Result<(Vec<Category>, Vec<Session>), SidebarError> {
        let categories = list_categories(&self.categories())?;
        // Two queries whatever the number of categories: this runs every
        // few seconds on the interface thread.
        let sessions = list_all_sessions(&self.sessions())?;
        Ok((categories, sessions))
    }

    fn session(&self, id: &str) -> Result<(Category, Session), SidebarError> {
        find_session_by_id(&self.categories(), &self.sessions(), id)?.ok_or(SidebarError::Gone)
    }

    fn category(&self, id: &str) -> Result<Category, SidebarError> {
        find_category_by_id(&self.categories(), id)?.ok_or(SidebarError::Gone)
    }

    /// The name is what the user typed; the slug is made from it.
    pub fn create_category(&self, name: &str, folder: &Path) -> Result<Category, SidebarError> {
        let slug =
            slug_from_name(name).ok_or_else(|| SidebarError::UnusableName(name.to_string()))?;
        let existing = list_categories(&self.categories())?;
        let taken = |slug: &str| existing.iter().any(|category| category.slug == slug);
        let new = NewCategory {
            slug: Slug::parse(&free_slug(&slug, taken)).map_err(CategoryError::from)?,
            name: name.trim().to_string(),
            root_dir: locate_directory(&FsDirectories, folder)?,
            color: None,
        };
        Ok(create_category(
            &self.categories(),
            &FsBoardStore::new(self.home),
            new,
            now_unix(),
        )?)
    }

    /// A session named after what the user typed. When that is not a slug
    /// as typed, it stays as the title.
    pub fn create_session(
        &self,
        category_id: &str,
        name: &str,
        profile: Profile,
    ) -> Result<(Category, Session), SidebarError> {
        let category = self.category(category_id)?;
        let slug =
            slug_from_name(name).ok_or_else(|| SidebarError::UnusableName(name.to_string()))?;
        let created = self.add_session(&category, &slug, name.trim(), profile)?;
        if name.trim() != slug {
            set_title(&self.sessions(), &created.1.id, name)?;
        }
        self.session(&created.1.id)
    }

    /// A shell session with the next free name of the category.
    pub fn create_terminal(&self, category_id: &str) -> Result<(Category, Session), SidebarError> {
        let category = self.category(category_id)?;
        let existing = list_sessions(&self.sessions(), &category)?;
        let slug = free_slug(TERMINAL_SLUG, |slug| {
            existing.iter().any(|session| session.slug == slug)
        });
        self.add_session(&category, &slug, &slug, Profile::Shell)
    }

    fn add_session(
        &self,
        category: &Category,
        slug: &str,
        role: &str,
        profile: Profile,
    ) -> Result<(Category, Session), SidebarError> {
        let new = NewSession {
            target: SessionRef::parse(&format!("{}/{slug}", category.slug))?,
            role: role.to_string(),
            kind: Kind::Member,
            profile,
            cwd: None,
            model: None,
        };
        Ok(create_session(
            &self.categories(),
            &self.sessions(),
            new,
            now_unix(),
        )?)
    }

    pub fn rename_session(&self, id: &str, title: &str) -> Result<(), SidebarError> {
        Ok(set_title(&self.sessions(), id, title)?)
    }

    pub fn rename_category(&self, id: &str, name: &str) -> Result<(), SidebarError> {
        Ok(rename_category(&self.categories(), id, name)?)
    }

    pub fn set_pinned(&self, id: &str, pinned: bool) -> Result<(), SidebarError> {
        if pinned {
            Ok(pin_session(&self.sessions(), id)?)
        } else {
            Ok(unpin_session(&self.sessions(), id)?)
        }
    }

    /// Refused by `core` when the category already has another coordinator.
    pub fn set_coordinator(&self, id: &str, coordinator: bool) -> Result<(), SidebarError> {
        let (category, session) = self.session(id)?;
        let kind = if coordinator {
            Kind::Coordinator
        } else {
            Kind::Member
        };
        Ok(set_kind(&self.sessions(), &category, &session, kind)?)
    }

    pub fn set_keep_alive(&self, id: &str, keep_alive: bool) -> Result<(), SidebarError> {
        let (_, session) = self.session(id)?;
        Ok(set_keep_alive(&self.sessions(), &session, keep_alive)?)
    }

    pub fn set_notify(&self, category_id: &str, notify: bool) -> Result<(), SidebarError> {
        Ok(set_notify(&self.categories(), category_id, notify)?)
    }

    pub fn set_collapsed(&self, category_id: &str, collapsed: bool) -> Result<(), SidebarError> {
        Ok(set_collapsed(&self.categories(), category_id, collapsed)?)
    }

    /// The list the session is ordered in (the pinned ones, or its
    /// category) and, of it, the cards shown together with the session:
    /// in a category, the coordinator is the header and the pinned ones are
    /// shown elsewhere.
    fn siblings(&self, session: &Session) -> Result<(Vec<String>, Vec<String>), SidebarError> {
        let list = if session.is_pinned() {
            self.sessions().list_pinned()?
        } else {
            self.sessions().list_in(&session.category_id)?
        };
        let shown = list
            .iter()
            .filter(|other| {
                session.is_pinned() || !(other.is_pinned() || other.kind == Kind::Coordinator)
            })
            .map(|other| other.id.clone())
            .collect();
        Ok((list.into_iter().map(|other| other.id).collect(), shown))
    }

    fn reorder(&self, session: &Session, order: &[String]) -> Result<(), SidebarError> {
        if session.is_pinned() {
            Ok(reorder_pins(&self.sessions(), order)?)
        } else {
            Ok(reorder_sessions(
                &self.sessions(),
                &session.category_id,
                order,
            )?)
        }
    }

    /// One place up (`-1`) or down (`1`) among the cards shown with it.
    pub fn shift_session(&self, id: &str, by: isize) -> Result<(), SidebarError> {
        let (_, session) = self.session(id)?;
        let (all, shown) = self.siblings(&session)?;
        self.reorder(
            &session,
            &with_shown_reordered(&all, &shifted(&shown, id, by)),
        )
    }

    /// Dragged onto another card of the same list: it takes that place.
    pub fn drop_session_on(&self, id: &str, target_id: &str) -> Result<(), SidebarError> {
        let (_, session) = self.session(id)?;
        let (all, shown) = self.siblings(&session)?;
        let order = dropped_on(&shown, id, target_id);
        self.reorder(&session, &with_shown_reordered(&all, &order))
    }

    fn category_ids(&self) -> Result<Vec<String>, SidebarError> {
        let categories = list_categories(&self.categories())?;
        Ok(categories.into_iter().map(|category| category.id).collect())
    }

    pub fn shift_category(&self, id: &str, by: isize) -> Result<(), SidebarError> {
        let order = shifted(&self.category_ids()?, id, by);
        Ok(reorder_categories(&self.categories(), &order)?)
    }

    pub fn drop_category_on(&self, id: &str, target_id: &str) -> Result<(), SidebarError> {
        let order = dropped_on(&self.category_ids()?, id, target_id);
        Ok(reorder_categories(&self.categories(), &order)?)
    }

    /// The Claude conversation is not touched.
    pub fn delete_session(&self, id: &str) -> Result<(), SidebarError> {
        let (_, session) = self.session(id)?;
        Ok(remove_session(&self.sessions(), &session)?)
    }

    /// Returns the sessions that went with it.
    pub fn delete_category(&self, id: &str) -> Result<Vec<Session>, SidebarError> {
        let category = self.category(id)?;
        let sessions = list_sessions(&self.sessions(), &category)?;
        remove_category(&self.categories(), &category.slug)?;
        Ok(sessions)
    }
}
