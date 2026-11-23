//! A row of `sessions` as a `Session`, and what SQLite's refusals mean.

use rusqlite::Row;
use rusqlite::types::Type;

use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::state::State;

pub(super) fn from_row(row: &Row<'_>) -> rusqlite::Result<Session> {
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

/// The index of migration 6 refused it: another session has that
/// conversation.
pub(super) fn takes_a_conversation(error: &rusqlite::Error) -> bool {
    error.sqlite_error_code() == Some(rusqlite::ErrorCode::ConstraintViolation)
        && error.to_string().contains("sessions.claude_session_id")
}
