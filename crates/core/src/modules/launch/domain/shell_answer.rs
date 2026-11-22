//! Telling the answer of a shell from everything else it prints. A login
//! shell runs the user's profile first, and the profile, its exit hooks and
//! the jobs it starts all write to the same output as the command asked for.

/// `script` between two lines that carry `marker`, a text nothing else will
/// ever print.
pub fn marked_script(script: &str, marker: &str) -> String {
    format!("printf '\\n{marker}\\n'; {script}; printf '\\n{marker}\\n'")
}

/// What was printed between the two markers; `None` when the script did not
/// run to its end.
pub fn marked_answer<'a>(output: &'a str, marker: &str) -> Option<&'a str> {
    let (_, after_first) = output.split_once(marker)?;
    let (answer, _) = after_first.split_once(marker)?;
    Some(answer)
}
