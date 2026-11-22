use rusqlite::types::Type;
use rusqlite::{OptionalExtension, Params, Row, params};

use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::state::State;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;
use crate::platform::store::connection::Db;
use crate::platform::store::error::StoreError;

const COLUMNS: &str = "id, category_id, slug, role, kind, profile, cwd, claude_session_id, \
     model, keep_alive, state, claude_version, last_activity_at, created_at, sort, \
     title, pin_sort, preview, unread, notice";

pub struct SqliteSessionRepository<'a> {
    db: &'a Db,
}

impl<'a> SqliteSessionRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        SqliteSessionRepository { db }
    }

    fn execute(&self, sql: &str, params: impl Params) -> Result<(), SessionError> {
        self.db
            .conn()
            .execute(sql, params)
            .map_err(StoreError::from)?;
        Ok(())
    }

    fn find_one(&self, filter: &str, params: impl Params) -> Result<Option<Session>, SessionError> {
        Ok(self
            .db
            .conn()
            .query_row(
                &format!("SELECT {COLUMNS} FROM sessions WHERE {filter}"),
                params,
                from_row,
            )
            .optional()
            .map_err(StoreError::from)?)
    }
}

impl SqliteSessionRepository<'_> {
    /// `rest` is what follows `FROM sessions`: the filter and the order.
    fn list(&self, rest: &str, params: impl Params) -> Result<Vec<Session>, SessionError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(&format!("SELECT {COLUMNS} FROM sessions {rest}"))
            .map_err(StoreError::from)?;
        let rows = stmt.query_map(params, from_row).map_err(StoreError::from)?;
        Ok(rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(StoreError::from)?)
    }
}

