// PLAN §3.6: «errores significativos del host (... migraciones)». And
// correction round of 3A (9): a file that was there but could not be read
// was not created.
use crate::session_host::startup::{Found, migration_note};

#[test]
fn a_new_database_and_a_migrated_one_are_noted() {
    assert_eq!(
        migration_note(Found::Nothing, Some(5)).as_deref(),
        Some("database created at schema v5")
    );
    assert_eq!(
        migration_note(Found::Schema(4), Some(5)).as_deref(),
        Some("database migrated from schema v4 to v5")
    );
}

#[test]
fn an_unchanged_or_unreadable_database_is_not_noted() {
    assert_eq!(migration_note(Found::Schema(5), Some(5)), None);
    assert_eq!(migration_note(Found::Unreadable, Some(5)), None);
    assert_eq!(migration_note(Found::Nothing, None), None);
}
