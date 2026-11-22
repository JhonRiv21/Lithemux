use crate::modules::launch::domain::shell_answer::{marked_answer, marked_script};

const MARKER: &str = "lithemux-4f2a";

// Audit of the delta (D-23): only what the script printed is the answer
#[test]
fn the_answer_is_what_lies_between_the_two_markers() {
    let output = format!(
        "motd of the profile\n/decoy/claude\n\n{MARKER}\n/real/claude\n\n{MARKER}\n/late/claude\n"
    );
    assert_eq!(marked_answer(&output, MARKER), Some("\n/real/claude\n\n"));
}

// A script that was cut short gave no answer
#[test]
fn output_without_both_markers_is_no_answer() {
    assert_eq!(marked_answer("/some/claude\n", MARKER), None);
    let cut_short = format!("\n{MARKER}\n/half/an/ans");
    assert_eq!(marked_answer(&cut_short, MARKER), None);
}

// The markers go around the script, on lines of their own
#[test]
fn the_script_is_wrapped_between_two_marker_lines() {
    assert_eq!(
        marked_script("command -v claude", MARKER),
        "printf '\\nlithemux-4f2a\\n'; command -v claude; printf '\\nlithemux-4f2a\\n'"
    );
}
