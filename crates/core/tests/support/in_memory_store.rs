use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;

use lithemux_core::modules::category::{BoardStore, Category, CategoryError, CategoryRepository};
use lithemux_core::modules::session::{Kind, Session, SessionError, SessionRepository, State};
use lithemux_core::modules::settings::{SettingsError, SettingsRepository};

// Same contract as the SQLite repositories: tests/support/contracts runs
// against both.
#[derive(Default)]
pub struct InMemoryStore {
    categories: RefCell<Vec<Category>>,
    sessions: RefCell<Vec<Session>>,
    settings: RefCell<BTreeMap<String, String>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn update(&self, id: &str, change: impl FnOnce(&mut Session)) -> Result<(), SessionError> {
        if let Some(session) = self.sessions.borrow_mut().iter_mut().find(|s| s.id == id) {
            change(session);
        }
        Ok(())
    }
}

impl CategoryRepository for InMemoryStore {
    fn insert(&self, category: &Category) -> Result<(), CategoryError> {
        self.categories.borrow_mut().push(category.clone());
        Ok(())
    }

    fn next_sort(&self) -> Result<i64, CategoryError> {
        Ok(self
            .categories
            .borrow()
            .iter()
            .map(|c| c.sort)
            .max()
            .unwrap_or(0)
            + 1)
    }

    fn list(&self) -> Result<Vec<Category>, CategoryError> {
        let mut all = self.categories.borrow().clone();
        all.sort_by_key(|c| (c.sort, c.created_at, c.slug.clone()));
        Ok(all)
    }

    fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, CategoryError> {
        Ok(self
            .categories
            .borrow()
            .iter()
            .find(|c| c.slug == slug)
            .cloned())
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Category>, CategoryError> {
        Ok(self
            .categories
            .borrow()
            .iter()
            .find(|c| c.id == id)
            .cloned())
    }

    fn delete(&self, id: &str) -> Result<(), CategoryError> {
        self.categories.borrow_mut().retain(|c| c.id != id);
        self.sessions.borrow_mut().retain(|s| s.category_id != id);
        Ok(())
    }
}

impl SessionRepository for InMemoryStore {
    fn insert(&self, session: &Session) -> Result<(), SessionError> {
        self.sessions.borrow_mut().push(session.clone());
        Ok(())
    }

    fn next_sort(&self, category_id: &str) -> Result<i64, SessionError> {
        let sessions = self.sessions.borrow();
        let in_category = sessions.iter().filter(|s| s.category_id == category_id);
        Ok(in_category.map(|s| s.sort).max().unwrap_or(0) + 1)
    }

    fn list_in(&self, category_id: &str) -> Result<Vec<Session>, SessionError> {
        let mut all: Vec<Session> = self
            .sessions
            .borrow()
            .iter()
            .filter(|s| s.category_id == category_id)
            .cloned()
            .collect();
        all.sort_by_key(|s| (s.sort, s.created_at, s.slug.clone()));
        Ok(all)
    }

    fn find(&self, category_id: &str, slug: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.borrow();
        let mut matching = sessions
            .iter()
            .filter(|s| s.category_id == category_id && s.slug == slug);
        Ok(matching.next().cloned())
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError> {
        Ok(self.sessions.borrow().iter().find(|s| s.id == id).cloned())
    }

    fn find_mother(&self, category_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.borrow();
        let mut mothers = sessions
            .iter()
            .filter(|s| s.category_id == category_id && s.kind == Kind::Mother);
        Ok(mothers.next().cloned())
    }

    fn update_role(&self, id: &str, role: &str) -> Result<(), SessionError> {
        self.update(id, |s| s.role = role.to_string())
    }

    fn update_kind(&self, id: &str, kind: Kind) -> Result<(), SessionError> {
        self.update(id, |s| s.kind = kind)
    }

    fn update_keep_alive(&self, id: &str, keep_alive: bool) -> Result<(), SessionError> {
        self.update(id, |s| s.keep_alive = keep_alive)
    }

    fn update_model(&self, id: &str, model: Option<&str>) -> Result<(), SessionError> {
        self.update(id, |s| s.model = model.map(str::to_string))
    }

    fn update_claude_version(&self, id: &str, version: &str) -> Result<(), SessionError> {
        self.update(id, |s| s.claude_version = Some(version.to_string()))
    }

    fn update_claude_session_id(&self, id: &str, claude_id: &str) -> Result<(), SessionError> {
        self.update(id, |s| s.claude_session_id = Some(claude_id.to_string()))
    }

    fn update_state(
        &self,
        id: &str,
        state: State,
        last_activity_at: Option<i64>,
        unless: Option<State>,
    ) -> Result<(), SessionError> {
        self.update(id, |s| {
            if Some(s.state) == unless {
                return;
            }
            s.state = state;
            if last_activity_at.is_some() {
                s.last_activity_at = last_activity_at;
            }
        })
    }

    fn touch_activity(&self, id: &str, at: i64) -> Result<(), SessionError> {
        self.update(id, |s| s.last_activity_at = Some(at))
    }

    fn move_state(&self, id: &str, from: State, to: State) -> Result<(), SessionError> {
        self.update(id, |s| {
            if s.state == from {
                s.state = to;
            }
        })
    }

    fn delete(&self, id: &str) -> Result<(), SessionError> {
        self.sessions.borrow_mut().retain(|s| s.id != id);
        Ok(())
    }
}

impl SettingsRepository for InMemoryStore {
    fn get(&self, key: &str) -> Result<Option<String>, SettingsError> {
        Ok(self.settings.borrow().get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), SettingsError> {
        self.settings
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), SettingsError> {
        self.settings.borrow_mut().remove(key);
        Ok(())
    }
}

// Same contract as FsBoardStore: an existing board is never overwritten.
#[derive(Default)]
pub struct InMemoryBoards {
    pub created: RefCell<Vec<String>>,
}

impl BoardStore for InMemoryBoards {
    fn path(&self, category_slug: &str) -> PathBuf {
        PathBuf::from(format!("/lithemux/categories/{category_slug}/board.md"))
    }

    fn ensure(&self, category: &Category) -> Result<(), CategoryError> {
        let mut created = self.created.borrow_mut();
        if !created.contains(&category.slug) {
            created.push(category.slug.clone());
        }
        Ok(())
    }
}
