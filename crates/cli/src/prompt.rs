use std::io::{self, BufRead, IsTerminal, Write};

/// Asks for confirmation on the terminal. Without a terminal the answer is no:
/// scripts must pass `--yes`.
pub fn confirm(question: &str) -> io::Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(false);
    }
    print!("{question} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}
