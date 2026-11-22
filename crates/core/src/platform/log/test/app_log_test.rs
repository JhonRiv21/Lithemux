use std::fs;

use crate::platform::log::app_log::{AppLog, Level};

fn log_in(dir: &tempfile::TempDir, max_bytes: u64) -> AppLog {
    AppLog::new(dir.path().join("logs").join("app.log"), max_bytes, 3)
}

fn read(dir: &tempfile::TempDir, name: &str) -> String {
    fs::read_to_string(dir.path().join("logs").join(name)).unwrap_or_default()
}

// PLAN §3.6: the app writes to <home>/logs/app.log, with level and time.
#[test]
fn a_line_has_its_time_its_level_and_its_text() {
    let dir = tempfile::tempdir().unwrap();
    let log = log_in(&dir, 1024 * 1024);
    log.write(
        Level::Error,
        "could not start demo/api: claude not found",
        1_800_000_000,
    );
    log.write(
        Level::Info,
        "database migrated from v3 to v5",
        1_800_000_001,
    );
    assert_eq!(
        read(&dir, "app.log"),
        "2027-01-15T08:00:00Z ERROR could not start demo/api: claude not found\n\
         2027-01-15T08:00:01Z INFO database migrated from v3 to v5\n"
    );
}

// A message is one line: what it quotes cannot add lines of its own.
#[test]
fn a_message_stays_on_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let log = log_in(&dir, 1024 * 1024);
    log.write(Level::Warn, "first\nsecond\r\nthird", 0);
    assert_eq!(
        read(&dir, "app.log"),
        "1970-01-01T00:00:00Z WARN first second third\n"
    );
}

// PLAN §3.6: rotated. A full log moves aside, and only the last few stay.
#[test]
fn a_full_log_is_rotated_and_only_three_files_are_kept() {
    let dir = tempfile::tempdir().unwrap();
    let log = log_in(&dir, 100);
    for n in 0..20 {
        log.write(
            Level::Info,
            &format!("line number {n:02} with some padding"),
            0,
        );
    }
    let names: Vec<String> = {
        let mut names: Vec<String> = fs::read_dir(dir.path().join("logs"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    };
    assert_eq!(names, ["app.log", "app.log.1", "app.log.2"]);
    for name in &names {
        let size = fs::metadata(dir.path().join("logs").join(name))
            .unwrap()
            .len();
        assert!(size <= 100 + 64, "{name}: {size}");
    }
    // The newest line is in the current file; the oldest ones are gone.
    assert!(read(&dir, "app.log").contains("line number 19"));
    let everything = names.iter().map(|n| read(&dir, n)).collect::<String>();
    assert!(!everything.contains("line number 00"));
}

// A log that cannot be written never stops what was being logged.
#[test]
fn a_log_that_cannot_be_written_is_silent() {
    let dir = tempfile::tempdir().unwrap();
    let blocked = dir.path().join("a-file");
    fs::write(&blocked, "").unwrap();
    let log = AppLog::new(blocked.join("logs").join("app.log"), 100, 3);
    log.write(Level::Error, "nowhere to go", 0);
}

// Correction round of 3A (7): one writer per file in the process, so the
// panic hook and the host never rotate the same file under each other.
#[test]
fn the_log_of_a_file_is_one_and_the_same_for_the_whole_process() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");
    let first = AppLog::shared(path.clone());
    let second = AppLog::shared(path);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    let other = AppLog::shared(dir.path().join("other.log"));
    assert!(!std::sync::Arc::ptr_eq(&first, &other));
}
