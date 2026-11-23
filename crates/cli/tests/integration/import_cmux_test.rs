//! PLAN §3.4: `lithemux import cmux`, from a synthetic file of cmux's format
//! where cmux keeps it under the sandbox's HOME. Never the user's real one.

use std::fs;
use std::path::{Path, PathBuf};

use crate::elsewhere::fake_claude_elsewhere;
use crate::sandbox::Sandbox;

const PLANNER: &str = "11111111-1111-4111-8111-111111111111";
const IN_HOME: &str = "22222222-2222-4222-8222-222222222222";
const BUILD: &str = "55555555-5555-4555-8555-555555555555";
/// The planner's conversation in the test that also runs it «in cmux»: the
/// check looks at every process of the machine, and tests run at once.
const PLANNER_IN_CMUX: &str = "99999999-1111-4111-8111-111111111111";

fn user(sandbox: &Sandbox) -> PathBuf {
    sandbox.dir.path().join("user")
}

/// Puts the core fixture where cmux keeps its file, with its folders moved
/// into the sandbox's home and the planner's conversation being `planner`;
/// Claude Code keeps the conversations of the planner and of BUILD.
fn with_cmux(sandbox: &Sandbox, planner: &str) -> PathBuf {
    let home = user(sandbox);
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests/fixtures/cmux/sessions-v1.json");
    let text = fs::read_to_string(fixture)
        .unwrap()
        .replace("/Users/someone", home.to_str().unwrap())
        .replace(PLANNER, planner);
    let place = home.join("Library/Application Support/cmux");
    fs::create_dir_all(&place).unwrap();
    let file = place.join("session-com.cmuxterm.app.json");
    fs::write(&file, text).unwrap();
    for folder in ["work/alpha/api", "work/beta"] {
        fs::create_dir_all(home.join(folder)).unwrap();
    }
    let project = sandbox.claude_config().join("projects/-somewhere");
    fs::create_dir_all(&project).unwrap();
    for id in [planner, BUILD] {
        fs::write(project.join(format!("{id}.jsonl")), "{}\n").unwrap();
    }
    file
}

fn arg_after(log: &[String], flag: &str) -> Option<String> {
    let position = log.iter().position(|line| line == &format!("arg={flag}"))?;
    log.get(position + 1)
        .map(|line| line.trim_start_matches("arg=").to_string())
}

// PLAN §3.4: `import cmux` muestra la propuesta y no escribe
#[test]
fn the_proposal_is_shown_and_nothing_is_written() {
    let sandbox = Sandbox::new().with_demo();
    let file = with_cmux(&sandbox, PLANNER);
    let before = fs::read(&file).unwrap();
    let status_before = sandbox.run(&["status", "--json"]);

    let shown = sandbox.run(&["import", "cmux"]);

    assert!(shown.starts_with(
        "cmux: 3 Claude sessions to import (0 already in lithemux, 1 without a conversation id, 1 without Claude)"
    ), "{shown}");
    assert!(shown.contains("  1  yes  Planner"), "{shown}");
    assert!(shown.contains("(no category: use --into)"), "{shown}");
    assert!(shown.contains("no saved conversation"), "{shown}");
    assert!(shown.contains("api (new)"), "{shown}");
    assert!(shown.contains("Nothing was written."), "{shown}");
    assert_eq!(sandbox.run(&["status", "--json"]), status_before);
    assert_eq!(fs::read(&file).unwrap(), before);
}

// PLAN §3.4: `--apply` la aplica; `--into` y `--include` eligen; la primera
// apertura usa --resume
#[test]
fn applying_imports_hibernated_sessions_that_resume_their_conversation() {
    let sandbox = Sandbox::new().with_demo();
    with_cmux(&sandbox, PLANNER);
    sandbox.use_fake_claude();

    let said = sandbox.run(&[
        "import",
        "cmux",
        "--apply",
        "--include",
        "2",
        "--into",
        "demo",
    ]);

    assert_eq!(
        said.lines().next(),
        Some(
            "Imported 3 sessions and 2 new categories, hibernated: opening one resumes its conversation."
        )
    );
    let status = sandbox.run(&["status", "--json"]);
    for id in [PLANNER, IN_HOME, BUILD] {
        assert!(status.contains(id), "{id} missing in {status}");
    }
    sandbox.run(&["open", "api/planner"]);
    let log = sandbox.fake_claude_log();
    assert_eq!(arg_after(&log, "--resume").as_deref(), Some(PLANNER));
    assert_eq!(
        log[0],
        format!(
            "cwd={}",
            user(&sandbox)
                .join("work/alpha/api")
                .canonicalize()
                .unwrap()
                .display()
        )
    );

    let again = sandbox.run(&["import", "cmux"]);
    assert!(
        again.starts_with("cmux: 0 Claude sessions to import (3 already in lithemux"),
        "{again}"
    );
}

// PLAN §3.4: a chosen session without a category stops the import whole
#[test]
fn a_chosen_session_without_a_category_stops_the_whole_import() {
    let sandbox = Sandbox::new().with_demo();
    with_cmux(&sandbox, PLANNER);
    let status_before = sandbox.run(&["status", "--json"]);

    let refused = sandbox.fail(&["import", "cmux", "--apply", "--include", "2"]);

    assert!(
        refused.contains("session 2 without a category"),
        "{refused}"
    );
    assert_eq!(sandbox.run(&["status", "--json"]), status_before);
}

// PLAN §3.4 riesgo real: si la sesión sigue viva en cmux, lithemux no la
// reanuda: está abierta en otro lado (pid N)
#[test]
fn a_session_still_running_in_cmux_is_not_opened_a_second_time() {
    let sandbox = Sandbox::new().with_demo();
    with_cmux(&sandbox, PLANNER_IN_CMUX);
    sandbox.use_fake_claude();
    sandbox.run(&["import", "cmux", "--apply", "--skip", "5"]);
    let cmux = tempfile::tempdir().unwrap();
    let mut in_cmux = fake_claude_elsewhere(cmux.path(), &["--resume", PLANNER_IN_CMUX]);

    let refused = sandbox.fail(&["open", "api/planner"]);

    in_cmux.kill().unwrap();
    in_cmux.wait().unwrap();
    assert!(
        refused.contains(&format!(
            "the session is already open (pid {})",
            in_cmux.id()
        )),
        "{refused}"
    );
    assert!(!sandbox.home().join("fake-claude.log").exists());
}