impl SessionRepository for SqliteSessionRepository<'_> {
    fn insert(&self, session: &Session) -> Result<(), SessionError> {
        self.execute(
            "INSERT INTO sessions(id, category_id, slug, role, kind, profile, cwd, \
             claude_session_id, model, keep_alive, state, claude_version, last_activity_at, \
             created_at, sort, title, pin_sort, preview, unread, notice) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, \
             ?16, ?17, ?18, ?19, ?20)",
            params![
                session.id,
                session.category_id,
                session.slug,
                session.role,
                session.kind.as_str(),
                session.profile.as_str(),
                session.cwd,
                session.claude_session_id,
                session.model,
                session.keep_alive,
                session.state.as_str(),
                session.claude_version,
                session.last_activity_at,
                session.created_at,
                session.sort,
                session.title,
                session.pin_sort,
                session.preview,
                session.unread,
                session.notice
            ],
        )
    }

    fn next_sort(&self, category_id: &str) -> Result<i64, SessionError> {
        Ok(self
            .db
            .conn()
            .query_row(
                "SELECT COALESCE(MAX(sort), 0) + 1 FROM sessions WHERE category_id = ?1",
                [category_id],
                |row| row.get(0),
            )
            .map_err(StoreError::from)?)
    }

    fn list_in(&self, category_id: &str) -> Result<Vec<Session>, SessionError> {
        self.list(
            "WHERE category_id = ?1 ORDER BY sort, created_at, slug",
            [category_id],
        )
    }

    fn list_all(&self) -> Result<Vec<Session>, SessionError> {
        self.list("ORDER BY category_id, sort, created_at, slug", [])
    }

    fn find(&self, category_id: &str, slug: &str) -> Result<Option<Session>, SessionError> {
        self.find_one("category_id = ?1 AND slug = ?2", params![category_id, slug])
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError> {
        self.find_one("id = ?1", [id])
    }

    fn find_coordinator(&self, category_id: &str) -> Result<Option<Session>, SessionError> {
        self.find_one("category_id = ?1 AND kind = 'coordinator'", [category_id])
    }

    fn update_role(&self, id: &str, role: &str) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET role = ?2 WHERE id = ?1",
            params![id, role],
        )
    }

    fn update_kind(&self, id: &str, kind: Kind) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET kind = ?2 WHERE id = ?1",
            params![id, kind.as_str()],
        )
    }

    fn update_keep_alive(&self, id: &str, keep_alive: bool) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET keep_alive = ?2 WHERE id = ?1",
            params![id, keep_alive],
        )
    }

    fn update_model(&self, id: &str, model: Option<&str>) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET model = ?2 WHERE id = ?1",
            params![id, model],
        )
    }

    fn update_claude_version(&self, id: &str, version: &str) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET claude_version = ?2 WHERE id = ?1",
            params![id, version],
        )
    }

    fn update_claude_session_id(&self, id: &str, claude_id: &str) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET claude_session_id = ?2 WHERE id = ?1",
            params![id, claude_id],
        )
    }

    fn update_state(
        &self,
        id: &str,
        state: State,
        last_activity_at: Option<i64>,
        unless: Option<State>,
    ) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET state = ?2, last_activity_at = COALESCE(?3, last_activity_at) \
             WHERE id = ?1 AND (?4 IS NULL OR state != ?4)",
            params![
                id,
                state.as_str(),
                last_activity_at,
                unless.map(State::as_str)
            ],
        )
    }

    fn touch_activity(&self, id: &str, at: i64) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET last_activity_at = ?2 WHERE id = ?1",
            params![id, at],
        )
    }

    fn move_state(&self, id: &str, from: State, to: State) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET state = ?3 WHERE id = ?1 AND state = ?2",
            params![id, from.as_str(), to.as_str()],
        )
    }

    fn update_title(&self, id: &str, title: Option<&str>) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET title = ?2 WHERE id = ?1",
            params![id, title],
        )
    }

    fn update_sort(&self, id: &str, sort: i64) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET sort = ?2 WHERE id = ?1",
            params![id, sort],
        )
    }

    fn update_pin_sort(&self, id: &str, pin_sort: Option<i64>) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET pin_sort = ?2 WHERE id = ?1",
            params![id, pin_sort],
        )
    }

    fn list_pinned(&self) -> Result<Vec<Session>, SessionError> {
        self.list(
            "WHERE pin_sort IS NOT NULL ORDER BY pin_sort, created_at, slug",
            [],
        )
    }

    fn note_message(
        &self,
        id: &str,
        preview: Option<&str>,
        asks_for_attention: bool,
    ) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET preview = COALESCE(?2, preview), unread = unread + ?3 \
             WHERE id = ?1",
            params![id, preview, i64::from(asks_for_attention)],
        )
    }

    fn update_notice(&self, id: &str, notice: Option<&str>) -> Result<(), SessionError> {
        self.execute(
            "UPDATE sessions SET notice = ?2 WHERE id = ?1",
            params![id, notice],
        )
    }

    fn clear_unread(&self, id: &str) -> Result<(), SessionError> {
        self.execute("UPDATE sessions SET unread = 0 WHERE id = ?1", [id])
    }

    fn delete(&self, id: &str) -> Result<(), SessionError> {
        self.execute("DELETE FROM sessions WHERE id = ?1", [id])
    }
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        category_id: row.get(1)?,
        slug: row.get(2)?,
        role: row.get(3)?,
        kind: parsed(row, 4, Kind::parse)?,
        profile: parsed(row, 5, Profile::parse)?,
        cwd: row.get(6)?,
        claude_session_id: row.get(7)?,
        model: row.get(8)?,
        keep_alive: row.get(9)?,
        state: parsed(row, 10, State::parse)?,
        claude_version: row.get(11)?,
        last_activity_at: row.get(12)?,
        created_at: row.get::<_, Option<i64>>(13)?.unwrap_or(0),
        sort: row.get::<_, Option<i64>>(14)?.unwrap_or(0),
        title: row.get(15)?,
        pin_sort: row.get(16)?,
        preview: row.get(17)?,
        unread: row.get(18)?,
        notice: row.get(19)?,
    })
}

fn parsed<T>(row: &Row<'_>, index: usize, parse: fn(&str) -> Option<T>) -> rusqlite::Result<T> {
    let text: String = row.get(index)?;
    parse(&text).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            Type::Text,
            format!("unexpected value \"{text}\"").into(),
        )
    })
}
