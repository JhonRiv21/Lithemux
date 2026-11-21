use crate::modules::launch::domain::claude_version::parse_version_output;

// docs/decisiones.md: `open` stores what `claude --version` reports
#[test]
fn the_version_is_the_first_word_of_the_output() {
    assert_eq!(
        parse_version_output("2.1.330 (Claude Code)\n"),
        Some("2.1.330".to_string())
    );
    assert_eq!(parse_version_output("  \n"), None);
}
