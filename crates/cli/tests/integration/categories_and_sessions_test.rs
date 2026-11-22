use std::fs;

use crate::sandbox::Sandbox;

// PLAN §0.6: cat add / cat list
#[test]
fn a_category_is_created_listed_and_gets_its_board() {
    let sandbox = Sandbox::new();
    let work = sandbox.work();
    let created = sandbox.run(&[
        "cat",
        "add",
        "selektal",
        "--name",
        "Selektal",
        "--root",
        work.to_str().unwrap(),
    ]);
    assert!(created.contains("selektal"));
    let board = sandbox.home().join("categories/selektal/board.md");
    assert!(board.is_file());
    assert!(created.contains(board.to_str().unwrap()));

    let listed = sandbox.run(&["cat", "list"]);
    assert!(listed.contains("selektal"));
    assert!(listed.contains("Selektal"));
    assert!(listed.contains(work.to_str().unwrap()));
    assert!(
        sandbox
            .run(&["cat", "list", "--json"])
            .contains("\"slug\": \"selektal\"")
    );
}

// PLAN §0.2 + §0.6: invalid input is rejected with a reason
#[test]
fn invalid_slugs_missing_folders_and_duplicates_are_rejected() {
    let sandbox = Sandbox::new().with_demo();
    let work = sandbox.work();
    let work = work.to_str().unwrap();
    assert!(
        sandbox
            .fail(&["cat", "add", "Mal Slug", "--name", "x", "--root", work])
            .contains("invalid slug")
    );
    assert!(
        sandbox
            .fail(&[
                "cat",
                "add",
                "otra",
                "--name",
                "x",
                "--root",
                "/no/such/folder"
            ])
            .contains("not an existing directory")
    );
    assert!(
        sandbox
            .fail(&["cat", "add", "demo", "--name", "x", "--root", work])
            .contains("already exists")
    );
    assert!(
        sandbox
            .fail(&["new", "demo/ejecutora", "--role", "x"])
            .contains("already exists")
    );
    assert!(
        sandbox
            .fail(&["new", "nada/ejecutora", "--role", "x"])
            .contains("not found")
    );
    assert!(
        sandbox
            .fail(&["new", "ejecutora", "--role", "x"])
            .contains("<category>/<session>")
    );
}

// PLAN §0.2: one coordinator per category, from the command line
#[test]
fn a_second_coordinator_is_rejected() {
    let sandbox = Sandbox::new().with_demo();
    let error = sandbox.fail(&["new", "demo/jefa", "--role", "x", "--coordinator"]);
    assert!(error.contains("already has a coordinator"));
    assert!(error.contains("planificadora"));
    assert!(
        sandbox
            .fail(&["set", "demo/ejecutora", "--coordinator"])
            .contains("already has a coordinator")
    );
}

// PLAN §0.6: status shows the tree coordinator -> other sessions with state and activity
#[test]
fn status_shows_the_coordinator_first_and_the_other_sessions_below() {
    let sandbox = Sandbox::new().with_demo();
    let status = sandbox.run(&["status"]);
    let lines: Vec<&str> = status.lines().collect();
    assert!(lines[0].contains("Demo") && lines[0].contains("demo"));
    assert!(lines[1].contains("demo-planificadora"));
    assert!(lines[2].contains("demo-ejecutora"));
    assert!(lines[3].contains("demo-documental"));
    assert!(
        lines[2].find("demo-ejecutora") > lines[1].find("demo-planificadora"),
        "the other sessions are indented under the coordinator"
    );
    for line in &lines[1..4] {
        assert!(line.contains("new"), "{line}");
        assert!(line.contains("never"), "{line}");
    }
    assert!(sandbox.run(&["status", "demo"]).contains("demo-ejecutora"));
    assert!(sandbox.fail(&["status", "nada"]).contains("not found"));
}

// PLAN §0.6: --json where it applies
#[test]
fn status_json_carries_names_kinds_and_states() {
    let sandbox = Sandbox::new().with_demo();
    let json = sandbox.run(&["status", "--json"]);
    for expected in [
        "\"slug\": \"demo\"",
        "\"name\": \"demo-planificadora\"",
        "\"kind\": \"coordinator\"",
        "\"name\": \"demo-ejecutora\"",
        "\"kind\": \"member\"",
        "\"state\": \"new\"",
        "\"open_pid\": null",
    ] {
        assert!(json.contains(expected), "{expected} missing in {json}");
    }
}

