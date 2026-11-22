use crate::launch_arguments::OpenRequest;

fn parse(arguments: &[&str]) -> Option<Result<OpenRequest, String>> {
    let arguments: Vec<String> = arguments.iter().map(|a| a.to_string()).collect();
    OpenRequest::parse(&arguments)
}

// PLAN §2.1: `lithemux-app --open <cat>/<session> [--open <cat>/<session>]`
#[test]
fn every_open_names_one_session_in_order() {
    let request = parse(&["--open", "demo/uno", "--open", "demo/dos"])
        .unwrap()
        .unwrap();
    assert_eq!(request.targets, ["demo/uno", "demo/dos"]);
    assert!(request.with_window);
}

// Without `--open` the app is not being asked to open sessions
#[test]
fn other_arguments_are_not_a_request_to_open() {
    assert_eq!(parse(&[]), None);
    assert_eq!(parse(&["/some/program", "argument"]), None);
}

// Audit of the §2.1 (7): nothing given on the command line is dropped in
// silence
#[test]
fn an_open_without_its_session_is_an_error() {
    for arguments in [
        &["--open"][..],
        &["--open", "demo/uno", "--open"],
        &["--open", "--open", "demo/uno"],
    ] {
        let error = parse(arguments).unwrap().unwrap_err();
        assert!(error.contains("needs a <category>/<session>"), "{error}");
    }
}

// Audit of the §2.1 (7)
#[test]
fn an_argument_that_is_not_understood_is_an_error() {
    for arguments in [
        &["--open", "demo/uno", "demo/dos"][..],
        &["demo/uno", "--open", "demo/dos"],
        &["--open", "demo/uno", "--verbose"],
    ] {
        let error = parse(arguments).unwrap().unwrap_err();
        assert!(error.contains("unexpected argument"), "{error}");
    }
}

// Audit of the §2.1 (7): one terminal cannot be shown twice
#[test]
fn the_same_session_twice_is_an_error() {
    let error = parse(&["--open", "demo/uno", "--open", "demo/uno"])
        .unwrap()
        .unwrap_err();
    assert_eq!(error, "demo/uno was given twice");
}

// For the tests of how the app exits
#[test]
fn no_window_can_be_asked_for_anywhere() {
    for arguments in [
        &["--no-window", "--open", "demo/uno"][..],
        &["--open", "demo/uno", "--no-window"],
    ] {
        let request = parse(arguments).unwrap().unwrap();
        assert_eq!(request.targets, ["demo/uno"]);
        assert!(!request.with_window);
    }
}
