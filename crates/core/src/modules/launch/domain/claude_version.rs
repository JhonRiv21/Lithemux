/// `claude --version` prints something like `2.1.330 (Claude Code)`.
pub fn parse_version_output(output: &str) -> Option<String> {
    output.split_whitespace().next().map(str::to_string)
}