// PLAN §0.6: `context` prints what the session would receive
#[test]
fn context_prints_what_the_session_would_receive() {
    let sandbox = Sandbox::new().with_demo();
    let context = sandbox.run(&["context", "demo/ejecutora"]);
    assert!(context.contains("This session's name: demo-ejecutora"));
    assert!(context.contains("Coordinator of the category: demo-planificadora"));
    assert!(context.contains("demo-documental"));
    let coordinator = sandbox.run(&["context", "demo/planificadora"]);
    assert!(coordinator.contains("coordinator session of the category"));
    assert!(coordinator.contains("demo-ejecutora"));
}

// PLAN §0.6: set --role|--coordinator|--keep-alive|--model
#[test]
fn set_changes_role_kind_keep_alive_and_model() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.run(&[
        "set",
        "demo/documental",
        "--role",
        "Documenta",
        "--keep-alive",
        "--model",
        "opus",
    ]);
    sandbox.run(&["set", "demo/planificadora", "--coordinator", "false"]);
    sandbox.run(&["set", "demo/documental", "--coordinator"]);
    let json = sandbox.run(&["status", "--json"]);
    assert!(json.contains("\"role\": \"Documenta\""));
    assert!(json.contains("\"keep_alive\": true"));
    assert!(json.contains("\"model\": \"opus\""));
    assert!(
        sandbox
            .run(&["context", "demo/ejecutora"])
            .contains("Coordinator of the category: demo-documental")
    );
    assert!(
        sandbox
            .fail(&["set", "demo/documental"])
            .contains("nothing to change")
    );
}

// PLAN §0.6: rm needs confirmation; --yes confirms
#[test]
fn rm_asks_for_confirmation_and_yes_confirms() {
    let sandbox = Sandbox::new().with_demo();
    assert!(sandbox.fail(&["rm", "demo/documental"]).contains("--yes"));
    assert!(sandbox.run(&["status"]).contains("demo-documental"));
    let removed = sandbox.run(&["rm", "demo/documental", "--yes"]);
    assert!(
        removed.contains("claude --resume"),
        "it says the transcript is untouched"
    );
    assert!(!sandbox.run(&["status"]).contains("demo-documental"));
}

// docs/decisiones.md: `cat rm` keeps board.md and prints where it is
#[test]
fn removing_a_category_keeps_its_board_and_says_where_it_is() {
    let sandbox = Sandbox::new().with_demo();
    let board = sandbox.home().join("categories/demo/board.md");
    fs::write(&board, "# Demo board\n\nnotas\n").unwrap();
    assert!(sandbox.fail(&["cat", "rm", "demo"]).contains("--yes"));

    let removed = sandbox.run(&["cat", "rm", "demo", "--yes"]);

    assert!(removed.contains(board.to_str().unwrap()));
    assert!(fs::read_to_string(&board).unwrap().contains("notas"));
    assert!(!sandbox.run(&["cat", "list"]).contains("demo"));
    assert!(sandbox.fail(&["status", "demo"]).contains("not found"));
}

// PLAN §3.1: the sound when done is set per category, from the command line too.
#[test]
fn a_category_turns_its_sound_off_and_on() {
    let sandbox = Sandbox::new().with_demo();
    let listed = |sandbox: &Sandbox| sandbox.run(&["cat", "list", "--json"]);
    assert!(listed(&sandbox).contains("\"notify\": true"));

    let said = sandbox.run(&["cat", "set", "demo", "--sound", "false"]);
    assert!(said.contains("sound when done off"), "{said}");
    assert!(listed(&sandbox).contains("\"notify\": false"));
    assert!(
        sandbox
            .run(&["cat", "list"])
            .contains("sound when done off")
    );

    sandbox.run(&["cat", "set", "demo", "--sound"]);
    assert!(listed(&sandbox).contains("\"notify\": true"));
    assert!(
        sandbox
            .fail(&["cat", "set", "missing", "--sound", "false"])
            .contains("not found")
    );
}
