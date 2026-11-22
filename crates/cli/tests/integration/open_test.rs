use rustix::process::Signal;
use std::fs;

use crate::sandbox::Sandbox;
use crate::signals;

fn arg_after(log: &[String], flag: &str) -> Option<String> {
    let position = log.iter().position(|line| line == &format!("arg={flag}"))?;
    log.get(position + 1)
        .map(|line| line.trim_start_matches("arg=").to_string())
}

// PLAN §0.6 `open` + §0.3: the session is launched with its name, hooks and environment
#[test]
fn open_launches_claude_with_the_session_name_hooks_and_environment() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();

    sandbox.run(&["open", "demo/ejecutora"]);

    let log = sandbox.fake_claude_log();
    assert_eq!(log[0], format!("cwd={}", sandbox.work().display()));
    assert!(log[1].starts_with("session_env=") && log[1].len() > "session_env=".len() + 30);
    assert_eq!(log[2], format!("home_env={}", sandbox.home().display()));
    assert_eq!(log[3], "arg=--session-id");
    assert_eq!(arg_after(&log, "-n").as_deref(), Some("demo-ejecutora"));
    let settings = arg_after(&log, "--settings").unwrap();
    for event in [
        "SessionStart",
        "UserPromptSubmit",
        "Stop",
        "Notification",
        "SessionEnd",
        "PostToolUse",
        "PostToolUseFailure",
    ] {
        assert!(settings.contains(event), "{event} missing in {settings}");
    }
    assert!(settings.contains("lithemux-hook session-start"));
    assert!(!log.iter().any(|line| line == "arg=--model"));
    // `open` stored the version reported by `claude --version`.
    assert!(
        sandbox
            .run(&["status", "--json"])
            .contains("\"claude_version\": \"9.9.9\"")
    );
}

// PLAN §0.3 + acceptance 2: reopening resumes the same conversation and keeps the name
#[test]
fn reopening_resumes_the_same_conversation_with_the_same_name() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    sandbox.run(&["open", "demo/ejecutora"]);
    let first = sandbox.fake_claude_log();
    let conversation = arg_after(&first, "--session-id").unwrap();

    // What Claude Code does on a real first run: it writes the transcript.
    let project = sandbox.claude_config().join("projects/-work");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(format!("{conversation}.jsonl")), "{}\n").unwrap();

    sandbox.run(&["open", "demo/ejecutora"]);

    let second = sandbox.fake_claude_log();
    assert_eq!(arg_after(&second, "--resume"), Some(conversation));
    assert_eq!(arg_after(&second, "--session-id"), None);
    assert_eq!(arg_after(&second, "-n").as_deref(), Some("demo-ejecutora"));
}

// PLAN §0.3: --model when the session defines one
#[test]
fn open_passes_the_model_of_the_session() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    sandbox.run(&["set", "demo/ejecutora", "--model", "opus"]);
    sandbox.run(&["open", "demo/ejecutora"]);
    assert_eq!(
        arg_after(&sandbox.fake_claude_log(), "--model").as_deref(),
        Some("opus")
    );
}

// docs/decisiones.md: spawn + wait, `open` exits with the child's exit code
#[test]
fn open_exits_with_the_exit_code_of_claude() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    fs::write(sandbox.home().join("fake-claude.exit"), "7").unwrap();
    let output = sandbox
        .lithemux(&["open", "demo/ejecutora"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
}

// PLAN §0.3 + acceptance 2: opening the same session twice fails because of the lock
#[test]
fn opening_the_same_session_twice_fails_while_the_first_is_alive() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    fs::write(sandbox.home().join("fake-claude.sleep"), "5").unwrap();

    let mut first = sandbox.open_in_background("demo/ejecutora");

    let error = sandbox.fail(&["open", "demo/ejecutora"]);
    assert!(error.contains("already open"), "{error}");
    assert!(error.contains("(pid "), "{error}");
    // A different session of the same category is not affected.
    let status = sandbox.run(&["status"]);
    let executor = status
        .lines()
        .find(|line| line.contains("demo-ejecutora"))
        .unwrap();
    assert!(executor.contains("open (pid "), "{executor}");

    // Ending the first one the way a terminal does frees the session.
    signals::send(first.id(), Signal::TERM);
    first.wait().unwrap();
    signals::wait_until_gone(sandbox.fake_claude_number("fake-claude.pid"));
    fs::remove_file(sandbox.home().join("fake-claude.sleep")).unwrap();
    sandbox.run(&["open", "demo/ejecutora"]);
}

// PLAN §0.3: shell profile runs `$SHELL -l`, no claude involved
#[test]
fn a_shell_session_opens_the_login_shell() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.run(&[
        "new",
        "demo/term",
        "--role",
        "Terminal",
        "--profile",
        "shell",
    ]);
    // No claude_path is configured: a shell session must not need claude.
    let output = sandbox
        .lithemux(&["open", "demo/term"])
        .write_stdin("exit 3\n")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(!sandbox.home().join("fake-claude.log").exists());
}

// PLAN §0.3: without any claude the error says so
#[test]
fn open_explains_when_claude_cannot_be_found() {
    let sandbox = Sandbox::new().with_demo();
    let error = sandbox.fail(&["open", "demo/ejecutora"]);
    assert!(error.contains("could not find `claude`"), "{error}");
}

// PLAN §0.6
#[test]
fn open_fails_for_a_session_that_does_not_exist() {
    let sandbox = Sandbox::new().with_demo();
    assert!(sandbox.fail(&["open", "demo/nadie"]).contains("not found"));
}

// PLAN §2.0 bug 7: the child runs in another directory, so a relative
// LITHEMUX_HOME would point somewhere else for it and for its hooks
#[test]
fn a_relative_lithemux_home_reaches_the_child_as_an_absolute_path() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();

    let output = sandbox
        .lithemux(&["open", "demo/ejecutora"])
        .current_dir(sandbox.dir.path())
        .env("LITHEMUX_HOME", "home")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log = sandbox.fake_claude_log();
    let home = log[2].trim_start_matches("home_env=");
    assert!(std::path::Path::new(home).is_absolute(), "{home}");
    assert_eq!(
        std::fs::canonicalize(home).unwrap(),
        sandbox.home().canonicalize().unwrap()
    );
}
