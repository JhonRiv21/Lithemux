use crate::launch_arguments::LaunchRequest;

fn parse(arguments: &[&str]) -> Result<LaunchRequest, String> {
    let arguments: Vec<String> = arguments.iter().map(|a| a.to_string()).collect();
    LaunchRequest::parse(&arguments)
}

// PLAN §2.2 (carried from 2.1): the app always starts with its window and
// its session host; nothing is opened from the command line.
#[test]
fn without_arguments_the_app_opens_its_window() {
    let request = parse(&[]).unwrap();
    assert!(request.with_window);
    assert!(request.targets.is_empty());
}

// PLAN §2.2 (carried from 2.1): `--open` is retired from the window, where
// the sidebar replaces it.
#[test]
fn open_is_refused_with_a_window() {
    let error = parse(&["--open", "demo/uno"]).unwrap_err();
    assert!(error.contains("--open needs --no-window"), "{error}");
}

// PLAN §2.2 (carried from 2.1): the spike that ran any program in two
// panes, without the host, is gone.
#[test]
fn a_program_to_run_is_not_understood_any_more() {
    let error = parse(&["/some/program", "argument"]).unwrap_err();
    assert!(error.contains("unexpected argument"), "{error}");
}

// For the tests of how the app exits (D-35): without a window, every
// `--open` names one session, in order.
#[test]
fn without_a_window_every_open_names_one_session_in_order() {
    for arguments in [
        &["--no-window", "--open", "demo/uno", "--open", "demo/dos"][..],
        &["--open", "demo/uno", "--open", "demo/dos", "--no-window"],
    ] {
        let request = parse(arguments).unwrap();
        assert_eq!(request.targets, ["demo/uno", "demo/dos"]);
        assert!(!request.with_window);
    }
    assert!(parse(&["--no-window"]).unwrap().targets.is_empty());
}

// Audit of the §2.1 (7): nothing given on the command line is dropped in
// silence
#[test]
fn an_open_without_its_session_is_an_error() {
    for arguments in [
        &["--no-window", "--open"][..],
        &["--no-window", "--open", "demo/uno", "--open"],
        &["--no-window", "--open", "--open", "demo/uno"],
    ] {
        let error = parse(arguments).unwrap_err();
        assert!(error.contains("needs a <category>/<session>"), "{error}");
    }
}

// Audit of the §2.1 (7)
#[test]
fn an_argument_that_is_not_understood_is_an_error() {
    for arguments in [
        &["--no-window", "--open", "demo/uno", "demo/dos"][..],
        &["demo/uno", "--no-window"],
        &["--no-window", "--verbose"],
    ] {
        let error = parse(arguments).unwrap_err();
        assert!(error.contains("unexpected argument"), "{error}");
    }
}

// Audit of the §2.1 (7): one terminal cannot be shown twice
#[test]
fn the_same_session_twice_is_an_error() {
    let error = parse(&["--no-window", "--open", "demo/uno", "--open", "demo/uno"]).unwrap_err();
    assert_eq!(error, "demo/uno was given twice");
}
