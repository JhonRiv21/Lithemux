use std::ffi::OsString;
use std::path::Path;

use crate::terminal_view::pty::{GridSize, PtyProcess, ShellCommand};

fn fake_program(mode: &str) -> ShellCommand {
    let fakes = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fakes");
    let mut command = ShellCommand::new(fakes.join("fake-program"));
    command.args.push(mode.to_string());
    command
}

/// What the fake program printed on its terminal before it exited.
fn printed_by(mode: &str, inherited: &[(&str, &str)]) -> String {
    printed_by_command(&fake_program(mode), inherited)
}

fn printed_by_command(command: &ShellCommand, inherited: &[(&str, &str)]) -> String {
    let inherited = inherited
        .iter()
        .map(|(name, value)| (OsString::from(name), OsString::from(value)))
        .collect::<Vec<_>>();

    let (_process, mut streams) = PtyProcess::spawn_in(
        command,
        GridSize::INITIAL,
        inherited.into_iter(),
        Box::new(|| {}),
    )
    .unwrap();

    let mut printed = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !printed.ends_with(b"\n") {
        match streams.reader.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read) => printed.extend_from_slice(&chunk[..read]),
        }
    }
    String::from_utf8_lossy(&printed).trim().to_string()
}

// PLAN §2.0 point 19 (audit I4): the program in a terminal of the app does
// not see the marks of the session the app was started from
#[test]
fn the_program_in_a_terminal_starts_without_the_marks_of_the_parent_session() {
    let printed = printed_by(
        "inherited-marks",
        &[
            ("PATH", "/usr/bin:/bin"),
            // An existing folder: the terminal starts in the user's home.
            ("HOME", env!("CARGO_MANIFEST_DIR")),
            ("CLAUDECODE", "1"),
            ("CLAUDE_CODE_CHILD_SESSION", "1"),
            ("CMUX_SURFACE_ID", "abc"),
        ],
    );

    assert_eq!(
        printed,
        format!("marks:0 home:{}", env!("CARGO_MANIFEST_DIR"))
    );
}

const NOWHERE: &str = "/nonexistent/lithemux-test-home";

fn real(path: &Path) -> String {
    path.canonicalize().unwrap().display().to_string()
}

// A terminal always starts somewhere that exists: a HOME that is gone must
// not keep it from starting
#[test]
fn a_home_that_does_not_exist_starts_the_terminal_at_the_root() {
    let printed = printed_by(
        "working-directory",
        &[("PATH", "/usr/bin:/bin"), ("HOME", NOWHERE)],
    );

    assert_eq!(printed, "cwd:/");
}

// The session's own directory comes first
#[test]
fn the_terminal_starts_in_the_directory_of_the_session() {
    let directory = tempfile::tempdir().unwrap();
    let mut command = fake_program("working-directory");
    command.cwd = Some(directory.path().to_path_buf());

    let printed = printed_by_command(&command, &[("PATH", "/usr/bin:/bin"), ("HOME", NOWHERE)]);

    assert_eq!(printed, format!("cwd:{}", real(directory.path())));
}

// A session directory that was deleted gives way to the user's home
#[test]
fn a_session_directory_that_is_gone_gives_way_to_the_home() {
    let home = tempfile::tempdir().unwrap();
    let mut command = fake_program("working-directory");
    command.cwd = Some(NOWHERE.into());
    let home_path = home.path().display().to_string();

    let printed = printed_by_command(&command, &[("PATH", "/usr/bin:/bin"), ("HOME", &home_path)]);

    assert_eq!(printed, format!("cwd:{}", real(home.path())));
}
