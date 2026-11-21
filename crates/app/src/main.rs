mod spike_window;
mod text_check;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    if env::args().nth(1).as_deref() == Some(text_check::FLAG) {
        return text_check::run();
    }
    spike_window::run();
    ExitCode::SUCCESS
}
