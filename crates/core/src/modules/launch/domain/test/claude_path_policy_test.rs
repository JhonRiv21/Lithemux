use crate::modules::launch::domain::claude_path_policy::{
    ClaudePathProblem, PathKind, claude_path_problem,
};

const RUNS: PathKind = PathKind::File { executable: true };

// ronda 3C: each reason a claude_path cannot run is told apart, so the
// message says what to fix.
#[test]
fn each_reason_a_claude_path_cannot_run_is_told_apart() {
    let cases = [
        ("\"/bin/claude\"", RUNS, Some(ClaudePathProblem::Quoted)),
        ("'/bin/claude'", RUNS, Some(ClaudePathProblem::Quoted)),
        ("bin/claude", RUNS, Some(ClaudePathProblem::NotAbsolute)),
        (
            "/bin/claude",
            PathKind::Missing,
            Some(ClaudePathProblem::Missing),
        ),
        (
            "/bin/claude",
            PathKind::Directory,
            Some(ClaudePathProblem::Directory),
        ),
        (
            "/bin/claude",
            PathKind::File { executable: false },
            Some(ClaudePathProblem::NotExecutable),
        ),
        ("/bin/claude", RUNS, None),
    ];
    for (value, kind, expected) in cases {
        assert_eq!(claude_path_problem(value, |_| kind), expected, "{value}");
    }
}

// ronda 3C: a quoted path to a program that runs was called «not an
// executable». The quotes are the problem, and the disk is not even asked.
#[test]
fn a_quoted_path_is_refused_for_its_quotes_without_looking_at_the_disk() {
    let problem = claude_path_problem("\"/opt/tools/claude\"", |_| panic!("the disk was asked"));

    assert_eq!(problem, Some(ClaudePathProblem::Quoted));
}
