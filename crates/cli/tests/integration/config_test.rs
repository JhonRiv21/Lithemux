use crate::sandbox::{Sandbox, fake_claude};

// PLAN §0.6: config get|set|unset
#[test]
fn config_sets_reads_and_unsets_the_two_known_keys() {
    let sandbox = Sandbox::new();
    assert!(
        sandbox
            .run(&["config", "get", "context_language"])
            .contains("not set")
    );
    sandbox.run(&["config", "set", "context_language", "es"]);
    assert_eq!(
        sandbox.run(&["config", "get", "context_language"]).trim(),
        "es"
    );
    sandbox.run(&["config", "unset", "context_language"]);
    assert!(
        sandbox
            .run(&["config", "get", "context_language"])
            .contains("not set")
    );

    sandbox.run(&["config", "set", "claude_path", &fake_claude()]);
    assert_eq!(
        sandbox.run(&["config", "get", "claude_path"]).trim(),
        fake_claude()
    );
}

// PLAN §0.6: other keys are rejected with the list of valid ones
#[test]
fn config_rejects_unknown_keys_and_invalid_values() {
    let sandbox = Sandbox::new();
    let unknown = sandbox.fail(&["config", "set", "telemetry", "on"]);
    assert!(unknown.contains("claude_path") && unknown.contains("context_language"));
    assert!(
        sandbox
            .fail(&["config", "set", "context_language", "fr"])
            .contains("es")
    );
    assert!(
        sandbox
            .fail(&["config", "set", "claude_path", "not-absolute"])
            .contains("absolute")
    );
}

// PLAN §0.5: context_language decides the language of the injected context
#[test]
fn context_language_changes_the_language_of_the_context() {
    let sandbox = Sandbox::new().with_demo();
    assert!(
        sandbox
            .run(&["context", "demo/ejecutora"])
            .contains("This session's name")
    );
    sandbox.run(&["config", "set", "context_language", "es"]);
    assert!(
        sandbox
            .run(&["context", "demo/ejecutora"])
            .contains("Nombre de esta sesión")
    );
}

// PLAN §0.6 doctor: path and version of claude, database, hook dry run
#[test]
fn doctor_reports_the_claude_it_would_use() {
    let sandbox = Sandbox::new();
    sandbox.use_fake_claude();
    let output = sandbox.lithemux(&["doctor"]).output().unwrap();
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains(&fake_claude()), "{report}");
    assert!(report.contains("9.9.9"), "{report}");
    assert!(report.contains("used by lithemux"), "{report}");
    assert!(report.contains("schema v6"), "{report}");
    assert!(report.contains("data directory"), "{report}");
}

// docs/decisiones.md D-28: the claude that lithemux remembers can be seen
// and forgotten, but only lithemux writes it
#[test]
fn the_remembered_claude_can_be_read_and_cleared_but_not_set() {
    // A `claude` in a temporary folder is never used, so never remembered.
    if !Sandbox::fakes_on_the_path_are_launchable() {
        return;
    }
    let sandbox = Sandbox::new().with_fake_shell("fake-shell-lingering");
    sandbox.lithemux(&["doctor"]).output().unwrap();

    let doctor = sandbox.lithemux(&["doctor"]).output().unwrap();
    let doctor = String::from_utf8_lossy(&doctor.stdout).to_string();
    let remembered = sandbox.run(&["config", "get", "last_claude_path"]);
    assert!(remembered.contains("fakes/on-path/"), "{remembered}");
    assert!(doctor.contains("claude remembered"), "{doctor}");

    let refused = sandbox.fail(&["config", "set", "last_claude_path", "/bin/anything"]);
    assert!(refused.contains("only be read or unset"), "{refused}");

    sandbox.run(&["config", "unset", "last_claude_path"]);
    let cleared = sandbox.run(&["config", "get", "last_claude_path"]);
    assert!(!cleared.contains("on-path"), "{cleared}");
}
